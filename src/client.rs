use crate::errors::AppError;
use crate::processor::{process_to_displayable_format, DisplayableResult};
use async_trait::async_trait;
use futures::future::join_all;

use reqwest::Response;

use crate::args::Network;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(Debug)]
pub struct ResponsePayloadWithEndpointInfo {
    pub payload: Result<SuccessEndpointPayload, AppError>,
    pub endpoint: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SuccessEndpointPayload {
    pub data: Data,
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

#[derive(Debug, Clone)]
pub struct CheckpointClient<C: HttpClient> {
    client: C,
    endpoints_config: EndpointsConfig,
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

#[derive(Debug, Deserialize, Clone)]
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
            .map_err(|e| AppError::EndpointResponseError(e.to_string()))
    }
}

impl<C: HttpClient> CheckpointClient<C> {
    pub fn new(client: C, state_id: StateId, endpoints: EndpointsConfig) -> Self {
        Self {
            client,
            endpoints_config: endpoints,
            state_id,
        }
    }
    pub async fn fetch_finality_checkpoints(
        &self,
        network: Network,
    ) -> Result<DisplayableResult, AppError> {
        let endpoints_config = &self.endpoints_config;
        let endpoints: &Vec<String> = endpoints_config.endpoints.get(&network.to_string()).ok_or(
            AppError::EndpointsNotFound(format!("Endpoint not found for {network}")),
        )?;

        let results = join_all(endpoints.iter().map(|endpoint| async {
            let raw_response = async {
                let path = format!(
                    "{}/eth/v1/beacon/states/{}/finality_checkpoints",
                    endpoint.clone(),
                    self.state_id
                );
                let result = self.client.send_request(path);
                match result.await {
                    Ok(res) => {
                        if res.status().is_success() {
                            ResponsePayloadWithEndpointInfo {
                                payload: res
                                    .json::<SuccessEndpointPayload>()
                                    .await
                                    .map_err(|e| AppError::EndpointResponseError(e.to_string())),
                                endpoint: endpoint.clone(),
                            }
                        } else {
                            ResponsePayloadWithEndpointInfo {
                                payload: Err(AppError::EndpointResponseError(format!(
                                    "Error with calling {} status code {}",
                                    endpoint.clone(),
                                    res.status().to_string()
                                ))),
                                endpoint: endpoint.clone(),
                            }
                        }
                    }
                    Err(e) => ResponsePayloadWithEndpointInfo {
                        payload: Err(e),
                        endpoint: endpoint.clone(),
                    },
                }
            }
            .await;
            raw_response
        }))
        .await;

        Ok(process_to_displayable_format(results))
    }
}
