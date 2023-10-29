//! File header and helper functions.
//!
use core::{cmp, mem};

pub mod binding;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct FileHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub flags: Flags,

    /// Length in octets of the binary blob.
    pub bin_length: u32,
    /// Lenght in octets of the json blob.
    ///
    /// Only used if [`Flags::JSON_BLOB`] is set.
    pub json_length: u32,

    /// System name to have easy access.
    pub system_name: [u8; 192],
}

impl FileHeader {
    pub const MAGIC: [u8; 4] = [0x53, 0x59, 0x42, 0x44];

    pub const fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }

    pub const fn from_bytes(buf: &[u8; mem::size_of::<Self>()]) -> &Self {
        unsafe { &*(buf as *const u8 as *const Self) }
    }

    pub fn empty() -> Self {
        Self {
            magic: Self::MAGIC,
            version: 1u32,
            ..unsafe { core::mem::zeroed() }
        }
    }
}

impl Default for FileHeader {
    fn default() -> Self {
        Self {
            flags: Flags::default(),
            ..Self::empty()
        }
    }
}

#[cfg(feature = "alloc")]
#[repr(C)]
pub struct FileWriter<'a> {
    pub system: &'a crate::system::SystemVec,
    pub flags: Flags,
}

#[cfg(feature = "alloc")]
impl<'a> FileWriter<'a> {
    pub fn new(system: &'a crate::system::SystemVec) -> Self {
        Self {
            system,
            flags: Flags::default(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        use sha2::Digest;
        let mut vec = Vec::new();

        let mut header = FileHeader::default();
        header.flags = self.flags;
        let len = cmp::min(
            self.system.name.len(),
            mem::size_of_val(&header.system_name) - 1,
        );
        (&mut header.system_name[..len]).copy_from_slice(&self.system.name.as_bytes()[..len]);

        #[cfg(feature = "tracing")]
        tracing::debug!("Saving {} byte system: {}", len, self.system.name);

        let bin = self.system.get_bin();
        header.bin_length = bin.len() as u32;

        let (json, json_len) = if self.flags.contains(Flags::JSON_BLOB) {
            #[cfg(feature = "tracing")]
            tracing::debug!("Encoding json blob");

            let json = serde_json::to_vec(self.system).unwrap(); // FIXME
            let len = json.len() as u32;
            (json, len)
        } else {
            (Vec::new(), 0)
        };
        header.json_length = json_len;

        vec.extend(header.as_bytes());

        let mut hasher = if self.flags.contains(Flags::SHA2_CHECKSUM) {
            Some(sha2::Sha256::new())
        } else {
            None
        };

        vec.extend(&bin);
        hasher.as_mut().map(|h| h.update(&bin));
        vec.extend(&json);
        hasher.as_mut().map(|h| h.update(&json));

        if let Some(hasher) = hasher {
            let hash = hasher.finalize();
            vec.extend(&hash[..]);
        }

        vec
    }
}

#[derive(Debug, Clone)]
pub struct File {
    pub header: FileHeader,
    pub payload: Vec<u8>,
    pub json: Option<Vec<u8>>,
    pub rest: Vec<u8>,
}

impl File {
    #[cfg(feature = "std")]
    pub fn read<Reader: std::io::Read>(reader: &mut Reader) -> std::io::Result<Option<Self>> {
        let mut header = [0; mem::size_of::<FileHeader>()];
        reader.read_exact(&mut header)?;
        let header = FileHeader::from_bytes(&header);

        if header.magic != FileHeader::MAGIC {
            #[cfg(feature = "tracing")]
            tracing::warn!("Invalid file header {:x?}", header.magic);

            return Ok(None);
        }

        if header.version != 1 {
            #[cfg(feature = "tracing")]
            tracing::warn!("Cannot handle file version {}", header.version);

            return Err(std::io::Error::from(std::io::ErrorKind::InvalidData));
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("Read header: {:?}", header);

        let mut payload = Vec::with_capacity(header.bin_length as usize);
        unsafe {
            core::ptr::write_bytes(payload.as_mut_ptr(), 0, header.bin_length as usize);
            payload.set_len(header.bin_length as usize);
        }
        reader.read_exact(&mut payload)?;

        let json = if header.flags.contains(Flags::JSON_BLOB) {
            let mut json = Vec::with_capacity(header.json_length as usize);
            unsafe {
                core::ptr::write_bytes(json.as_mut_ptr(), 0, header.json_length as usize);
                json.set_len(header.json_length as usize);
            }
            reader.read_exact(&mut json)?;
            Some(json)
        } else {
            None
        };

        let mut rest = Vec::new();
        reader.read_to_end(&mut rest)?;

        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Read file: payload={}, json={}, rest={}",
            payload.len(),
            json.as_ref().map(Vec::len).unwrap_or_default(),
            rest.len()
        );

        Ok(Some(Self {
            header: header.clone(),
            payload,
            json,
            rest,
        }))
    }

    pub fn from_byte_slice(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() < mem::size_of::<FileHeader>() {
            return Err(());
        }
        let header_end = core::mem::size_of::<FileHeader>();
        let header = FileHeader::from_bytes((&slice[0..header_end]).try_into().unwrap());

        if header.magic != FileHeader::MAGIC {
            #[cfg(feature = "tracing")]
            tracing::warn!("Invalid file header {:x?}", header.magic);

            return Err(());
        }

        if header.version != 1 {
            #[cfg(feature = "tracing")]
            tracing::warn!("Cannot handle file version {}", header.version);

            return Err(());
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("Read header: {:?}", header);

        let mut payload = Vec::with_capacity(header.bin_length as usize);
        unsafe {
            core::ptr::copy_nonoverlapping(
                slice.as_ptr().add(header_end),
                payload.as_mut_ptr(),
                header.bin_length as usize,
            );
            payload.set_len(header.bin_length as usize);
        }

        let json = if header.flags.contains(Flags::JSON_BLOB) {
            let mut json = Vec::with_capacity(header.json_length as usize);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    slice.as_ptr().add(header_end + header.bin_length as usize),
                    json.as_mut_ptr(),
                    header.json_length as usize,
                );
                json.set_len(header.json_length as usize);
            }
            Some(json)
        } else {
            None
        };

        let rest_start = header_end + header.bin_length as usize + header.json_length as usize;
        let rest = slice[rest_start..].to_vec();

        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Read file: payload={}, json={}, rest={}",
            payload.len(),
            json.as_ref().map(Vec::len).unwrap_or_default(),
            rest.len()
        );

        Ok(Self {
            header: header.clone(),
            payload,
            json,
            rest,
        })
    }

