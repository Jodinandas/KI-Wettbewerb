use simulator::SimulatorBuilder;
use std::thread;

pub struct SimManager {
    pub sim_builder: SimulatorBuilder,
    simulation_threads: Vec<thread::Thread>,
}
