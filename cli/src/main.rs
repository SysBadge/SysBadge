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
    Pkdl {
        #[clap()]
        id: String,

        /// Output format.
        #[clap(long, short, value_parser, default_value = "uf2")]
        format: PkDlFormat,

        #[clap(long, value_parser, default_value = "270467072")]
        offset: u32,

        /// Output file '-' for stdout
        #[clap(long, short, value_parser)]
        output: Option<Output>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum PkDlFormat {
    UF2,
    Bin,
}

impl Display for PkDlFormat {
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
        Some(Commands::Pkdl {
            id,
            format,
            offset,
            output,
        }) => {
            let mut client = sysbadge::system::Updater::new();
            client.client.user_agent = "SysBadge CLI".to_string();

            let mut system = client.get(id).await.unwrap();
            system.sort_members();

            let mut output = match output {
                Some(output) => output.clone(),
                None => Output::new(&format!("{}.{}", system.name, format)).unwrap(),
            };

            let data = match format {
                PkDlFormat::UF2 => system.get_uf2(*offset),
                PkDlFormat::Bin => system.get_bin(),
            };

            output.write_all(&data).unwrap();
        }
        _ => todo!(),
    }
}
