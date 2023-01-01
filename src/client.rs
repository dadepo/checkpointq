use crate::args::Network;
use crate::errors::AppError;
use crate::processor::process_to_displayable_format;
use async_trait::async_trait;
use futures::future::join_all;

use reqwest::Response;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessPayload {
    pub payload: FinalityCheckpointPayload,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FailurePayload {
    pub payload: AppError,
    pub endpoint: String,
}

#[derive(Debug)]
pub struct GroupedResult {
    pub success: HashMap<String, Vec<SuccessPayload>>,
    pub failure: Vec<FailurePayload>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayableResult {
    pub canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub non_canonical: Option<HashMap<String, Vec<SuccessPayload>>>,
    pub failure: Vec<FailurePayload>,
}

#[derive(Debug, Clone)]
pub struct CheckpointClient<C: HttpClient> {
    client: C,
    endpoints: Vec<String>,
    state_id: StateId,
}

#[derive(Debug, Clone)]
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
                    self.state_id
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
