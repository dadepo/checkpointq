extern crate core;

use std::collections::HashMap;
use std::convert::Infallible;
use futures::StreamExt;
use reqwest::{Error, Response};
use eth_checkpoint_lib::*;
use eth_checkpoint_lib::client::{CheckpointClient, HttpClient, StateId, FinalityCheckpointPayload, Data, BlockInfo};
use async_trait::async_trait;
use eth_checkpoint_lib::errors;
use serde::Serialize;
use eth_checkpoint_lib::errors::AppError;
use eth_checkpoint_lib::processor::process_to_displayable_format;

type Req = String;
type BlockRootRes = String;
type ErrorRes = String;


struct MockClient {
    base: FinalityCheckpointPayload,
    paths: Vec<(Req, Result<BlockRootRes, ErrorRes>)>
}
impl MockClient {
    pub fn new(paths: Vec<(Req, Result<BlockRootRes, ErrorRes>)>) -> Self {
        Self {
            base: FinalityCheckpointPayload {
                data: Data {
                    finalized: BlockInfo { epoch: "".to_string(), root: "".to_string() },
                    current_justified: BlockInfo { epoch: "".to_string(), root: "".to_string() },
                    previous_justified: BlockInfo { epoch: "".to_string(), root: "".to_string() }
                }
            },
            paths
        }
    }
}


#[async_trait]
impl HttpClient for MockClient {
    async fn send_request(&self, path: String) -> Result<Response, AppError> {

        let responses:
            (Vec<(Req, Result<BlockRootRes, ErrorRes>)>,
             Vec<(Req, Result<BlockRootRes, ErrorRes>)>) = self.paths.clone().into_iter()
            .partition(|(req, res)| res.is_ok());

        let success_responses: Vec<Result<BlockRootRes, ErrorRes>> = responses.0.into_iter().filter(|(req, res)|{
            path.contains(req)
        }).map(|(_, res)| res).collect();

        let err_responses: Vec<Result<BlockRootRes, ErrorRes>> = responses.1.into_iter().filter(|(req, res)|{
            path.contains(req)
        }).map(|(_, res)| res).collect();

        let mut payload = FinalityCheckpointPayload {
            ..Clone::clone(&self.base)
        };
        if !success_responses.is_empty() {
            payload.data.finalized.root = success_responses.into_iter().nth(0).unwrap().ok().unwrap().to_string();
            Ok(Response::from(http::response::Response::new(serde_json::to_string(&payload).unwrap())))
        } else {
            if !err_responses.is_empty() {
                Err(AppError::AppError(err_responses.into_iter().nth(0).unwrap().err().unwrap().to_string()))
            } else {
                Err(AppError::AppError("mock error".to_string()))
            }
        }
    }
}

#[tokio::test]
pub async fn test_single_result() {
    let client = MockClient::new(
        vec![
            ("http://www.good1.com".to_string(), Ok("Hash".to_string())),
            ("http://www.good2.com".to_string(), Ok("Hash1".to_string())),
            ("http://www.bad.com".to_string(), Err("error".to_string()))]
    );
    let endpoints = vec![
        "http://www.good1.com".to_string(),
        "http://www.good2.com".to_string(),
        "http://www.bad.com".to_string()
    ];

    let checkpoint_client = CheckpointClient::new(client, StateId::Finalized, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    dbg!(result);
}