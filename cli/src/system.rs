use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};
use tracing::debug;

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
                //.required(true)
                .multiple(false),
        )
        .arg(
            clap::Arg::new("erase")
                .long("erase")
                .short('e')
                .action(clap::ArgAction::SetTrue)
                .required_unless_present("source"),
        );

    crate::dl::dl_common_args(cmd)
}

pub async fn run(matches: &clap::ArgMatches) -> Result<()> {
    let mut badge = crate::find_badge(matches).await?;
    badge.set_timeout(std::time::Duration::from_secs(5));

    /*badge
    .enter_update_system(matches.get_flag("erase"))
    .context("Failed to enter update mode")?;*/
    // TODO: check for update mode

    let vec = if let Some(vec) = if let Some(file) = matches.get_one::<String>("file") {
        Some(run_file(matches, file).await?)
    } else if let Some(id) = matches.get_one::<String>("dl-id") {
        Some(run_dl(matches, id).await?)
    } else {
        None
    } {
        vec
    } else {
        debug!("No source specified");
        return Ok(());
    };

    badge
        .system_update_blocking(matches.get_flag("erase"), vec.into_iter())
        .context("Failed to update")?;

    Ok(())
}

async fn run_file(_matches: &clap::ArgMatches, file: &String) -> Result<Vec<u8>> {
    let mut file = File::options().read(true).open(file)?;

    let mut vec = Vec::new();

    file.read_to_end(&mut vec)?;

    Ok(vec)
}

async fn run_dl(matches: &clap::ArgMatches, id: &String) -> Result<Vec<u8>> {
    todo!()
    //Ok(())
}
