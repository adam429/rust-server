use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::prelude::*;
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};
use tracing_subscriber::EnvFilter;

struct LocalTimer;

const fn east8() -> Option<chrono::FixedOffset> {
    chrono::FixedOffset::east_opt(8 * 3600)
}

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now().with_timezone(&east8().unwrap());
        write!(w, "{}", now.format("%FT%T%.3f"))
    }
}

pub fn init() {
    let info_file = rolling::daily("log", "info");
    let err_file = rolling::daily("log", "error").with_max_level(Level::ERROR);
    let all_files = info_file.and(err_file);

    let log_level = match dotenvy::var("RUST_LOG")
        .unwrap_or_else(|_| "info".into()).as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "error" => Level::ERROR,
        _ => Level::INFO
    };
    let env_layer = EnvFilter::from_default_env().add_directive(log_level.into());
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .pretty()
        .with_timer(LocalTimer);
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(all_files)
        .with_target(true)
        .with_line_number(true)
        .with_timer(LocalTimer)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(env_layer)
        .with(stdout_layer)
        .with(file_layer)
        .init();
}