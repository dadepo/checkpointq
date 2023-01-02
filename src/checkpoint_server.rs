use crate::client::CheckpointClient;
use crate::processor::DisplayableResult;
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryParams {
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    block_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    payload: Option<DisplayableResult>,
}

#[derive(Debug)]
pub struct CheckPointMiddleware {
    checkpoint_client: CheckpointClient<reqwest::Client>,
    port: u16,
}

impl CheckPointMiddleware {
    pub fn new(checkpoint_client: CheckpointClient<reqwest::Client>, port: u16) -> Self {
        Self {
            checkpoint_client,
            port,
        }
    }

    pub async fn serve(self) {
        let port = self.port;
        let app = Router::new()
            .route("/finalized", axum::routing::get(finalized))
            .with_state(Arc::new(self));

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }
}

#[debug_handler]
async fn finalized(
    State(middle_ware): State<Arc<CheckPointMiddleware>>,
    Query(query_params): Query<QueryParams>,
) -> impl IntoResponse {
    let displayable_result = middle_ware
        .checkpoint_client
        .fetch_finality_checkpoints()
        .await;

    let not_found_msg = "Finalized block root not found";
    let block_root = match displayable_result.canonical.as_ref() {
        Some(canonical) => canonical
            .keys()
            .next()
            .map(|s| s.to_string())
            .unwrap_or(not_found_msg.to_string()),
        None => not_found_msg.to_string(),
    };

    let payload = if query_params.verbose {
        Some(displayable_result)
    } else {
        None
    };

    let api_response = ApiResponse {
        block_root,
        payload,
    };

    (StatusCode::OK, Json(api_response))
}
