extern crate core;

use async_trait::async_trait;
use eth_checkpoint_lib::client::{
    BlockInfo, CheckpointClient, Data, FinalityCheckpointPayload, HttpClient, StateId,
};

use eth_checkpoint_lib::errors::AppError;


use futures::StreamExt;
use reqwest::{Response};




type Req = String;
type BlockRootRes = String;
type ErrorRes = String;

struct MockClient {
    base: FinalityCheckpointPayload,
    paths: Vec<(Req, Result<BlockRootRes, ErrorRes>)>,
}
impl MockClient {
    pub fn new(paths: Vec<(Req, Result<BlockRootRes, ErrorRes>)>) -> Self {
        Self {
            base: FinalityCheckpointPayload {
                data: Data {
                    finalized: BlockInfo {
                        epoch: "".to_string(),
                        root: "".to_string(),
                    },
                    current_justified: BlockInfo {
                        epoch: "".to_string(),
                        root: "".to_string(),
                    },
                    previous_justified: BlockInfo {
                        epoch: "".to_string(),
                        root: "".to_string(),
                    },
                },
            },
            paths,
        }
    }
}

#[async_trait]
impl HttpClient for MockClient {
    async fn send_request(&self, path: String) -> Result<Response, AppError> {
        let responses: (
            Vec<(Req, Result<BlockRootRes, ErrorRes>)>,
            Vec<(Req, Result<BlockRootRes, ErrorRes>)>,
        ) = self
            .paths
            .clone()
            .into_iter()
            .partition(|(_req, res)| res.is_ok());

        let success_responses: Vec<Result<BlockRootRes, ErrorRes>> = responses
            .0
            .into_iter()
            .filter(|(req, _res)| path.contains(req))
            .map(|(_, res)| res)
            .collect();

        let err_responses: Vec<Result<BlockRootRes, ErrorRes>> = responses
            .1
            .into_iter()
            .filter(|(req, _res)| path.contains(req))
            .map(|(_, res)| res)
            .collect();

        let mut payload = FinalityCheckpointPayload {
            ..Clone::clone(&self.base)
        };
        if !success_responses.is_empty() {
            payload.data.finalized.root = success_responses
                .into_iter()
                .nth(0)
                .unwrap()
                .ok()
                .unwrap()
                .to_string();
            Ok(Response::from(http::response::Response::new(
                serde_json::to_string(&payload).unwrap(),
            )))
        } else {
            if !err_responses.is_empty() {
                Err(AppError::GenericError(
                    err_responses
                        .into_iter()
                        .nth(0)
                        .unwrap()
                        .err()
                        .unwrap()
                        .to_string(),
                ))
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
    let first_mock = (
        "http://www.good1.com".to_string(),
        Ok(expected_block_root.to_string()),
    );
    let second_mock = (
        "http://www.good2.com".to_string(),
        Ok(expected_block_root.to_string()),
    );
    let third_mock = (
        "http://www.good3.com".to_string(),
        Ok(expected_block_root.to_string()),
    );
    let client = MockClient::new(vec![
        first_mock.clone(),
        second_mock.clone(),
        third_mock.clone(),
    ]);
    let endpoints = vec![
        first_mock.0.clone(),
        second_mock.0.clone(),
        third_mock.0.clone(),
    ];

    let checkpoint_client = CheckpointClient::new(client, StateId::Finalized, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    // assert only canonical results are returned
    assert!(result.non_canonical.is_none());
    assert_eq!(result.failure.len(), 0);
    // assert the correct hash is returned
    let canonical_result = result.canonical.unwrap();
    let success_payload = canonical_result
        .get(&first_mock.1.clone().unwrap())
        .unwrap();
    assert_eq!(success_payload.len(), 3);
    assert_eq!(
        &success_payload.get(0).unwrap().payload.data.finalized.root,
        &expected_block_root.to_string()
    );
    assert_eq!(
        &success_payload.get(1).unwrap().payload.data.finalized.root,
        &expected_block_root.to_string()
    );
    assert_eq!(
        &success_payload.get(2).unwrap().payload.data.finalized.root,
        &expected_block_root.to_string()
    );
}

#[tokio::test]
pub async fn test_only_non_canonical_results() {
    // Test case where only non canonical results are returned
    let expected_block_root1 = "Hash1";
    let expected_block_root2 = "Hash2";
    let expected_block_root3 = "Hash3";
    let first_mock = (
        "http://www.good1.com".to_string(),
        Ok(expected_block_root1.to_string()),
    );
    let second_mock = (
        "http://www.good2.com".to_string(),
        Ok(expected_block_root2.to_string()),
    );
    let third_mock = (
        "http://www.good3.com".to_string(),
        Ok(expected_block_root3.to_string()),
    );
    let client = MockClient::new(vec![
        first_mock.clone(),
        second_mock.clone(),
        third_mock.clone(),
    ]);
    let endpoints = vec![
        first_mock.0.clone(),
        second_mock.0.clone(),
        third_mock.0.clone(),
    ];

    let checkpoint_client = CheckpointClient::new(client, StateId::Finalized, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    // assert only non canonical results are returned
    assert!(!result.non_canonical.is_none());
    assert!(result.canonical.is_none());
    assert_eq!(result.failure.len(), 0);
    // assert the correct hash is returned
    let non_canonical_result = result.non_canonical.unwrap();
    let payload1 = non_canonical_result
        .get(&first_mock.1.clone().unwrap())
        .unwrap();
    let payload2 = non_canonical_result
        .get(&second_mock.1.clone().unwrap())
        .unwrap();
    let payload3 = non_canonical_result
        .get(&third_mock.1.clone().unwrap())
        .unwrap();
    assert_eq!(
        &payload1.get(0).unwrap().payload.data.finalized.root,
        &expected_block_root1.to_string()
    );
    assert_eq!(
        &payload2.get(0).unwrap().payload.data.finalized.root,
        &expected_block_root2.to_string()
    );
    assert_eq!(
        &payload3.get(0).unwrap().payload.data.finalized.root,
        &expected_block_root3.to_string()
    );
}

#[tokio::test]
pub async fn test_only_failure_results() {
    // Test case where only failure results are returned
    let error0 = "error0";
    let error1 = "error1";
    let error2 = "error2";
    let first_mock = ("http://www.good1.com".to_string(), Err(error0.to_string()));
    let second_mock = ("http://www.good2.com".to_string(), Err(error1.to_string()));
    let third_mock = ("http://www.good3.com".to_string(), Err(error2.to_string()));
    let client = MockClient::new(vec![
        first_mock.clone(),
        second_mock.clone(),
        third_mock.clone(),
    ]);

    let endpoints = vec![
        first_mock.0.clone(),
        second_mock.0.clone(),
        third_mock.0.clone(),
    ];

    let checkpoint_client = CheckpointClient::new(client, StateId::Finalized, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    // assert only error results are returned
    assert!(result.non_canonical.is_none());
    assert!(result.canonical.is_none());
    assert_eq!(result.failure.len(), 3);
    // assert the correct failure values are returned
    let failure_result = result.failure;
    assert_eq!(
        &failure_result.get(0).unwrap().payload.to_string(),
        &error0.to_string()
    );
    assert_eq!(
        &failure_result.get(1).unwrap().payload.to_string(),
        &error1.to_string()
    );
    assert_eq!(
        &failure_result.get(2).unwrap().payload.to_string(),
        &error2.to_string()
    );
}
