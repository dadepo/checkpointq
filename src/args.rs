use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[command(flatten)]
    pub shared: SharedCommands,
    #[arg(long, value_enum)]
    pub network: Option<Network>,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "display verbose result or not"
    )]
    pub verbose: bool,
    #[command(subcommand)]
    pub subcommands: Option<SubCommands>,
}

#[derive(Subcommand)]
pub enum SubCommands {
    #[command(about = "Run in server mode", name = "server")]
    ServerCliCommands(ServerCommands),
}

#[derive(Args)]
pub struct ServerCommands {
    #[command(flatten)]
    pub shared: SharedCommands,
    #[arg(short, long, default_value_t = 7070, help = "port for HTTP server")]
    pub port: u16,
}

#[derive(Args)]
pub struct SharedCommands {
    #[arg(
        short,
        long,
        help = "path to config file where endpoints for network are listed. default is ./endpoint.yaml"
    )]
    pub endpoints: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Deserialize, Serialize)]
pub enum Network {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "goerli")]
    Goerli,
    #[serde(rename = "sepolia")]
    Sepolia,
}

impl Display for Network {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Mainnet => write!(f, "mainnet"),
            Network::Goerli => write!(f, "goerli"),
            Network::Sepolia => write!(f, "sepolia"),
        }
    }
}
