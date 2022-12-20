use crate::args::Network;
use crate::errors::AppError;
use crate::processor::process_to_displayable_format;
use async_trait::async_trait;
use futures::future::{join_all};

use reqwest::{Error, Response};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};

impl From<reqwest::Error> for AppError {
    fn from(value: Error) -> Self {
        AppError::GenericError(value.to_string())
    }
}

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
    "https://goerli.checkpoint-sync.ethdevops.io",
];
const DEFAULT_SEPOLIA: [&'static str; 2] = [
    "https://sepolia.beaconstate.info",
    "https://sepolia.checkpoint-sync.ethdevops.io",
];

pub fn default_network_endpoints(network: Network) -> Vec<String> {
    match network {
        Network::Mainnet => DEFAULT_MAINNET.iter().map(|s| s.to_string()).collect(),
        Network::Goerli => DEFAULT_GOERLI.iter().map(|s| s.to_string()).collect(),
        Network::Sepolia => DEFAULT_SEPOLIA.iter().map(|s| s.to_string()).collect(),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyncingRes {
    head_slot: String,
    sync_distance: String,
    is_syncing: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct SyncingResGetResponse {
    data: SyncingRes,
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FinalityCheckpointPayload {
    pub data: Data,
}

#[derive(Debug)]
pub struct ResponsePayload {
    pub payload: Result<FinalityCheckpointPayload, AppError>,
    pub endpoint: String,
}

#[derive(Debug)]
pub struct SuccessPayload {
    pub payload: FinalityCheckpointPayload,
    pub endpoint: String,
}

#[derive(Debug)]
pub struct FailurePayload {
    pub payload: AppError,
    pub endpoint: String,
}

#[derive(Debug)]
pub struct GroupedResult {
    pub success: HashMap<String, Vec<SuccessPayload>>,
    pub failure: Vec<FailurePayload>,
}

#[derive(Debug)]
pub struct DisplayableResult {
    pub canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub non_canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub failure: Vec<FailurePayload>,
}

#[derive(Debug)]
pub struct CheckpointClient<C: HttpClient> {
    client: C,
    endpoints: Vec<String>,
    state_id: StateId,
}

#[derive(Debug)]
pub enum StateId {
    Finalized,
    Slot(u128), // TODO is u128 to big?
}

impl fmt::Display for StateId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StateId::Finalized => write!(f, "finalized"),
            StateId::Slot(slot) => write!(f, "{:?}", slot.to_string()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct EndpointsConfig {
    pub endpoints: HashMap<String, Vec<String>>,
}

#[async_trait]
pub trait HttpClient {
    async fn send_request(&self, path: String) -> Result<Response, AppError>;
}

#[async_trait]
impl HttpClient for reqwest::Client {
    async fn send_request(&self, path: String) -> Result<Response, AppError> {
        self.get(path)
            .send()
            .await
            .map_err(|e| AppError::GenericError(e.to_string()))
    }
}

impl<C: HttpClient> CheckpointClient<C> {
    pub fn new(client: C, state_id: StateId, endpoints: Vec<String>) -> Self {
        Self {
            client,
            endpoints,
            state_id,
        }
    }
    pub async fn fetch_finality_checkpoints(&self) -> DisplayableResult {
        let endpoints = &self.endpoints;
        let results = join_all(endpoints.iter().map(|endpoint| async {
            let raw_response = async {
                let path = format!(
                    "{}/eth/v1/beacon/states/{}/finality_checkpoints",
                    endpoint.clone(),
                    self.state_id.to_string()
                );
                let result = self.client.send_request(path);
                match result.await {
                    // TODO Possible not to use match?
                    // Catch error before parsing to json so that original error message is used upstream
                    Ok(res) => ResponsePayload {
                        payload: res
                            .json::<FinalityCheckpointPayload>()
                            .await
                            .map_err(|e| AppError::GenericError(e.to_string())),
                        endpoint: endpoint.clone(),
                    },
                    Err(e) => ResponsePayload {
                        payload: Err(e),
                        endpoint: endpoint.clone(),
                    },
                }
            }
            .await;
            raw_response
        }))
        .await;

        process_to_displayable_format(results)
    }
}
