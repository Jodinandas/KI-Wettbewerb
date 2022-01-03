use crate::int_mut::{IntMut, WeakIntMut};
use crate::node::Node;
use dyn_clone::DynClone;
use std::error::Error;
use std::fmt::Debug;
use std::sync::MutexGuard;

use crate::movable::{MovableStatus, RandCar};

/// This is a trait defining all functionality a Node needs
///
///
/// All Node variants must implement this trait
/// The nodes are mostly used in the form of `Box<dyn Node>`
pub trait NodeTrait<Car = RandCar>: Debug + Sync + Send + DynClone
where
    Car: Movable,
{
    /// returns true, if the given node is connected
    fn is_connected(&self, other: &IntMut<Node<Car>>) -> bool;
    /// advances the car position and returns references to all cars at the end
    fn update_cars(&mut self, t: f64) -> Vec<&Car>;
    /// returns a list of all the other nodes connected to the node
    fn get_out_connections(&self) -> Vec<WeakIntMut<Node<Car>>>;
    /// adds a new car to the beginning of the node
    fn add_car(&mut self, car: Car);
    /// a unique node id
    ///
    /// (the id stored in the SimulationBuilder at the beginning)
    /// is used to simplify the path algorithm used to generate
    /// paths for cars
    fn id(&self) -> usize;
    /// returns a vector of [MovableStatus] structs containing information
    /// on cars
    fn get_car_status(&self) -> Vec<MovableStatus>;
    /// removes a car by reference
    fn rm_car_by_ref(&mut self, car: &Car) -> Car;
}

// make it possible to derive Clone for structs with Box<dyn NodeTrait>
dyn_clone::clone_trait_object!(NodeTrait);

/// This trait represents some kind of movable
///
/// idea for movables:
///  use the delta t when updating to weigh a chance of
/// come action taking place internally. Example: Going into a shop
/// for 10 min or maybe someone tripping
pub trait Movable: Debug + Clone + Send + Sync + DynClone {
    /// unused
    fn get_speed(&self) -> f32;
    /// unused
    fn set_speed(&mut self, s: f32);
    /// returns new Movable
    fn new() -> Self;
    /// advances the time
    fn update(&mut self, t: f64);
    /// sets the path. (Only used in PathAwareCar)
    fn set_path(&mut self, P: Vec<usize>) {}
    /// Decides the next node for the movable to move to
    ///
    /// It can very well happen that the next node can't be determined
    /// if the part of the program that figures out the paths makes a mistake
    fn decide_next(
        &self,
        connections: &Vec<WeakIntMut<Node<Self>>>,
        current_node: &MutexGuard<Node<Self>>,
    ) -> Result<Option<WeakIntMut<Node<Self>>>, Box<dyn Error>>;
    /// Returns a unique indentifier
    fn get_id(&self) -> u32;
    // sets the internal id
    fn set_id(&mut self, id: u32);
    /// is called when the movable advances to the next node
    fn advance(&mut self) {}
}

// make it possible to derive Clone for structs with Box<dyn Movable>
// dyn_clone::clone_trait_object!(Movable);
