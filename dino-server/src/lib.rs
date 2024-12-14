mod config;
mod error;
mod router;
use dashmap::DashMap;
pub use error::*;
use std::collections::HashMap;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::request::Parts,
    response::IntoResponse,
    routing::any,
    Json, Router,
};
pub use config::*;
use indexmap::IndexMap;
pub use router::*;
use tokio::net::TcpListener;
pub type ProjectRouters = IndexMap<String, Vec<ProjectRoute>>;
#[derive(Clone)]
pub struct AppState {
    //key is host name
    router: DashMap<String, SwappableAppRouter>,
}
pub async fn start_server(port: u16, router: DashMap<String, SwappableAppRouter>) -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = format!("0.0.0.0:{}", port);
    info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;

    let app = Router::new()
        .route("/*path", any(handler))
        .with_state(AppState::new(router));
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

//only support json request and return json response
async fn handler(
    State(state): State<AppState>,
    parts: Parts,
    Host(host): Host,
    Query(query): Query<serde_json::Value>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    // let host = parts
    //     .uri
    //     .host()
    //     .ok_or(AppError::HostNotFound("".to_string()))?;
    let host = host
        .split(":")
        .next()
        .ok_or_else(|| AppError::HostNotFound("".to_string()))?;
    info!("host: {}", host);
    info!("parts: {:?}", parts);
    let router = state
        .router
        .get(host)
        .ok_or(AppError::HostNotFound(host.to_string()))?
        .load();
    let path = parts.uri.path();
    let method = parts.method;

    let matched = router.match_it(method, path)?;
    let handler = matched.value;
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let body = if let Some(body) = body {
        serde_json::from_slice(&body)?
    } else {
        serde_json::Value::Null
    };
    Ok(Json(serde_json::json!({
        "handler":handler,
        "params":params,
        "query":query,
        "body":body,
    })))
}

impl AppState {
    pub fn new(router: DashMap<String, SwappableAppRouter>) -> Self {
        Self { router }
    }
}
