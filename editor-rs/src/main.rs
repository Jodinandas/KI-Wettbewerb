use editor_rs::run;
use log::{info, error, debug};
extern crate pretty_env_logger;

pub fn main() {
    pretty_env_logger::init();
    debug!("running bevy");
    run()
}
