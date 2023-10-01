use clap::{command, Arg, ArgAction, Command, Parser, ValueEnum};
use clio::Output;
use std::fmt::{Display, Formatter};
use std::io::Write;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    Dl {
        #[clap()]
        id: String,

        #[clap(long, short, default_value = "PluralKit")]
        source: sysbadge::system::downloaders::Source,

        /// Output format.
        #[clap(long, short, value_parser, default_value = "uf2")]
        format: DlFormat,

        #[clap(long, value_parser, default_value = "270467072")]
        offset: u32,

        /// Output file '-' for stdout
        #[clap(long, short, value_parser)]
        output: Option<Output>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum DlFormat {
    UF2,
    Bin,
}

impl Display for DlFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UF2 => write!(f, "uf2"),
            Self::Bin => write!(f, "bin"),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Dl {
            id,
            source,
            format,
            offset,
            output,
        }) => {
            let mut downloader = sysbadge::system::downloaders::GenericDownloader::new();
            downloader.useragent = "SysBadge CLI".to_string();
            let mut system = downloader.get(*source, id).await.unwrap();
            system.sort_members();

            let mut output = match output {
                Some(output) => output.clone(),
                None => Output::new(&format!("{}.{}", system.name, format)).unwrap(),
            };

            let data = match format {
                DlFormat::UF2 => system.get_uf2(*offset),
                DlFormat::Bin => system.get_bin(),
            };

            output.write_all(&data).unwrap();
        }
        _ => todo!(),
    }
}
