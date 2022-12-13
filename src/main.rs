use crate::args::{Cli, Network};
use clap::Parser;
use crate::client::CheckpointClient;
use crate::client::StateId;
use crate::processor::group_by_root_hash;
use crate::StateId::Slot;

mod args;
mod client;
mod processor;

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
        CheckpointClient::default_network_endpoints(network)
    } else {
        input.endpoints
    };


    // 2. get the state id to get the checkpoint from. If none is given use the finalized
    let stateId: StateId = if input.slot == "finalized" {
        StateId::Finalized
    } else {
        match input.slot.parse::<u128>() {
            Ok(value) => Slot(value),
            Err(_) => StateId::Finalized
        }
    };

    let checkpoint_client = CheckpointClient::new(client, stateId, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    group_by_root_hash(result);
    Ok(())
}
