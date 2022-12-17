use std::collections::HashMap;
use std::convert::Infallible;
use reqwest::{Error, Response};
use eth_checkpoint_lib::*;
use eth_checkpoint_lib::client::{CheckpointClient, HttpClient, StateId, FinalityCheckpointPayload, Data, BlockInfo};
use async_trait::async_trait;
use serde::Serialize;
use eth_checkpoint_lib::processor::{group_success_failure, to_displayable_result};

type Req = String;
type BlockRootRes = String;


struct MockClient {
    base: FinalityCheckpointPayload,
    paths: Vec<(Req, BlockRootRes)>
}
impl MockClient {
    pub fn new(paths: Vec<(Req, BlockRootRes)>) -> Self {
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
    async fn send_request(&self, path: String) -> Result<Response, Error> {
        let (_, block_hash_response) = self.paths.iter().filter(|(rq, _)| {
            path.contains(rq)
        }).next().unwrap();

        let mut payload = FinalityCheckpointPayload {
            ..Clone::clone(&self.base)
        };
        payload.data.finalized.root = block_hash_response.to_string();
        Ok(Response::from(http::response::Response::new(serde_json::to_string(&payload).unwrap())))
    }
}

#[tokio::test]
pub async fn test_single_result() {
    let client = MockClient::new(
        vec![
            ("http://www.good1.com".to_string(), "Hash".to_string()),
            ("http://www.good2.com".to_string(), "Hash".to_string()),
            ("http://www.bad.com".to_string(), "error".to_string())]
    );
    let endpoints = vec![
        "http://www.good1.com".to_string(),
        "http://www.good2.com".to_string(),
        "http://www.bad.com".to_string()
    ];

    let checkpoint_client = CheckpointClient::new(client, StateId::Finalized, endpoints);
    let result = checkpoint_client.fetch_finality_checkpoints().await;
    let displayable_result = to_displayable_result(group_success_failure(result));
    dbg!(displayable_result);
}