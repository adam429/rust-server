use tracing::Level;
use tracing_appender::rolling;
use tracing_subscriber::prelude::*;
use tracing_subscriber::fmt::{format::Writer, time::FormatTime};
use tracing_subscriber::EnvFilter;

// 自定义时间格式化结构体
struct LocalTimer;

// 定义东八区时区偏移
const fn east8() -> Option<chrono::FixedOffset> {
    chrono::FixedOffset::east_opt(8 * 3600)
}

// 实现 FormatTime trait 为 LocalTimer
impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        // 获取当前 UTC 时间并转换为东八区时间
        let now = chrono::Utc::now().with_timezone(&east8().unwrap());
        // 格式化时间为 "YYYY-MM-DDTHH:MM:SS.mmm" 格式
        write!(w, "{}", now.format("%FT%T%.3f"))
    }
}

pub fn init() {
    // 设置每日滚动的日志文件
    let info_file = rolling::daily("log", "info");
    let err_file = rolling::daily("log", "error").with_max_level(Level::ERROR);
    let all_files = info_file.and(err_file);

    // 从环境变量获取日志级别，默认为 INFO
    let log_level = match dotenvy::var("RUST_LOG")
        .unwrap_or_else(|_| "info".into()).as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "error" => Level::ERROR,
        _ => Level::INFO
    };

    // 创建环境过滤层
    let env_layer = EnvFilter::from_default_env().add_directive(log_level.into());

    // 创建标准输出层
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .pretty()
        .with_timer(LocalTimer);

    // 创建文件输出层
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(all_files)
        .with_target(true)
        .with_line_number(true)
        .with_timer(LocalTimer)
        .with_ansi(false);

    // 初始化 tracing subscriber，组合所有层
    tracing_subscriber::registry()
        .with(env_layer)
        .with(stdout_layer)
        .with(file_layer)
        .init();
}