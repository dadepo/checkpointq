use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
/**
TODO
1. Allow exe -e url1 url2
2. make endpoint compulsory only if network is not specified.
3. move configuration of default to file - also include passing endpoints as file
*/

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[arg(long, value_enum)]
    pub network: Option<Network>,
    #[arg(
        short,
        long,
        help = "path to config file where endpoints for network are listed. default is ./endpoint.yaml"
    )]
    pub endpoints: Option<String>,
    #[arg(short = 'i', long,  default_value_t = String::from("finalized"), help = "provide the slot number or finalized")]
    pub state_id: String,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "display verbose result or not"
    )]
    pub verbose: bool,
    #[arg(
        short,
        long,
        default_value_t = false,
        help = "start an HTTP server for the checkpoint data"
    )]
    pub server: bool,
    #[arg(short, long, default_value_t = 7070, help = "port for HTTP server")]
    pub port: u16,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Network {
    Mainnet,
    Goerli,
    Sepolia,
}
