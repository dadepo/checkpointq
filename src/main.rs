use crate::args::{Cli, DisplayLevel};
use clap::Parser;
use crate::client::CheckpointClient;
use crate::client::StateId;
use crate::processor::{display_result, group_success_failure, to_displayable_result};
use crate::StateId::Slot;
use colored::*;

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
    let state_id: StateId = if input.slot == "finalized" {
        StateId::Finalized
    } else {
        match input.slot.parse::<u128>() {
            Ok(value) => Slot(value),
            Err(_) => StateId::Finalized
        }
    };

    let checkpoint_client = CheckpointClient::new(client, state_id, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    let result1 = group_success_failure(result);
    let to_display = to_displayable_result(result1);
    display_result(to_display, DisplayLevel::Normal);
    Ok(())
}
