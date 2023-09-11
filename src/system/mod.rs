mod owned;
mod uf2;

#[cfg(feature = "updater")]
pub use owned::Updater;
pub use owned::{MemberStrings, SystemVec};
pub use uf2::*;

use alloc::borrow::Cow;

pub trait Member {
    fn name(&self) -> &str;
    fn pronouns(&self) -> &str;
}

impl<M: Member> Member for &M {
    fn name(&self) -> &str {
        (*self).name()
    }

    fn pronouns(&self) -> &str {
        (*self).pronouns()
    }
}

pub trait System {
    fn name(&self) -> Cow<'_, str>;
    fn member_count(&self) -> usize;
    fn member(&self, index: usize) -> &dyn Member;
}

impl<S: System> System for &S {
    fn name(&self) -> Cow<'_, str> {
        (*self).name()
    }

    fn member_count(&self) -> usize {
        (*self).member_count()
    }

    fn member(&self, index: usize) -> &dyn Member {
        (*self).member(index)
    }
}
