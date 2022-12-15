use clap::{Parser, ValueEnum};

/**
TODO
1. Allow exe -e url1 url2
2. make endpoint compulsory only if network is not specified.
3. move configuration of default to file - also include passing endpoints as file
*/

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub struct Cli {
    #[arg(short, long, value_enum)]
    pub network: Option<Network>,
    #[arg(short, long)]
    pub endpoints: Vec<String>,
    #[arg(short, long,  default_value_t = String::from("finalized"), help = "provide the slot number or finalized")]
    pub slot: String
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Network {
    Mainnet,
    Goerli,
    Sepolia,
}

pub enum DisplayLevel {
    Normal,
    Verbose
}