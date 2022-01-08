use editor_rs::run;
use tracing::{info, error, level_filters::STATIC_MAX_LEVEL};
use tracing_core::LevelFilter;
use tracing_subscriber::{self, EnvFilter, layer::Filter};


pub fn main() {
    // NOTE: The logger expects an environment variable called RUST_LOG
    //  for logging the frontend, use RUST_LOG="editor_rs=<level>" (where <level> is either "trace", "debug", "info", "warn" or "error")
    //      (be careful, the name editor-rs is tranformed to editor_rs)
    //  for logging the backend, use RUST_LOG="simulator=<level>"
    //  for both: RUST_LOG="editor_rs=<level>,simulator=<level>"
    // std::env::set_var("RUST_LOG", "error");
    // tracing_subscriber::fmt().init();
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::OFF)
    
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or("simulator=warn,editor=warn".to_string())
        )
    .init();
    info!("Initalized logger");
    run()
}
