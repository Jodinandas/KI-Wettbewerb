use std::vec;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::traits::Movable;

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
    
    fn update_cars(&mut self, t: f32) -> Vec<RandCar> {
        self.car_lane.update_movables(t)
    }
}
/// A Node that represents either the start of the simulation or the end of it
/// 
/// One of its responsibilities is to add cars and passengers to the simulation
#[derive(Debug)]
pub struct IONode{
    connections: Vec<usize>,
    global_car_count: Option<Weak<RefCell<usize>>>,
    max_num_cars: Option<Weak<usize>>
}
impl IONode{
    pub fn new() -> IONode {
        IONode {
            connections: vec![],
            global_car_count: None,
            max_num_cars: None
        }
    }
    pub fn car_count(&mut self, car_count: &Rc<RefCell<usize>>) -> &mut IONode {
        self.global_car_count = Some(Rc::downgrade(car_count));
        self
    }

    pub fn max_cars(&mut self, max_cars: &Rc<usize>) -> &mut IONode {
        self.max_num_cars = Some(Rc::downgrade(max_cars));
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
    
    /// Spawn cars
    fn update_cars(&mut self, t: f32) -> Vec<RandCar> {
        // let mut max_cars = match self.max_num_cars {
        //     Some(reference) => {reference.},
        //     None => 1000
        // };
        // let mut car_count = 1000;
        Vec::new()
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

    fn update_cars(&mut self, t: f32) -> Vec<RandCar> {
        self.car_lane.update_movables(t)
    }
}