#[cfg(feature = "alloc")]
mod alloc;

pub mod system_capnp {
    include!(concat!(env!("OUT_DIR"), "/system/system_capnp.rs"));
}

#[cfg(feature = "updater")]
pub use alloc::Updater;
#[cfg(feature = "alloc")]
pub use alloc::{MemberStrings, SystemVec};
use capnp::message::ReaderSegments;

pub use capnp;

pub trait Member {
    fn name<'a>(&'a self) -> impl AsRef<str> + 'a;
    fn pronouns<'a>(&'a self) -> impl AsRef<str> + 'a;
}

impl<M: Member> Member for &M {
    fn name<'a>(&'a self) -> impl AsRef<str> + 'a {
        (*self).name()
    }

    fn pronouns<'a>(&'a self) -> impl AsRef<str> + 'a {
        (*self).pronouns()
    }
}

pub trait System {
    fn name<'a>(&'a self) -> impl AsRef<str> + 'a;
    fn member_count(&self) -> usize;
    fn member<'a>(&'a self, index: usize) -> impl Member + 'a;

    /// Function to validate the system.
    ///
    /// This returns true in the default implementation, assuming a system cannot be invalid.
    fn is_valid(&self) -> bool {
        true
    }
}

impl<S: System> System for &S {
    fn name<'a>(&'a self) -> impl AsRef<str> + 'a {
        (*self).name()
    }

    fn member_count(&self) -> usize {
        (*self).member_count()
    }

    fn member<'a>(&'a self, index: usize) -> impl Member + 'a {
        (*self).member(index)
    }

    fn is_valid(&self) -> bool {
        (*self).is_valid()
    }
}

pub struct SystemReader<S>
where
    S: ReaderSegments,
{
    pub(crate) reader: capnp::message::Reader<S>,
}

impl<S: ReaderSegments> SystemReader<S> {
    pub fn reader(&self) -> capnp::Result<system_capnp::system::Reader> {
        self.reader.get_root()
    }
}

impl<'a> SystemReader<capnp::serialize::NoAllocSliceSegments<'a>> {
    pub fn from_byte_slice(slice: &mut &'a [u8]) -> capnp::Result<Self> {
        let reader =
            capnp::serialize::read_message_from_flat_slice_no_alloc(slice, Default::default())?;
        Ok(Self { reader })
    }
}

impl SystemReader<capnp::serialize::NoAllocSliceSegments<'static>> {
    pub unsafe fn from_linker_symbols() -> Self {
        let mut bytes = unsafe { Self::flat_bytes() };

        Self::from_byte_slice(&mut bytes).unwrap()
    }

    unsafe fn flat_bytes() -> &'static [u8] {
        extern "C" {
            static __ssystem_start: u8;
            static __ssystem_end: u8;
        }

        let len = unsafe {
            (&__ssystem_end as *const u8 as usize) - (&__ssystem_start as *const u8 as usize)
        };
        #[cfg(feature = "defmt")]
        defmt::trace!(
            "reading system from addr: {}, len: {}",
            unsafe { &__ssystem_start as *const u8 as usize },
            len
        );
        unsafe { core::slice::from_raw_parts(&__ssystem_start as *const u8, len) }
    }
}

impl<S: ReaderSegments> System for SystemReader<S> {
    fn name(&self) -> &str {
        self.reader().unwrap().get_name().unwrap().to_str().unwrap()
    }

    fn member_count(&self) -> usize {
        self.reader().unwrap().get_members().unwrap().len() as usize
    }

    fn member<'b>(&'b self, index: usize) -> MemberReader<'b> {
        let reader = self
            .reader()
            .unwrap()
            .get_members()
            .unwrap()
            .get(index as u32);
        MemberReader { reader }
    }
}

pub struct MemberReader<'a> {
    pub(crate) reader: system_capnp::member::Reader<'a>,
}

impl<'a> Member for MemberReader<'a> {
    fn name(&self) -> &str {
        self.reader.get_name().unwrap().to_str().unwrap()
    }

    fn pronouns(&self) -> &str {
        self.reader
            .get_pronouns()
            .unwrap()
            .to_str()
            .unwrap_or_default()
    }
}
