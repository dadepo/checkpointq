use crate::client::CheckpointClient;
use axum::{http::StatusCode, response::IntoResponse, Json, Router, extract::State};
use std::{net::SocketAddr, sync::Arc};

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

async fn finalized(State(middle_ware): State<Arc<CheckPointMiddleware>>) -> impl IntoResponse {
    let result = middle_ware.checkpoint_client.fetch_finality_checkpoints().await;
    (StatusCode::OK, Json(result))
}