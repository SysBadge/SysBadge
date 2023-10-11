use anyhow::Result;
use clap::ValueEnum;
use clio::Output;
use std::fmt::{Display, Formatter};
use std::io::Write;

pub fn command() -> clap::Command {
    let cmd = clap::Command::new("dl")
        .about("Download a system")
        .arg(
            clap::Arg::new("id")
                .required(true)
                .index(1)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("format")
                .long("format")
                .short('f')
                .default_value("json")
                .value_parser(clap::builder::EnumValueParser::<DlFormat>::new()),
        )
        .arg(
            clap::Arg::new("output")
                .long("output")
                .short('o')
                .value_parser(clap::value_parser!(Output)),
        );
    dl_common_args(cmd)
}

pub fn dl_common_args(command: clap::Command) -> clap::Command {
    command
        .arg(
            clap::Arg::new("dl-source")
                .long("source")
                .short('s')
                .default_value("PluralKit")
                .value_parser(clap::builder::EnumValueParser::<
                    sysbadge::system::downloaders::Source,
                >::new()),
        )
        .arg(
            clap::Arg::new("dl-user-agent")
                .hide(true)
                .long("user-agent")
                .default_value("SysBadge CLI")
                .value_parser(clap::value_parser!(String)),
        )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum DlFormat {
    Bin,
    Json,
}

impl Display for DlFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bin => write!(f, "bin"),
            Self::Json => write!(f, "json"),
        }
    }
}

pub async fn run(matches: &clap::ArgMatches) -> Result<()> {
    let mut downloader = sysbadge::system::downloaders::GenericDownloader::new();
    downloader.useragent = matches.get_one::<String>("dl-user-agent").unwrap().clone();
    let source = matches
        .get_one::<sysbadge::system::downloaders::Source>("dl-source")
        .unwrap();
    let id = matches.get_one::<String>("id").unwrap();
    let output = matches.get_one::<Output>("output");
    let format = *matches.get_one::<DlFormat>("format").unwrap();

    let mut system = downloader.get(*source, id).await.unwrap();
    system.sort_members();

    let mut output = match output {
        Some(o) => o.clone(),
        None => Output::new(&format!("{}.{}", system.name, format)).unwrap(),
    };

    let data = match format {
        DlFormat::Bin => system.get_bin(),
        DlFormat::Json => serde_json::ser::to_vec_pretty(&system).unwrap(),
    };

    output.write_all(&data).unwrap();

    Ok(())
}
