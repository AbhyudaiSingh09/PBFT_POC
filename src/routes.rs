// src/router.rs
use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use tracing::{error, info};

use crate::types::{Ack, PbftMsg, BlockId, BroadcastReport};
use crate::node::AppState;

pub async fn health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let h = state.height.lock().await;
    Json(serde_json::json!({ "status":"ok", "node_id": state.node_id, "height": *h }))
}

pub async fn peers(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "self_id": state.node_id, "peers": state.peers }))
}

pub async fn handle_pbft_msg(
    State(state): State<Arc<AppState>>,
    Json(msg): Json<PbftMsg>,
) -> Result<(StatusCode, Json<Ack>), (StatusCode, String)> {
    let quorum = state.quorum_size();
    let me = state.node_id;

    match msg {
        PbftMsg::Proposal { height, proposer, block: _ } => {
            let bid = BlockId { height, hash: format!("h{height}-p{proposer}") };
            info!(node_id=%me, %height, %proposer, bid=%bid.hash, "recv Proposal");
            state.set_candidate(bid.clone()).await;

            // Record my own PREPARE vote
            let my_prepare_count = state.note_prepare(&bid, me).await;
            info!(node_id=%me, count=%my_prepare_count, need=%quorum, bid=%bid.hash, "self Prepare recorded");

            // Broadcast PREPARE if I'm not the proposer
            if proposer != me {
                let prep = PbftMsg::Prepare { from: me, bid: bid.clone() };
                if let Err(e) = crate::node::broadcast_pbft_inner(&state, &prep).await {
                    error!(?e, "prepare broadcast failed");
                }
            }
        }

        PbftMsg::Prepare { from, bid } => {
            let count = state.note_prepare(&bid, from).await;
            info!(node_id=%me, from=%from, count=%count, need=%quorum, bid=%bid.hash, "recv Prepare");
            if count >= quorum {
                // Record my own COMMIT vote
                let self_commit_count = state.note_commit(&bid, me).await;
                info!(node_id=%me, count=%self_commit_count, need=%quorum, bid=%bid.hash, "self Commit recorded");

                let commit = PbftMsg::Commit { from: me, bid: bid.clone() };
                if let Err(e) = crate::node::broadcast_pbft_inner(&state, &commit).await {
                    error!(?e, "commit broadcast failed");
                }
            }
        }

        PbftMsg::Commit { from, bid } => {
            let count = state.note_commit(&bid, from).await;
            info!(node_id=%me, from=%from, count=%count, need=%quorum, bid=%bid.hash, "recv Commit");
            if count >= quorum {
                if state.finalize_if_current(&bid).await {
                    info!(node_id=%me, height=%bid.height, hash=%bid.hash, "FINALIZED");
                }
            }
        }
    }

    Ok((StatusCode::OK, Json(Ack { ok: true, node_id: me })))
}

pub async fn broadcast_pbft(
    State(state): State<Arc<AppState>>,
    Json(msg): Json<PbftMsg>,
) -> Result<(StatusCode, Json<BroadcastReport>), (StatusCode, String)> {
    match crate::node::broadcast_pbft_inner(&state, &msg).await {
        Ok((attempted, succeeded)) => Ok((StatusCode::OK, Json(BroadcastReport {
            sender_id: state.node_id, attempted, succeeded
        }))),
        Err(e) => Err((StatusCode::BAD_GATEWAY, format!("broadcast failed: {e}")))
    }
}