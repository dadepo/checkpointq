use crate::args::{Cli, Network};
use clap::Parser;
use crate::client::Client;

mod args;
mod client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = Cli::parse();
    let client = reqwest::Client::new();

    // Validations
    // TODO move to clap?
    if input.network.is_some() && !input.endpoints.is_empty() {
        Err("Either set network or endpoints but not both")?
    }

    // get values needed from the cli

    // 1. Get the endpoints
    let endpoints: Vec<String> = if let Some(network) = input.network {
        Client::default_network_endpoints(network)
    } else {
        input.endpoints
    };


    // 2. get the slot to get the checkpoint from. If none is given use the head slot
    let slot: u128 = if input.slot == "head" {
        Client::get_head_slot(client, endpoints).await?
    } else {
        input.slot.parse::<u128>().unwrap()
    };

    Ok(())
}
