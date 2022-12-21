use crate::client::CheckpointClient;
use axum::{http::StatusCode, response::IntoResponse, Json, Router};
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct CheckPointServer {
    checkpoint_client: CheckpointClient<reqwest::Client>,
    port: u16,
}

impl CheckPointServer {
    pub fn new(checkpoint_client: CheckpointClient<reqwest::Client>, port: u16) -> Self {
        Self {
            checkpoint_client,
            port,
        }
    }

    pub async fn serve(self) -> () {
        let port = self.port;
        let app = Router::new().route("/finalized", axum::routing::get(move || self.finalized()));

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    }

    async fn finalized(self) -> impl IntoResponse {
        let result = self.checkpoint_client.fetch_finality_checkpoints().await;
        (StatusCode::OK, Json(result))
    }
}
