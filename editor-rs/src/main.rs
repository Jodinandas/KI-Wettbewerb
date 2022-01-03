use editor_rs::run;
extern crate pretty_env_logger;

pub fn main() {
    // NOTE: The logger expects an environment variable called RUST_LOG
    //  for logging the frontend, use RUST_LOG="editor_rs=<level>" (where <level> is either "trace", "debug", "info", "warn" or "error")
    //      (be careful, the name editor-rs is tranformed to editor_rs)
    //  for logging the backend, use RUST_LOG="simulator=<level>"
    //  for both: RUST_LOG="editor_rs=<level>,simulator=<level>"
    pretty_env_logger::init();
    run()
}
