use eth_checkpoint_lib::args::Cli;
use clap::Parser;
use eth_checkpoint_lib::client::{CheckpointClient, default_network_endpoints};
use eth_checkpoint_lib::client::{StateId, StateId::Slot};
use eth_checkpoint_lib::errors::AppError;
use eth_checkpoint_lib::processor::display_result;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = Cli::parse();
    let client = reqwest::Client::new();

    // Validations
    // TODO move to clap?
    if input.network.is_some() && !input.endpoints.is_empty() {
        Err(AppError::NetworkAndEndpoint("Either set network or endpoints but not both".to_string()))?
    }

    // get values needed from the cli

    // 1. Get the endpoints
    let endpoints: Vec<String> = if let Some(network) = input.network {
        default_network_endpoints(network)
    } else {
        input.endpoints
    };

    if endpoints.len() < 3 {
        Err(AppError::EndpointsBelowThreshold("Endpoints must be greater than 3".to_string()))?
    }

    // 2. get the state id to get the checkpoint from. If none is given use the finalized
    let state_id: StateId = if input.slot == "finalized" {
        StateId::Finalized
    } else {
        match input.slot.parse::<u128>() {
            Ok(value) => Slot(value),
            Err(_) => StateId::Finalized
        }
    };

    // 3. get the response display level
    let display_level = input.display;

    let checkpoint_client = CheckpointClient::new(client, state_id, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    display_result(result, display_level);
    Ok(())
}
