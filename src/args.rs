use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[command(flatten)]
    pub shared: SharedCommands,
    #[arg(long, short, value_enum)]
    pub network: Option<Network>,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "Display verbose result or not"
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
    #[arg(
        short,
        long,
        default_value_t = 7070,
        help = "Port for HTTP server. Defaults to 7070"
    )]
    pub port: u16,
}

#[derive(Args)]
pub struct SharedCommands {
    #[arg(
        short,
        long,
        help = "Path to config file where endpoints for network are listed. default is ./endpoint.yaml"
    )]
    pub endpoints: Option<PathBuf>,
}

#[derive(
    Display, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug, Deserialize, Serialize,
)]
#[serde(rename_all = "lowercase")]
pub enum Network {
    Mainnet,
    Goerli,
    Sepolia,
}