    #[cfg(feature = "std")]
    pub fn read_or_bin<Reader: std::io::Read>(reader: &mut Reader) -> std::io::Result<Self> {
        if let Some(file) = Self::read(reader)? {
            Ok(file)
        } else {
            let mut payload = Vec::new();
            reader.read_to_end(&mut payload)?;
            Ok(Self {
                header: FileHeader::empty(),
                payload,
                json: None,
                rest: Vec::new(),
            })
        }
    }

    pub fn verify(&self) -> bool {
        if !self.header.flags.contains(Flags::SHA2_CHECKSUM) {
            #[cfg(feature = "tracing")]
            tracing::warn!("Trying to verify a file that has no checksum");
            return true;
        }

        use sha2::Digest;

        let mut hasher = sha2::Sha256::new();
        hasher.update(&self.payload);
        self.json.as_ref().map(|b| hasher.update(b));

        let hash = &hasher.finalize()[..];

        let checksum = self.rest.get(..hash.len());
        if checksum.is_none() {
            return false;
        }
        let checksum = checksum.unwrap();

        #[cfg(feature = "tracing")]
        tracing::debug!("checksum: {:X?}, expecting: {:X?}", checksum, hash);

        hash == checksum
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    pub struct Flags: u32 {
        /// Use Sha2 checksum to verify binary blob.
        const SHA2_CHECKSUM = 0b00000001;

        /// Add a JSON blob after the binary blob
        const JSON_BLOB = 0b00001000;

        // The source may set any bits
        // const _ = !0;
    }
}

impl Default for Flags {
    fn default() -> Self {
        let mut bits = Self::empty();
        bits.insert(Self::SHA2_CHECKSUM);
        bits.insert(Self::JSON_BLOB);
        bits
    }
}

#[cfg(test)]
mod test {
    use core::mem;

    use super::*;

    #[test]
    fn test_sizes() {
        // test that size does not change
        assert_eq!(mem::size_of::<Flags>(), 4);
        assert_eq!(mem::size_of::<FileHeader>(), 212);
    }

    #[test]
    fn write_and_read_slice() {
        let mut system = crate::system::SystemVec::new("test".to_string());
        system.members.push(crate::system::MemberStrings {
            name: "test".to_string(),
            pronouns: "test".to_string(),
        });
        let writer = FileWriter::new(&system);
        let vec = writer.to_vec();
        let file = File::from_byte_slice(&vec).unwrap();
        assert_eq!(file.header.flags, writer.flags);
        assert_eq!(file.header.bin_length, system.get_bin().len() as u32);
        assert_ne!(file.header.json_length, 0);
        assert_eq!(file.payload, system.get_bin());
        assert!(file.json.is_some());
        assert!(file.verify());
    }

    #[test]
    fn write_and_read_slice_no_json() {
        let mut system = crate::system::SystemVec::new("test".to_string());
        system.members.push(crate::system::MemberStrings {
            name: "test".to_string(),
            pronouns: "test".to_string(),
        });
        let mut writer = FileWriter::new(&system);
        writer.flags.remove(Flags::JSON_BLOB);
        let vec = writer.to_vec();
        let file = File::from_byte_slice(&vec).unwrap();
        assert_eq!(file.header.flags, writer.flags);
        assert_eq!(file.header.bin_length, system.get_bin().len() as u32);
        assert_eq!(file.header.json_length, 0);
        assert_eq!(file.payload, system.get_bin());
        assert_eq!(file.json, None);
        assert!(file.verify());
    }
}
