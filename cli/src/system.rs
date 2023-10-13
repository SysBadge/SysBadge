use std::fs::File;
use std::io::Read;

use anyhow::{bail, Context, Result};
use sysbadge::system::downloaders::Source;
use sysbadge_usb::UsbSysBadge;
use tracing::*;

pub fn command() -> clap::Command {
    let cmd = clap::Command::new("system")
        .about("Update system on device")
        .arg(
            clap::Arg::new("file")
                .long("file")
                .short('f')
                .value_hint(clap::ValueHint::FilePath)
                .value_parser(clap::value_parser!(String)),
        )
        .arg(
            clap::Arg::new("dl-id")
                .long("id")
                .value_parser(clap::value_parser!(String)),
        )
        .group(
            clap::ArgGroup::new("source")
                .args(&["file", "dl-id"])
                .required(true)
                .multiple(false),
        )
        .arg(
            clap::Arg::new("erase")
                .long("erase")
                .short('e')
                .action(clap::ArgAction::SetFalse),
        )
        .subcommand(
            clap::Command::new("update")
                .about("update current system")
                .arg(
                    clap::Arg::new("erase")
                        .long("erase")
                        .short('e')
                        .action(clap::ArgAction::SetFalse),
                )
                .arg(
                    clap::Arg::new("dl-user-agent")
                        .hide(true)
                        .long("user-agent")
                        .default_value("SysBadge CLI")
                        .value_parser(clap::value_parser!(String)),
                ),
        )
        .subcommand_negates_reqs(true);

    crate::dl::dl_common_args(cmd)
}

pub async fn run(matches: &clap::ArgMatches) -> Result<()> {
    let mut badge = crate::find_badge(matches).await?;
    badge.set_timeout(std::time::Duration::from_secs(5));

    if let Some(file) = matches.get_one("file") {
        return run_file(badge, matches, file).await;
    }

    if let Some(id) = matches.get_one("dl-id") {
        return run_dl(badge, matches, id).await;
    }

    if let Some(matches) = matches.subcommand_matches("update") {
        return run_update(badge, matches).await;
    }

    bail!("No source specified")
}

async fn run_file(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    matches: &clap::ArgMatches,
    file: &String,
) -> Result<()> {
    let file = File::options().read(true).open(file)?;

    let iter = file.bytes().map(|b| b.unwrap());

    badge
        .system_update_blocking(matches.get_flag("erase"), iter)
        .context("Failed to update")?;
    Ok(())
}

async fn run_dl(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    matches: &clap::ArgMatches,
    id: &String,
) -> Result<()> {
    let system = crate::dl::download(matches, id).await?;

    let bytes = system.get_bin().into_iter();

    badge
        .system_update_blocking(matches.get_flag("erase"), bytes)
        .context("Failed to update")?;
    Ok(())
}

async fn run_update(
    badge: UsbSysBadge<sysbadge_usb::rusb::GlobalContext>,
    matches: &clap::ArgMatches,
) -> Result<()> {
    let current_id = badge.system_id()?;
    let id = current_id.id().context("Invalid id")?;
    let source = Source::try_from(*current_id.as_ref()).context("Invalid source")?;

    let mut downloader = sysbadge::system::downloaders::GenericDownloader::new();
    downloader.useragent = matches.get_one::<String>("dl-user-agent").unwrap().clone();

    let mut system = downloader.get(source, id).await.unwrap();
    system.sort_members();

    let bytes = system.get_bin().into_iter();

    badge
        .system_update_blocking(matches.get_flag("erase"), bytes)
        .context("Failed to update")?;
    Ok(())
}
