mod config_load;
mod logging;
mod types;
mod node;
mod routes;


use crate::config_load::ClusterConfig;
use tracing_subscriber::{fmt, EnvFilter};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Optional global logger for early boot
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // Load cluster config
    let cluster: ClusterConfig = config_load::load_cluster("config/cluster")?;
    assert!(!cluster.nodes.is_empty(), "no nodes configured");

    // Spawn one HTTP server per configured node
    let mut handles = Vec::new();
    for node_cfg in cluster.nodes.clone() {
        let peers = cluster.nodes.clone();
        handles.push(tokio::spawn(async move { node::start_node(node_cfg, peers).await }));
    }

    info!("all nodes spawned");
    for h in handles { let _ = h.await; }
    Ok(())
}

