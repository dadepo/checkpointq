use crate::args::DisplayLevel;
use crate::client::{CheckpointClient, DisplayableResult};
use axum::extract::Query;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryParams {
    pub display_level: Option<DisplayLevel>,
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
    let display_level = query_params.display_level.unwrap_or(DisplayLevel::Normal);

    let block_root = match displayable_result.canonical.as_ref() {
        Some(canonical) => canonical.keys().next().unwrap().to_string(),
        None => "Finalized block root not found".to_string(),
    };

    match display_level {
        DisplayLevel::Normal => (
            StatusCode::OK,
            Json(ApiResponse {
                block_root: block_root.to_string(),
                payload: None,
            }),
        ),
        DisplayLevel::Verbose => (
            StatusCode::OK,
            Json(ApiResponse {
                block_root: block_root.to_string(),
                payload: Some(displayable_result),
            }),
        ),
    }
}
