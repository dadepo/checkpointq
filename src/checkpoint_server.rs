use crate::args::Network;
use crate::client::CheckpointClient;
use crate::errors::AppError;
use crate::processor::DisplayableResult;
use axum::extract::{Path, Query};
use axum::response::Response;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json, Router};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryParams {
    #[serde(default)]
    pub verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    block_root: String,
    epoch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    payload: Option<DisplayableResult>,
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::NOT_FOUND, self.to_string()).into_response()
    }
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
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .init();

        let port = self.port;
        let app = Router::new()
            .route("/:network/finalized", axum::routing::get(finalized))
            .layer(TraceLayer::new_for_http())
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
    Path(network): Path<Network>,
    Query(query_params): Query<QueryParams>,
) -> Result<Json<ApiResponse>, AppError> {
    let displayable_result = middle_ware
        .checkpoint_client
        .fetch_finality_checkpoints(network)
        .await?;

    let block_not_found_msg = "Finalized block root not found";
    let epoch_not_found_msg = "Epoch not found";
    let (block_root, epoch) = match displayable_result.canonical.as_ref() {
        Some(canonical) => {
            let block_root = canonical
                .keys()
                .next()
                .map(|s| s.to_string())
                .unwrap_or(block_not_found_msg.to_string());
            let epoch = canonical
                .get(&block_root)
                .and_then(|success_payloads| success_payloads.iter().next())
                .map(|success_payload| success_payload.payload.data.finalized.epoch.to_string())
                .unwrap_or(epoch_not_found_msg.to_string());
            (block_root, epoch)
        }
        None => (
            block_not_found_msg.to_string(),
            epoch_not_found_msg.to_string(),
        ),
    };

    let payload = if query_params.verbose {
        Some(displayable_result)
    } else {
        None
    };

    let api_response = ApiResponse {
        block_root,
        epoch,
        payload,
    };

    Ok(Json(api_response))
}
