use checkpointq_lib::args::{Cli, SubCommands};
use checkpointq_lib::checkpoint_server;
use checkpointq_lib::client::StateId;
use checkpointq_lib::client::{CheckpointClient, EndpointsConfig};
use std::path::PathBuf;

use checkpointq_lib::args::Network::Mainnet;
use checkpointq_lib::errors::AppError;
use checkpointq_lib::processor::print_result;
use clap::Parser;

fn parse_endpoint_config(
    endpoints_path: PathBuf,
) -> Result<EndpointsConfig, Box<dyn std::error::Error>> {
    let endpoints_config: EndpointsConfig =
        serde_yaml::from_reader(std::fs::File::open(endpoints_path)?)?;
    let above_threshold = endpoints_config
        .endpoints
        .values()
        .all(|value| value.len() >= 3);
    if above_threshold {
        Ok(endpoints_config)
    } else {
        Err(Box::new(AppError::EndpointsBelowThreshold(
            "Number of endpoints provided for networks in the config must be at least 3"
                .to_string(),
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = Cli::parse();
    let client = reqwest::Client::new();
    let state_id: StateId = StateId::Finalized; // only finalized supported for now

    match input.subcommands {
        Some(subcommand) => {
            match subcommand {
                SubCommands::ServerCliCommands(server_command) => {
                    // server run
                    let endpoints_path = server_command
                        .shared
                        .endpoints
                        .unwrap_or("endpoints.yaml".into());
                    let port = server_command.port;
                    let endpoints_config: EndpointsConfig = parse_endpoint_config(endpoints_path)?;

                    let checkpoint_client =
                        CheckpointClient::new(client, state_id, endpoints_config);
                    let server =
                        checkpoint_server::CheckPointMiddleware::new(checkpoint_client, port);
                    server.serve().await;
                }
            }
        }
        None => {
            // Normal run
            let is_verbose = input.verbose;
            let endpoints_path = input.shared.endpoints.unwrap_or("endpoints.yaml".into());
            let endpoints_config: EndpointsConfig = parse_endpoint_config(endpoints_path)?;

            let network = input.network.unwrap_or(Mainnet);
            let checkpoint_client = CheckpointClient::new(client, state_id, endpoints_config);
            let result = checkpoint_client
                .fetch_finality_checkpoints(network)
                .await?;
            print_result(result, is_verbose);
        }
    }
    Ok(())
}
