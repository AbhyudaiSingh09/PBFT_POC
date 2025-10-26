
use axum::{extract::State, http::StatusCode, Json};
use tracing::{error, info};

use crate::types::{Ack, PbftMsg, BlockId, BroadcastReport};
use crate::node::AppState;

pub async fn health(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status":"ok","node_id": state.node_id}))
}

pub async fn peers(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"self_id": state.node_id, "peers": state.peers}))
}

pub async fn handle_pbft_msg(
    State(state): State<AppState>,
    Json(msg): Json<PbftMsg>,
) -> Result<(StatusCode, Json<Ack>), (StatusCode, String)> {
    match &msg {
        PbftMsg::Proposal { height, proposer, block } => {
            info!(node_id=%state.node_id, %height, %proposer, block=%block, "recv Proposal");
            let bid = BlockId { height: *height, hash: format!("h{height}-p{proposer}") };
            if *proposer != state.node_id {
                let prep = PbftMsg::Prepare { from: state.node_id, bid: bid.clone() };
                if let Err(e) = crate::node::broadcast_pbft_inner(&state, &prep).await {
                    error!(?e, "prepare broadcast failed");
                }
            }
        }
        PbftMsg::Prepare { from, bid } => {
            info!(node_id=%state.node_id, from=%from, height=%bid.height, hash=%bid.hash, "recv Prepare");
        }
        PbftMsg::Commit { from, bid } => {
            info!(node_id=%state.node_id, from=%from, height=%bid.height, hash=%bid.hash, "recv Commit");
        }
    }
    Ok((StatusCode::OK, Json(Ack { ok: true, node_id: state.node_id })))
}

pub async fn broadcast_pbft(
    State(state): State<AppState>,
    Json(msg): Json<PbftMsg>,
) -> Result<(StatusCode, Json<BroadcastReport>), (StatusCode, String)> {
    match crate::node::broadcast_pbft_inner(&state, &msg).await {
        Ok((attempted, succeeded)) => Ok((StatusCode::OK, Json(BroadcastReport { sender_id: state.node_id, attempted, succeeded }))),
        Err(e) => Err((StatusCode::BAD_GATEWAY, format!("broadcast failed: {e}")))
    }
}
