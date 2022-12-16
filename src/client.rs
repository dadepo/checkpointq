use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use crate::args::Network;
use serde::{Serialize, Deserialize};
use futures::stream::FuturesUnordered;
use futures::future::{select_all, join_all};

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
pub struct FinalityCheckpointPayload {
    pub data: Data,
}

#[derive(Debug)]
pub struct ResponsePayload {
    pub payload: Result<FinalityCheckpointPayload, reqwest::Error>,
    pub endpoint: String
}

#[derive(Debug)]
pub struct SuccessPayload {
    pub payload: FinalityCheckpointPayload,
    pub endpoint: String
}
#[derive(Debug)]
pub struct FailurePayload {
    pub payload: reqwest::Error,
    pub endpoint: String
}

#[derive(Debug)]
pub struct GroupedResult {
    pub success: HashMap<String, Vec<SuccessPayload>>,
    pub failure: Vec<FailurePayload>
}

#[derive(Debug)]
pub struct DisplayableResult {
    pub canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub non_canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub failure: Vec<FailurePayload>
}

#[derive(Debug)]
pub struct CheckpointClient {
    client: reqwest::Client,
    endpoints: Vec<String>,
    state_id: StateId
}

#[derive(Debug)]
pub enum StateId {
    Finalized,
    Slot(u128) // TODO is u128 to big?
}

impl fmt::Display for StateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StateId::Finalized => write!(f, "finalized"),
            StateId::Slot(slot) => write!(f, "{:?}", slot.to_string())
        }
    }
}

impl CheckpointClient {
    pub fn new(client: reqwest::Client, state_id: StateId, endpoints: Vec<String>) -> Self {
        Self {
            client,
            endpoints,
            state_id
        }
    }
    pub fn default_network_endpoints(network: Network) -> Vec<String> {
        match network {
            Network::Mainnet => DEFAULT_MAINNET.iter().map(|s| s.to_string()).collect(),
            Network::Goerli => DEFAULT_GOERLI.iter().map(|s| s.to_string()).collect(),
            Network::Sepolia => DEFAULT_SEPOLIA.iter().map(|s| s.to_string()).collect()
        }
    }
    pub async fn _get_head_slot(client: reqwest::Client, endpoints: Vec<String>) -> Result<u128, Box<dyn std::error::Error>> {
        // TODO Add retry mechanism
        let futures = FuturesUnordered::new();

        endpoints.into_iter().for_each(|endpoint| {
            futures.push(client.get(format!("{}{}", endpoint, "/eth/v1/node/syncing")).send());
        });

        let (item_resolved, _, _) =
            select_all(futures).await;

        let head_slot = item_resolved?.json::<SyncingResGetResponse>().await?.data.head_slot.parse::<u128>()?;
        Ok(head_slot)
    }
    pub async fn fetch_finality_checkpoints(&self) -> Vec<ResponsePayload> {
        let endpoints = &self.endpoints;
        join_all(endpoints.iter().map(|endpoint| async {
            let raw_response = async {
                let path = format!("{}/eth/v1/beacon/states/{}/finality_checkpoints", endpoint.clone(), self.state_id.to_string());
                let result = self.client.get(path).send();
                match result.await {
                    // TODO Possible not to use match?
                    // Catch error before parsing to json so that original error message is used upstream
                    Ok(res) => {
                        ResponsePayload {
                            payload: res.json::<FinalityCheckpointPayload>().await,
                            endpoint: endpoint.clone() }
                    },
                    Err(e) => {
                        ResponsePayload {
                            payload: Err(e),
                            endpoint: endpoint.clone()
                        }
                    }
                }
            }.await;
            raw_response
        })).await
    }
}