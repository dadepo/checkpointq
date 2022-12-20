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
                Err(AppError::GenericError(err_responses.into_iter().nth(0).unwrap().err().unwrap().to_string()))
            } else {
                Err(AppError::GenericError("mock error".to_string()))
            }
        }
    }
}

#[tokio::test]
pub async fn test_only_canonical_results() {
    // Test case where only canonical results are returned
    let expected_block_root = "Hash1";
    let first_mock = ("http://www.good1.com".to_string(), Ok(expected_block_root.to_string()));
    let second_mock = ("http://www.good2.com".to_string(), Ok(expected_block_root.to_string()));
    let third_mock = ("http://www.good3.com".to_string(), Ok(expected_block_root.to_string()));
    let client = MockClient::new(
        vec![
            first_mock.clone(),
            second_mock.clone(),
            third_mock.clone()
        ]
    );
    let endpoints = vec![
        first_mock.0.clone(),
        second_mock.0.clone(),
        third_mock.0.clone()
    ];

    let checkpoint_client = CheckpointClient::new(client, StateId::Finalized, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    // assert only canonical results are returned
    assert!(result.non_canonical.is_none());
    assert_eq!(result.failure.len(), 0);
    // assert the correct hash is returned
    let canonical_result = result.canonical.unwrap();
    let success_payload = canonical_result.get(&first_mock.1.clone().unwrap()).unwrap();
    assert_eq!(success_payload.len(), 3);
    assert_eq!(&success_payload.get(0).unwrap().payload.data.finalized.root, &expected_block_root.to_string());
    assert_eq!(&success_payload.get(1).unwrap().payload.data.finalized.root, &expected_block_root.to_string());
    assert_eq!(&success_payload.get(2).unwrap().payload.data.finalized.root, &expected_block_root.to_string());
}