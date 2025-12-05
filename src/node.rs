// src/node.rs
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use tracing::{error, info};
use tracing::dispatcher::{self, DefaultGuard};

use std::collections::{HashMap, HashSet};
use tokio::sync::Mutex;

use crate::config_load::NodeConfig;
use crate::logging::build_node_dispatch;
use crate::types::{PbftMsg, BlockId};

#[derive(Debug)]
pub struct AppState {
    pub node_id: u16,
    pub peers: Vec<NodeConfig>,
    pub client: reqwest::Client,
    pub is_byzantine: bool,
    // PBFT consensus state (mutable via async locks)
    pub height: Mutex<u64>,
    pub current_bid: Mutex<Option<BlockId>>,
    pub prev_hash: Mutex<String>,
    pub prepares: Mutex<HashMap<u64, HashMap<String, HashSet<u16>>>>,
    pub commits:  Mutex<HashMap<u64, HashMap<String, HashSet<u16>>>>,
}

pub type SharedState = Arc<AppState>;

pub async fn start_node(node: NodeConfig, peers: Vec<NodeConfig>) {
    // Build per-node dispatch + guard
    let (dispatch, file_guard) = build_node_dispatch(node.id);

    // Install this dispatch as the default for the duration of this async fn
    let _dispatch_guard: DefaultGuard = dispatcher::set_default(&dispatch);
    let _file_guard = file_guard; // keep the non-blocking writer alive

    let addr: SocketAddr = format!("{}:{}", node.host, node.port)
        .parse()
        .unwrap_or_else(|_| panic!("bad addr for node {}", node.id));

    let state: SharedState = Arc::new(AppState {
        node_id: node.id,
        peers,
        client: reqwest::Client::new(),
        is_byzantine: node.byzantine,
        height: Mutex::new(1),
        current_bid: Mutex::new(None),
        prev_hash: Mutex::new("genesis".into()),
        prepares: Mutex::new(HashMap::new()),
        commits:  Mutex::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/health", get(crate::routes::health))
        .route("/peers", get(crate::routes::peers))
        .route("/msg", post(crate::routes::handle_pbft_msg))
        .route("/broadcast", post(crate::routes::broadcast_pbft))
        .with_state(state.clone());

    info!(%addr, node_id=%node.id, "server starting");
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            error!(node_id=%node.id, %addr, error=?e, "bind failed");
            return;
        }
    };

    if let Err(e) = axum::serve(listener, app).await {
        error!(node_id=%node.id, error=?e, "server error");
    }
}

pub async fn broadcast_pbft_inner(
    state: &AppState,
    msg: &PbftMsg,
) -> Result<(usize, usize), reqwest::Error> {
    let mut attempts = 0usize;
    let mut ok = 0usize;
    for peer in &state.peers {
        // we now include self; HashSet<u16> prevents double-count
        attempts += 1;
        if post_pbft(&state.client, peer, msg).await.is_ok() {
            ok += 1;
        }
    }
    Ok((attempts, ok))
}

async fn post_pbft(
    client: &reqwest::Client,
    dst: &crate::config_load::NodeConfig,
    msg: &PbftMsg,
) -> Result<(), reqwest::Error> {
    let url = format!("http://{}:{}/msg", dst.host, dst.port);
    client.post(url).json(msg).send().await?.error_for_status()?;
    Ok(())
}

// ---------- AppState helpers (same as before, async locks) ----------

impl AppState {
    pub fn quorum_size(&self) -> usize {
        let n = self.peers.len();
        let f = (n.saturating_sub(1)) / 3;
        2 * f + 1
    }

    pub async fn note_prepare(&self, bid: &BlockId, from: u16) -> usize {
        let mut prepares = self.prepares.lock().await;
        let hmap = prepares.entry(bid.height).or_default();
        let voters = hmap.entry(bid.hash.clone()).or_default();
        voters.insert(from);
        voters.len()
    }

    pub async fn note_commit(&self, bid: &BlockId, from: u16) -> usize {
        let mut commits = self.commits.lock().await;
        let hmap = commits.entry(bid.height).or_default();
        let voters = hmap.entry(bid.hash.clone()).or_default();
        voters.insert(from);
        voters.len()
    }

    pub async fn set_candidate(&self, bid: BlockId) {
        let mut cur = self.current_bid.lock().await;
        *cur = Some(bid);
    }

    pub async fn finalize_if_current(&self, bid: &BlockId) -> bool {
        let mut current = self.current_bid.lock().await;
        if matches!(&*current, Some(cur) if cur.height == bid.height && cur.hash == bid.hash) {
            {
                let mut ph = self.prev_hash.lock().await;
                *ph = bid.hash.clone();
            }
            {
                let mut h = self.height.lock().await;
                *h += 1;
            }
            *current = None;
            return true;
        }
        false
    }
}