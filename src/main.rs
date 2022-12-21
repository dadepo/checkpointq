use clap::Parser;
use eth_checkpoint_lib::args::{Cli, Network};
use eth_checkpoint_lib::client::{CheckpointClient, EndpointsConfig};
use eth_checkpoint_lib::client::{StateId, StateId::Slot};
use eth_checkpoint_lib::errors::AppError;
use eth_checkpoint_lib::processor::display_result;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = Cli::parse();
    let client = reqwest::Client::new();

    // get values needed from the cli
    // 1. get the network
    let network = match input.network.unwrap_or(Network::Mainnet) {
        Network::Mainnet => "mainnet",
        Network::Goerli => "goerli",
        Network::Sepolia => "sepolia",
    };

    // 2. Get the endpoints
    let endpoints_path = input.endpoints.unwrap_or("endpoints.yaml".to_string());
    let endpoints_config: EndpointsConfig =
        serde_yaml::from_reader(std::fs::File::open(endpoints_path)?)?;

    let endpoints: &Vec<String> = endpoints_config
        .endpoints
        .get(&network.to_string())
        .ok_or(AppError::NoEndpointsFound("Endpoint not found".to_string()))?;

    if endpoints.len() < 3 {
        Err(AppError::EndpointsBelowThreshold(
            "Endpoints must be greater than 3".to_string(),
        ))?
    }

    // 3. get the state id to get the checkpoint from. If none is given use the finalized
    let state_id: StateId = if input.slot == "finalized" {
        StateId::Finalized
    } else {
        Slot(input.slot.parse::<u128>()?)
    };

    // 3. get the response display level
    let display_level = input.display;

    let checkpoint_client = CheckpointClient::new(client, state_id, endpoints.to_vec());
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    display_result(result, display_level);
    Ok(())
}
