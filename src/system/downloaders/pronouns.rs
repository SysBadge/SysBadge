use core::cmp::Ordering;

use alloc::{
    format,
    string::{String, ToString},
};

const BASE_URL: &str = "https://pronouns.cc/api/";

#[derive(Debug)]
pub struct PronounsDownloader {
    base_url: String,
    client: reqwest::Client,
}

impl PronounsDownloader {
    pub fn new() -> Self {
        Self {
            base_url: BASE_URL.to_string(),
            client: Self::build_client("sysbadge downloader"),
        }
    }

    async fn get_user(&self, id: &str) -> Result<User, reqwest::Error> {
        let resp = self
            .client
            .get(&format!("{}v1/users/{}", self.base_url, id))
            .send()
            .await?
            .error_for_status()?;

        resp.json().await
    }

    fn build_client(ua: &str) -> reqwest::Client {
        let builder = reqwest::Client::builder();

        #[cfg(not(target_family = "wasm"))]
        let builder = builder.user_agent(ua);

        builder.build().unwrap()
    }
}

impl super::Downloader for PronounsDownloader {
    async fn set_useragent(&mut self, ua: impl ToString) {
        self.client = Self::build_client(&ua.to_string());
    }

    async fn get(&self, id: impl AsRef<str>) -> Result<super::SystemVec, reqwest::Error> {
        let user = self.get_user(id.as_ref()).await?;

        let mut system = super::SystemVec::new(transform_name(
            &user.display_name.unwrap_or_else(|| user.name),
        ));
        system.source_id = Some(crate::system::alloc::SourceId::Pronouns(user.sid.clone()));

        for member in user.members {
            let mut pronouns = member.pronouns.clone();
            pronouns.sort_by(|a, b| {
                if a.status == b.status {
                    return Ordering::Equal;
                }
                if a.status == "favourite" {
                    return Ordering::Less;
                } else if b.status == "favourite" {
                    return Ordering::Greater;
                } else {
                    return Ordering::Equal;
                }
            });

            system.members.push(MemberStrings {
                name: transform_name(&member.display_name.unwrap_or_else(|| member.name)),
                pronouns: transform_name(
                    &pronouns
                        .get(0)
                        .map(|p| {
                            p.display_text
                                .as_deref()
                                .unwrap_or_else(|| p.pronouns.as_str())
                        })
                        .unwrap_or_default(),
                ),
            })
        }

        Ok(system)
    }
}

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::system::{downloaders::transform_name, MemberStrings};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub sid: String,
    pub name: String,
    pub display_name: Option<String>,
    pub pronouns: Vec<UserPronouns>,
    pub members: Vec<UserMember>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserMember {
    pub sid: String,
    pub name: String,
    pub display_name: Option<String>,
    pub pronouns: Vec<UserPronouns>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserPronouns {
    pub pronouns: String,
    pub display_text: Option<String>,
    pub status: String,
}
