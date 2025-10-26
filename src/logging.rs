// src/logging.rs
use std::fs;
use tracing::Dispatch;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter};

/// Build a per-node tracing Dispatch + keep a guard alive for the non-blocking writer.
pub fn build_node_dispatch(node_id: u16) -> (Dispatch, WorkerGuard) {
    let _ = fs::create_dir_all("logs");

    let file_appender =
        RollingFileAppender::new(Rotation::NEVER, "logs", format!("node-{}.log", node_id));
    let (nb_writer, guard) = tracing_appender::non_blocking(file_appender);

    let subscriber = fmt()
        .with_writer(nb_writer)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    (Dispatch::new(subscriber), guard)
}