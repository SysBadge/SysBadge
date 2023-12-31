use crate::system::Member;
use crate::System;

use alloc::string::String;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub enum SourceId {
    PluralKit(String),
    Pronouns(String),
}

/// Owned system utilizing a vec to hold members.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct SystemVec {
    /// Name of the system
    pub name: String,
    /// Optional source id
    pub source_id: Option<SourceId>,
    /// Vector of members
    pub members: alloc::vec::Vec<MemberStrings>,
}

impl SystemVec {
    pub fn new(name: String) -> Self {
        Self {
            name,
            source_id: None,
            members: alloc::vec::Vec::new(),
        }
    }

    #[cfg(feature = "downloader-pk")]
    #[inline]
    pub async fn fetch_pk(id: impl AsRef<str>) -> Result<Self, reqwest::Error> {
        super::downloaders::PkDownloader::new().get(id).await
    }

    pub fn sort_members(&mut self) {
        self.members
            .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }
}

impl SystemVec {
    #[cfg(any(feature = "uf2", doc))]
    pub fn get_uf2(&self, offset: u32) -> alloc::vec::Vec<u8> {
        let buf = self.get_bin();
        Self::bin_to_uf2(&buf, offset)
    }

    #[cfg(any(feature = "uf2", doc))]
    pub fn bin_to_uf2(bin: &[u8], offset: u32) -> alloc::vec::Vec<u8> {
        uf2::bin_to_uf2(bin, uf2::RP2040_FAMILY_ID, offset)
    }

    pub fn get_bin(&self) -> alloc::vec::Vec<u8> {
        let builder = self.capnp_builder();
        capnp::serialize::write_message_to_words(&builder)
    }

    fn capnp_builder(&self) -> capnp::message::Builder<capnp::message::HeapAllocator> {
        let mut builder = capnp::message::Builder::new_default();
        {
            let mut system = builder.init_root::<super::system_capnp::system::Builder>();
            system.set_name(self.name.as_str().into());
            if let Some(source_id) = &self.source_id {
                match source_id {
                    SourceId::PluralKit(hid) => {
                        system.set_pk_hid(hid.as_str().into());
                    }
                    SourceId::Pronouns(id) => {
                        system.set_pronouns(id.as_str().into());
                    }
                }
            }

            let mut members = system.init_members(self.members.len() as u32);
            for (i, member) in self.members.iter().enumerate() {
                let mut out = members.reborrow().get(i as u32);
                out.set_name(member.name.as_str().into());
                out.set_pronouns(member.pronouns.as_str().into());
            }
        }
        builder
    }
}

impl System for SystemVec {
    fn name(&self) -> &str {
        &self.name
    }

    fn member_count(&self) -> usize {
        self.members.len()
    }

    fn member(&self, index: usize) -> &MemberStrings {
        &self.members[index]
    }
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct MemberStrings {
    pub name: String,
    pub pronouns: String,
}

impl Member for MemberStrings {
    fn name(&self) -> &str {
        &self.name
    }

    fn pronouns(&self) -> &str {
        &self.pronouns
    }
}

#[cfg(feature = "uf2")]
mod uf2 {
    use alloc::vec::Vec;

    /// copied and modified from the uf2 crate

    const UF2_MAGIC_START0: u32 = 0x0A324655; // "UF2\n"
    const UF2_MAGIC_START1: u32 = 0x9E5D5157; // Randomly selected
    const UF2_MAGIC_END: u32 = 0x0AB16F30; // Ditto

    pub const RP2040_FAMILY_ID: u32 = 0xe48bff56;

    pub fn bin_to_uf2(bytes: &[u8], family_id: u32, app_start_addr: u32) -> Vec<u8> {
        let datapadding = 512 - 256 - 32 - 4;
        let nblocks: u32 = ((bytes.len() + 255) / 256) as u32;
        let mut outp: Vec<u8> = Vec::new();
        for blockno in 0..nblocks {
            let ptr = 256 * blockno;
            let chunk = match bytes.get(ptr as usize..ptr as usize + 256) {
                Some(bytes) => bytes.to_vec(),
                None => {
                    let mut chunk = bytes[ptr as usize..bytes.len()].to_vec();
                    while chunk.len() < 256 {
                        chunk.push(0);
                    }
                    chunk
                }
            };
            let mut flags: u32 = 0;
            if family_id != 0 {
                flags |= 0x2000
            }

            // header
            outp.extend(UF2_MAGIC_START0.to_le_bytes());
            outp.extend(UF2_MAGIC_START1.to_le_bytes());
            outp.extend(flags.to_le_bytes());
            outp.extend((ptr + app_start_addr).to_le_bytes());
            outp.extend(256u32.to_le_bytes());
            outp.extend(blockno.to_le_bytes());
            outp.extend(nblocks.to_le_bytes());
            outp.extend(family_id.to_le_bytes());

            // data
            outp.extend(chunk);
            outp.extend(core::iter::repeat(0).take(datapadding));

            // foote
            outp.extend(UF2_MAGIC_END.to_le_bytes());
        }
        outp
    }
}
