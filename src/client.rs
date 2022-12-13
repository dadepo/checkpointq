use crate::args::Network;
use reqwest::header::CONTENT_TYPE;
use serde::{Serialize, Deserialize};
use futures::prelude::*;
use futures::stream::FuturesUnordered;
use futures::future::{select_all, FutureExt, join_all};

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

// `Req/Res
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
struct SyncingResGetResponse {
    data: SyncingRes
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockInfo {
    pub epoch: String,
    pub root: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Data {
    pub finalized: BlockInfo,
    pub current_justified: BlockInfo,
    pub previous_justified: BlockInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FinalityCheckpointResp {
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlFinalityCheckpointResp {
    pub data: Data,
    pub url: String
}

pub type Finality_Results = Vec<(Result<FinalityCheckpointResp, reqwest::Error>, String)>;

#[derive(Debug)]
pub struct CheckpointClient {
    client: reqwest::Client,
    endpoints: Vec<String>,
    stateId: StateId
}

#[derive(Debug)]
pub enum StateId {
    Finalized,
    Slot(u128) // TODO is u128 to big?
}

impl CheckpointClient {
    pub fn new(client: reqwest::Client, stateId: StateId, endpoints: Vec<String>) -> Self {
        Self {
            client,
            endpoints,
            stateId
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
    pub async fn fetch_finality_checkpoints(&self) -> Finality_Results {
        let endpoints = &self.endpoints;
        join_all(endpoints.iter().map(|endpoint| async {
            let raw_response = async {
                let result = self.client.get(format!("{}{}", endpoint.clone(), "/eth/v1/beacon/states/finalized/finality_checkpoints")).send();
                let result = match result.await {
                    // TODO Possible not to use match?
                    // Catch error before parsing to json so that original error message is used upstream
                    Ok(res) => res.json::<FinalityCheckpointResp>().await,
                    Err(e) => Err(e)
                };
                (result, endpoint.clone())
            }.await;
            raw_response
        })).await
    }
}