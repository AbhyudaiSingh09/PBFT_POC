
use std::fs;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, EnvFilter};

pub struct LogGuards {
    #[allow(dead_code)]
    pub nb_guard: tracing_appender::non_blocking::WorkerGuard,
    #[allow(dead_code)]
    pub sub_guard: tracing::dispatcher::DefaultGuard,
}

pub fn init_node_file_logger(node_id: u16) -> LogGuards {
    let _ = fs::create_dir_all("logs");
    let file_appender: RollingFileAppender = RollingFileAppender::new(
        Rotation::NEVER, "logs", format!("node-{}.log", node_id));
    let (nb_writer, nb_guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = fmt().with_writer(nb_writer).with_env_filter(EnvFilter::from_default_env()).finish();
    let sub_guard = tracing::subscriber::set_default(subscriber);
    LogGuards { nb_guard, sub_guard }
}
