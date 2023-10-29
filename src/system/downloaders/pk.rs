use alloc::string::{String, ToString};

use super::transform_name;
use crate::system::{MemberStrings, SystemVec};
use crate::usb::SystemId;

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
        system.source_id = SystemId::PluralKit { id: id.to_string() };
        for member in members {
            system.members.push(MemberStrings {
                name: transform_name(&member.display_name.unwrap_or_else(|| member.name)),
                pronouns: transform_name(member.pronouns.as_deref().unwrap_or("")),
            });
        }

        Ok(system)
    }
}

impl super::Downloader for PkDownloader {
    async fn set_useragent(&mut self, useragent: impl ToString) {
        self.client.user_agent = useragent.to_string();
    }

    async fn get(&self, args: impl AsRef<str>) -> Result<SystemVec, reqwest::Error> {
        self.get(args).await
    }
}
