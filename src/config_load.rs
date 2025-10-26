use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig { pub id: u16, pub host: String, pub port: u16 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig { pub nodes: Vec<NodeConfig> }

pub fn load_cluster(path_no_ext: &str) -> Result<ClusterConfig> {
    let settings = config::Config::builder()
        .add_source(config::File::with_name(path_no_ext))
        .build()?;
    let cluster: ClusterConfig = settings.try_deserialize()?;
    Ok(cluster)
}