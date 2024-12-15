mod config;
mod error;
mod jsengine;
mod middleware;
mod router;
use dashmap::DashMap;
pub use error::*;
pub use jsengine::*;
use matchit::Match;
pub use middleware::ServiceTimeLayer;
use std::collections::HashMap;
use tracing::info;

use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Host, Query, State},
    http::{request::Parts, Response},
    response::IntoResponse,
    routing::any,
    Router,
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

#[derive(Clone)]
pub struct TenentRouter {
    pub host: String,
    pub router: SwappableAppRouter,
}

pub async fn start_server(port: u16, router: Vec<TenentRouter>) -> Result<()> {
    // let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    // tracing_subscriber::registry().with(layer).init();

    let addr = format!("0.0.0.0:{}", port);
    info!("Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    let map_router = DashMap::new();
    for TenentRouter { host, router } in router {
        map_router.insert(host, router);
    }

    let app = Router::new()
        .route("/*path", any(handler))
        .layer(ServiceTimeLayer)
        .with_state(AppState::new(map_router));
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

//only support json request and return json response
async fn handler(
    State(state): State<AppState>,
    parts: Parts,
    Host(host): Host,
    Query(query): Query<HashMap<String, String>>,
    body: Option<Bytes>,
) -> Result<impl IntoResponse, AppError> {
    let router = get_router_by_host(host, state)?;
    let path = parts.uri.path();
    let method = parts.method.clone();
    let matched = router.match_it(method, path)?;
    let handler = matched.value;
    let req = get_request_parts(parts.clone(), body, matched, query)?;
    let ret = JsEngine::new(&router.code)?.run(handler, req)?;

    Ok(Response::from(ret))
}

fn get_router_by_host(host: String, state: AppState) -> Result<AppRouter, AppError> {
    let host = host
        .split(":")
        .next()
        .ok_or_else(|| AppError::HostNotFound("".to_string()))?;
    let router = state
        .router
        .get(host)
        .ok_or(AppError::HostNotFound(host.to_string()))?
        .load();
    Ok(router)
}
fn get_request_parts(
    parts: Parts,
    body: Option<Bytes>,
    matched: Match<&str>,
    query: HashMap<String, String>,
) -> Result<Req, AppError> {
    let headers = parts
        .headers
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let params: HashMap<String, String> = matched
        .params
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let body = body.and_then(|b| {
        let b = b.to_vec();
        if b.is_empty() {
            None
        } else {
            String::from_utf8(b).ok()
        }
    });
    let req = Req::builder()
        .method(parts.method.to_string())
        .url(parts.uri.to_string())
        .headers(headers)
        .body(body)
        .query(query)
        .params(params)
        .build();
    Ok(req)
}

impl AppState {
    pub fn new(router: DashMap<String, SwappableAppRouter>) -> Self {
        Self { router }
    }
}

impl TenentRouter {
    pub fn new(host: String, router: SwappableAppRouter) -> Self {
        Self { host, router }
    }
}
