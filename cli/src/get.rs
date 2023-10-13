use std::io::Write;

use anyhow::{bail, Context, Result};
use clio::Output;
use sysbadge::system::SystemVec;
use sysbadge_usb::UsbSysBadge;

use crate::dl::DlFormat;

pub fn command() -> clap::Command {
    clap::Command::new("get")
        .about("Read value from a badge connected via usb")
        .subcommand(
            clap::Command::new("name").about("Get the name of the system loaded on the badge"),
        )
        .subcommand(
            clap::Command::new("member")
                .about("Get Member")
                .arg(
                    clap::Arg::new("id")
                        .value_parser(clap::value_parser!(u16))
                        .required(true),
                )
                .subcommand(clap::Command::new("name"))
                .subcommand(clap::Command::new("pronouns")),
        )
        .subcommand(
            clap::Command::new("id")
                .about("Get the ID that the system loaded on the badge is from"),
        )
        .subcommand(
            clap::Command::new("dl")
                .about("Download a system from a badge")
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
                ),
        )
}

pub async fn run(matches: &clap::ArgMatches) -> Result<()> {
    let mut badge = crate::find_badge(matches).await?;
    badge.set_timeout(std::time::Duration::from_secs(5));

    if matches.subcommand().is_none() {
        return get_name(badge, matches).await;
    }
    match matches.subcommand().unwrap() {
        ("name", matches) => get_name(badge, matches).await,
        ("member", matches) => get_member(badge, matches).await,
        ("id", matches) => get_id(badge, matches).await,
        ("dl", matches) => get_dl(badge, matches).await,
        _ => bail!("Unknown subcommand"),
    }
}

async fn get_name(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    _matches: &clap::ArgMatches,
) -> Result<()> {
    let name = badge.system_name().context("Failed to get name")?;
    println!("{}", name);

    Ok(())
}

async fn get_member(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    matches: &clap::ArgMatches,
) -> Result<()> {
    let id = matches.get_one::<u16>("id").context("Could not parse id")?;

    if matches.subcommand().is_none() {
        return get_member_name(badge, *id).await;
    }
    match matches.subcommand().unwrap() {
        ("name", _) => get_member_name(badge, *id).await,
        ("pronouns", _) => get_member_pronouns(badge, *id).await,
        _ => bail!("Unknown subcommand"),
    }
}

async fn get_member_name(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    id: u16,
) -> Result<()> {
    let name = badge
        .system_member_name(id)
        .context("Failed to get member name")?;
    println!("{}", name);

    Ok(())
}

async fn get_member_pronouns(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    id: u16,
) -> Result<()> {
    let pronouns = badge
        .system_member_pronouns(id)
        .context("Failed to get member pronouns")?;
    println!("{}", pronouns);

    Ok(())
}

async fn get_id(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    _matches: &clap::ArgMatches,
) -> Result<()> {
    let id = badge.system_id().context("Failed to get system id")?;
    println!("{}", id);

    Ok(())
}

async fn get_dl(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    matches: &clap::ArgMatches,
) -> Result<()> {
    let mut system = SystemVec::new(badge.system_name().context("Failed to get system name")?);
    system.source_id = badge.system_id().context("Failed to get system id")?;

    let member_count = badge
        .system_member_count()
        .context("Failed to get member count")?;
    system.members.reserve(member_count as usize);
    for i in 0..member_count {
        let name = badge
            .system_member_name(i)
            .context("Failed to get member name")?;
        let pronouns = badge
            .system_member_pronouns(i)
            .context("Failed to get member pronouns")?;
        system
            .members
            .push(sysbadge::system::MemberStrings { name, pronouns });
    }

    let output = matches.get_one::<Output>("output");
    let format = *matches.get_one::<DlFormat>("format").unwrap();
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
