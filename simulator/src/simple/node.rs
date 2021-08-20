use std::borrow::Borrow;
use std::vec;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use super::super::traits::NodeTrait;
use enum_dispatch::enum_dispatch;
use super::traversible::Traversible;
use super::movable::{RandCar, RandPerson};


/// This enum represents all types of simulation data types
///
/// the connections are not saved as references, but rather as
/// indices in the list of all parts of the simulation, to avoid
/// the overhead (and tremendous complexity and annoyance of using
/// these types of e.g. a nested ```Weak<RefCell<Node>>```
///
/// The consequence of this way of organizing the data is that
/// things like moving cars from one ```Node``` to another has
/// to be done by the simulator and not by functions implemented
/// in the Node.
#[enum_dispatch(NodeTrait)]
#[derive(Debug)]
pub enum Node {
    Crossing,
    IONode,
    Street
}

impl Node {
    /// Returns the name of the variant
    ///
    /// primarily for use in displaying. Not very efficient
    pub fn name(&self) -> String {
        match self {
            Node::Crossing(_i) => "Crossing".to_owned(),
            Node::IONode(_i) => "IONode".to_owned(),
            Node::Street(_i) => "Street".to_owned()
        }
    }
}

/// A simple crossing
#[derive(Debug)]
pub struct Crossing {
    connections: Vec<usize>,
    car_lane: Traversible<RandCar>
}
impl Crossing {
    pub fn new() -> Crossing {
        Crossing {
            connections: vec![],
            car_lane: Traversible::<RandCar>::new(100.0)
        }
    }
}
impl NodeTrait for Crossing {
    fn is_connected(&self, other: usize) -> bool {
        self.connections.contains(&other)
    }

    fn connect(&mut self, other: usize) {
        self.connections.push(other)
    }
    fn get_connections(&self) -> Vec<usize> {
        self.connections.clone()
    }
    
    fn update_cars(&mut self, t: f64) -> Vec<RandCar> {
        self.car_lane.update_movables(t)
    }
    
    fn add_car(&mut self, car: RandCar) {
        self.car_lane.add(car)
    }
}
/// A Node that represents either the start of the simulation or the end of it
/// 
/// One of its responsibilities is to add cars and passengers to the simulation
#[derive(Debug)]
pub struct IONode{
    connections: Vec<usize>,
    spawn_rate: f64,
    time_since_last_spawn: f64
}
impl IONode{
    pub fn new() -> IONode {
        IONode {
            connections: vec![],
            spawn_rate: 1.0,
            time_since_last_spawn: 0.0
        }
    }
    // Spawn rate in cars / second
    pub fn spawn_rate(&mut self, rate: f64) -> &mut IONode {
        self.spawn_rate = rate;
        self
    }
}
impl NodeTrait for IONode {
    fn is_connected(&self, other: usize) -> bool {
        self.connections.contains(&other)
    }

    fn connect(&mut self, other: usize) {
        self.connections.push(other)
    }
    fn get_connections(&self) -> Vec<usize> {
        self.connections.clone()
    }
    /// Spawn cars
    fn update_cars(&mut self, dt: f64) -> Vec<RandCar> {
        self.time_since_last_spawn += dt;
        let mut new_cars = Vec::<RandCar>::new();
        if self.time_since_last_spawn >= self.spawn_rate {
            new_cars.push(
                RandCar::new()
            )
        }
        new_cars
    }
    fn add_car(&mut self, car: RandCar) {
        drop(car)
    }
}

/// A `Street` is mostly used to connect `IONode`s or `Crossing`s
/// 
/// # Fields
/// - `lanes` stores how many lanes the `Street` has
#[derive(Debug)]
pub struct Street{
    pub connection: Option<usize>,
    pub lanes: u8,
    pub car_lane: Traversible<RandCar>
} 
impl Street {
    pub fn new() -> Street{
        Street {
            connection: None,
            lanes: 1,
            car_lane: Traversible::<RandCar>::new(100.0)
        }
    }
}
impl NodeTrait for Street {
    fn is_connected(&self, other: usize) -> bool {
        match self.connection {
            Some(c) => c == other,
            None => false
        }
    }

    fn connect(&mut self, other: usize) {
        self.connection = Some(other)
    }
    fn get_connections(&self) -> Vec<usize> {
        match self.connection {
            Some(c) => vec![c],
            None => vec![]
        }
    }

    fn update_cars(&mut self, t: f64) -> Vec<RandCar> {
        self.car_lane.update_movables(t)
    }
    fn add_car(&mut self, car: RandCar) {
        self.car_lane.add(car);
    }
}