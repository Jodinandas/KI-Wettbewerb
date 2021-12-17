/// wrapper for interior mutability
pub mod int_mut;
/// logic for cars and pedestrians
pub mod movable;
/// provides nodes in the simulations (crossings, streets...)
pub mod node;
/// utilizes the builder pattern to construct nodes
pub mod node_builder;
/// is responsible for calculating paths through the street network
pub mod pathfinding;
/// used for simulating a street network
pub mod simulation;
/// constructs simulations
pub mod simulation_builder;
/// provides logic to move cars and pedestrians
pub mod traversible;
