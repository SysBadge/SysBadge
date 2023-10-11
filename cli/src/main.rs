mod dl;

use anyhow::{bail, Context, Result};
use clap::{command, Arg, ArgAction, Command, Parser, ValueEnum};
use clio::Output;
use std::fmt::{Display, Formatter};
use std::io::Write;

#[tokio::main]
async fn main() {
    match run_main().await {
        Ok(()) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_main() -> Result<()> {
    let app = clap::command!()
        .subcommand_required(true)
        .subcommand(dl::command());

    let matches = app.get_matches();

    match matches.subcommand().unwrap() {
        ("dl", matches) => dl::run(matches).await,

        _ => bail!("Unknown subcommand"),
    }
}
