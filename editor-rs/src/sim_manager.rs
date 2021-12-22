use simulator::SimulatorBuilder;
use simulator::datastructs::IntMut;
use simulator::path::{MovableServer, PathAwareCar};
use std::thread::{self, JoinHandle};
use std::sync::mpsc;

struct CarUpdate {
    sim_index: usize,
    position: f32,
}

/// saves a handle to the thread performing the simulation
/// and provides ways of communication
struct Simulating {
    sim: JoinHandle<()>,
    pub car_updates: mpsc::Receiver<Vec<CarUpdate>>,
    pub terminated: IntMut<bool>,
    pub report_updates: IntMut<bool>
}

/// This struct saves a list of currently simulating Simulators
/// It also provides the ability to get car updates one of the currently
/// simulating Simulations
pub struct SimManager {
    movable_server: IntMut<MovableServer>,
    pub sim_builder: SimulatorBuilder, // <PathAwareCar>, TODO: Finally implement generics in the simulator struct
    simulation_threads: Vec<thread::Thread>
}

impl SimManager {
    pub fn 

    
}