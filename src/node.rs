// use std::sync::Arc;
use std::net::SocketAddr;
use axum::{routing::{get, post}, Router};
use tracing::{error, info};

use crate::config_load::NodeConfig;
use crate::logging::init_node_file_logger;
use crate::types::{PbftMsg};

#[derive(Clone)]
pub struct AppState {
    pub node_id: u16,
    pub peers: Vec<NodeConfig>,
    pub client: reqwest::Client,
}

pub async fn start_node(node: NodeConfig, peers: Vec<NodeConfig>) {
    // per-node file logger
    let _guards = init_node_file_logger(node.id);

    let addr: SocketAddr = format!("{}:{}", node.host, node.port)
        .parse()
        .unwrap_or_else(|_| panic!("bad addr for node {}", node.id));

    // let state = Arc::new(AppState { node_id: node.id, peers, client: reqwest::Client::new() });
    let state = AppState { node_id: node.id, peers, client: reqwest::Client::new() };

    let app = Router::new()
        .route("/health", get(crate::routes::health))
        .route("/peers", get(crate::routes::peers))
        .route("/msg", post(crate::routes::handle_pbft_msg))
        .route("/broadcast", post(crate::routes::broadcast_pbft))
        // .with_state(state.clone());
         .with_state(state.clone());

    info!(%addr, node_id=%node.id, "server starting");
    let listener = match tokio::net::TcpListener::bind(addr).await { Ok(l) => l, Err(e) => { error!(node_id=%node.id, %addr, error=?e, "bind failed"); return; } };

    if let Err(e) = axum::serve(listener, app).await {
        error!(node_id=%node.id, error=?e, "server error");
    }
}

pub async fn broadcast_pbft_inner(state: &AppState, msg: &PbftMsg) -> Result<(usize, usize), reqwest::Error> {
    let mut attempts = 0usize; let mut ok = 0usize;
    for peer in &state.peers {
        if peer.id == state.node_id { continue; }
        attempts += 1;
        if post_pbft(&state.client, peer, msg).await.is_ok() { ok += 1; }
    }
    Ok((attempts, ok))
}

async fn post_pbft(client: &reqwest::Client, dst: &crate::config_load::NodeConfig, msg: &PbftMsg) -> Result<(), reqwest::Error> {
    let url = format!("http://{}:{}/msg", dst.host, dst.port);
    client.post(url).json(msg).send().await?.error_for_status()?;
    Ok(())
}
