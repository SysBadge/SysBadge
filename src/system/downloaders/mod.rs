use alloc::string::{String, ToString};

#[cfg(feature = "downloader-pk")]
mod pk;

#[cfg(feature = "downloader-pk")]
pub use pk::PkDownloader;

use super::SystemVec;

pub trait Downloader {
    async fn set_useragent(&mut self, _useragent: impl ToString) {}
    async fn get(&self, args: impl AsRef<str>) -> Result<SystemVec, reqwest::Error>;
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    #[cfg(feature = "downloader-pk")]
    PluralKit,
}

impl Source {
    pub fn short_identifier(&self) -> &'static str {
        match self {
            Self::PluralKit => "pk",
        }
    }
}

impl core::fmt::Display for Source {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "downloader-pk")]
            Self::PluralKit => write!(f, "PluralKit"),
        }
    }
}

impl core::str::FromStr for Source {
    type Err = ParseError;

    #[cfg(feature = "clap")]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use clap::ValueEnum;

        for variant in Self::value_variants() {
            if variant.to_possible_value().unwrap().matches(s, true) {
                return Ok(*variant);
            }
        }
        Err(ParseError)
    }

    #[cfg(not(feature = "clap"))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "downloader-pk")]
            "pk" | "PluralKit" => Ok(Self::PluralKit),
            _ => Err(ParseError),
        }
    }
}

#[cfg(feature = "clap")]
impl clap::ValueEnum for Source {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            #[cfg(feature = "downloader-pk")]
            Self::PluralKit,
        ]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            #[cfg(feature = "downloader-pk")]
            Self::PluralKit => clap::builder::PossibleValue::new("PluralKit").alias("pk"),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError;

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "invalid source".fmt(f)
    }
}

impl core::error::Error for ParseError {}

pub struct GenericDownloader {
    pub useragent: String,
}

impl GenericDownloader {
    pub fn new() -> Self {
        Self {
            useragent: "sysbadge downloader".to_string(),
        }
    }

    pub async fn get(
        &self,
        source: Source,
        id: impl AsRef<str>,
    ) -> Result<SystemVec, reqwest::Error> {
        match source {
            #[cfg(feature = "downloader-pk")]
            Source::PluralKit => self.get_pk(id).await,
        }
    }

    #[cfg(feature = "downloader-pk")]
    pub async fn get_pk(&self, id: impl AsRef<str>) -> Result<SystemVec, reqwest::Error> {
        let mut downloader = PkDownloader::new();
        downloader.set_useragent(&self.useragent).await;
        downloader.get(id).await
    }
}

pub(crate) fn transform_name(input: &str) -> String {
    // Convert the input string to bytes
    let bytes = input.as_bytes();

    // Find the index of the first occurrence of more than 2 spaces or a tab
    let index = bytes.iter().enumerate().position(|(idx, &c)| {
        (c == b' ' && bytes.iter().skip(idx).take(3).all(|&x| x == b' ')) || c == b'\t'
    });

    // If such an index is found, truncate the input string at that position, else use the original input
    let filtered_input = match index {
        Some(idx) => &input[..idx],
        None => input,
    };

    // Filter out non-ASCII characters and create an iterator of chars
    let ascii_chars: String = filtered_input
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric() || c.is_ascii_punctuation() || matches!(c, ' ' | '\t')
        })
        .collect();

    // Trim leading and trailing whitespace
    let trimmed_ascii = ascii_chars.trim();

    // Convert the trimmed string to a new String
    String::from(trimmed_ascii)
}
