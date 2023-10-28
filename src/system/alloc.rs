use alloc::string::String;

use crate::system::Member;
use crate::usb::SystemId;
use crate::System;

/// Owned system utilizing a vec to hold members.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone)]
pub struct SystemVec {
    /// Name of the system
    pub name: String,
    /// Optional source id
    pub source_id: SystemId,
    /// Vector of members
    pub members: alloc::vec::Vec<MemberStrings>,
}

impl SystemVec {
    pub fn new(name: String) -> Self {
        Self {
            name,
            source_id: SystemId::None,
            members: alloc::vec::Vec::new(),
        }
    }

    #[cfg(feature = "downloader-pk")]
    #[inline]
    pub async fn fetch_pk(id: impl AsRef<str>) -> Result<Self, reqwest::Error> {
        super::downloaders::PkDownloader::new().get(id).await
    }

    pub fn sort_members(&mut self) {
        self.members.sort_by(|a, b| a.name.cmp(&b.name));
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

    #[cfg(feature = "file")]
    pub fn get_file(&self) -> alloc::vec::Vec<u8> {
        super::file::FileWriter::new(&self).to_vec()
    }

    fn capnp_builder(&self) -> capnp::message::Builder<capnp::message::HeapAllocator> {
        let mut builder = capnp::message::Builder::new_default();
        {
            let mut system = builder.init_root::<super::system_capnp::system::Builder>();
            system.set_name(self.name.as_str().into());
            match &self.source_id {
                SystemId::None => {},
                SystemId::PluralKit { id } => {
                    system.set_pk_hid(id.as_str().into());
                },
                SystemId::PronounsCC { id } => {
                    system.set_pronouns(id.as_str().into());
                },
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
