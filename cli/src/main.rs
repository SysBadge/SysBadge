mod dl;
mod system;

use anyhow::{bail, Context, Result};
use clap::{command, Arg, ArgAction, Command, Parser, ValueEnum};
use tracing::*;
use tracing_subscriber::prelude::*;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::{fmt, reload, EnvFilter, Registry};

#[tokio::main]
async fn main() {
    let fmt = fmt::layer().with_target(false);

    let filter = tracing_subscriber::EnvFilter::from_default_env();
    let (filter, reload_handle) = reload::Layer::new(filter);

    tracing_subscriber::registry().with(filter).with(fmt).init();

    match run_main(reload_handle).await {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        },
    }
}

async fn run_main(reload_handle: Handle<tracing_subscriber::EnvFilter, Registry>) -> Result<()> {
    let app = clap::command!()
        .arg(
            clap::Arg::new("verbose")
                .global(true)
                .action(clap::ArgAction::Count)
                .long("verbose")
                .short('v'),
        )
        .subcommand_required(true)
        .subcommand(dl::command())
        .subcommand(system::command());

    let matches = app.get_matches();

    reload_handle
        .modify(|f| {
            let level = match matches.get_count("verbose") {
                0 => Level::WARN,
                1 => Level::INFO,
                2 => Level::DEBUG,
                3.. => Level::TRACE,
            };
            *f = EnvFilter::from_default_env().add_directive(level.into());
        })
        .context("Failed to reaload logger")?;
    debug!("Enabled debug logging");
    trace!("Enabled trace logging");

    match matches.subcommand().unwrap() {
        ("dl", matches) => dl::run(matches).await,
        ("system", matches) => system::run(matches).await,

        _ => bail!("Unknown subcommand"),
    }
}
