use alloc::string::ToString;

use crate::system::{MemberStrings, SystemVec};

use super::transform_name;

pub struct PkDownloader {
    pub client: pkrs::client::PkClient,
}

impl PkDownloader {
    pub fn new() -> Self {
        Self {
            client: pkrs::client::PkClient {
                user_agent: "sysbadge downloader".to_string(),
                ..Default::default()
            },
        }
    }

    pub async fn get(&self, id: impl AsRef<str>) -> Result<SystemVec, reqwest::Error> {
        let id = pkrs::model::PkId(id.as_ref().to_string());
        let info = self.client.get_system(&id).await?;
        let members = self.client.get_system_members(&id).await?;

        let mut system = SystemVec::new(info.name.unwrap_or("no system name".to_string()));
        system.hid = Some(id.to_string());
        for member in members {
            system.members.push(MemberStrings {
                name: transform_name(&member.display_name.unwrap_or_else(|| member.name)),
                pronouns: transform_name(member.pronouns.as_deref().unwrap_or("")),
            });
        }

        Ok(system)
    }
}
