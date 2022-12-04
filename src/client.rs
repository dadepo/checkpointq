use crate::args::Network;
use reqwest::header::CONTENT_TYPE;
use serde::{Serialize, Deserialize};
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use futures::future::select_all;
use futures::FutureExt;

const DEFAULT_MAINNET: [&'static str; 8] = [
    "https://checkpointz.pietjepuk.net",
    "https://mainnet-checkpoint-sync.stakely.io",
    "https://beaconstate.ethstaker.cc",
    "https://beaconstate.info",
    "https://mainnet.checkpoint.sigp.io",
    "https://sync-mainnet.beaconcha.in",
    "https://sync.invis.tools",
    "https://mainnet-checkpoint-sync.attestant.io",
];

const DEFAULT_GOERLI: [&'static str; 6] = [
    "https://sync-goerli.beaconcha.in",
    "https://goerli.beaconstate.info",
    "https://prater-checkpoint-sync.stakely.io",
    "https://goerli.beaconstate.ethstaker.cc",
    "https://goerli-sync.invis.tools",
    "https://goerli.checkpoint-sync.ethdevops.io"
];

const DEFAULT_SEPOLIA: [&'static str; 2] = [
    "https://sepolia.beaconstate.info",
    "https://sepolia.checkpoint-sync.ethdevops.io",
];

pub struct CheckpointRes {
    network: Network,
    endpoints: Vec<String>,
    checkpoint_root: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncingRes {
    head_slot: String,
    sync_distance: String,
    is_syncing: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncGetResponse {
    data: SyncResponse
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncingResGetResponse {
    data: SyncingRes
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncResponse {
    head_slot: String,
    sync_distance: String,
    is_syncing: bool
}

#[derive(Debug)]
pub struct Client {
    network: Network,
    endpoints: Vec<String>
}

impl Client {
    pub fn new(network: Network, endpoints: Vec<String>) -> Self {
        if endpoints.is_empty() {
            Self {
                network,
                endpoints: Self::default_network_endpoints(network)
            }
        } else {
            Self {
                network,
                endpoints
            }
        }
    }
    pub fn default_network_endpoints(network: Network) -> Vec<String> {
        match network {
            Network::Mainnet => {
                return DEFAULT_MAINNET.iter().map(|s| s.to_string()).collect();
            },
            Network::Goerli => {
                return DEFAULT_GOERLI.iter().map(|s| s.to_string()).collect();
            },
            Network::Sepolia => {
                return DEFAULT_SEPOLIA.iter().map(|s| s.to_string()).collect();
            }
        }
    }
    pub async fn get_head_slot(client: reqwest::Client, endpoints: Vec<String>) -> Result<u128, Box<dyn std::error::Error>> {
        // TODO Add retry mechanism
        let mut futures = FuturesUnordered::new();

        endpoints.into_iter().for_each(|endpoint| {
            futures.push(client.get(format!("{}{}", endpoint, "/eth/v1/node/syncing")).send());
        });

        let (item_resolved, _, _) =
            select_all(futures).await;

        let head_slot = item_resolved?.json::<SyncingResGetResponse>().await?.data.head_slot.parse::<u128>()?;
        Ok(head_slot)
    }
}