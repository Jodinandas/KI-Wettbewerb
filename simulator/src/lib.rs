//! The backend that does all the heavy lifing.

#![warn(missing_docs)]
/// constructs a square of crossings. mostly for debugging purposes
mod build_grid;
/// the most important traits are seperated
pub mod traits;
/// put build_grid in a submodule
pub mod debug {
    pub use super::build_grid::*;
}

/// wrapper for interior mutability
mod int_mut;
/// logic for cars and pedestrians
mod movable;
/// provides nodes in the simulations (crossings, streets...)
mod node;
/// utilizes the builder pattern to construct nodes
mod node_builder;
/// is responsible for calculating paths through the street network
mod pathfinding;
/// top level struct used for managing Simulation, SimulationManager, MovableServer
mod sim_manager;
/// used for simulating a street network
mod simulation;
/// constructs simulations
mod simulation_builder;
/// provides logic to move cars and pedestrians
mod traversible;
// reexport
pub mod nodes {
    //! importing the different node types
    pub use crate::node::*;
    pub use crate::node_builder::*;
}
pub mod path {
    //! Crate for the Car and Movableserver
    pub use crate::pathfinding::{MovableServer, PathAwareCar};
}

pub use sim_manager::SimManager;

pub mod datastructs {
    //! The most important data saving structs needed
    pub use crate::int_mut::{IntMut, WeakIntMut};
    pub use crate::movable::MovableStatus;
}
pub use simulation::Simulator;
pub use simulation_builder::SimulatorBuilder;

/// TODO: Make this unique for different cars
const CAR_SPACING: f32 = 3.0;