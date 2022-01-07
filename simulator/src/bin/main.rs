use simulator::{path::PathAwareCar, SimulatorBuilder};

fn main() {
    // NOTE: The logger expects an environment variable called RUST_LOG
    //  for logging the frontend, use RUST_LOG="editor_rs=<level>" (where <level> is either "trace", "debug", "info", "warn" or "error")
    //      (be careful, the name editor-rs is tranformed to editor_rs)
    //  for logging the backend, use RUST_LOG="simulator=<level>"
    //  for both: RUST_LOG="editor_rs=<level>,simulator=<level>"
    // pretty_env_logger::init();
    // tracing_subscriber::fmt::init();
    let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 15]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
    let mut _data = SimulatorBuilder::<PathAwareCar>::from_json(json).unwrap();
}
