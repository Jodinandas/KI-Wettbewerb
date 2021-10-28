use std::vec;

use super::super::traits::NodeTrait;
use super::traversible::Traversible;
use super::movable::RandCar;

/// Objects for storing data relevant for rendering
/// different nodes
/// 
/// This is necessary, as functions for rendering the nodes
/// can't be implemented here, as it would require to import
/// bevy as a dependency. It is not a good to import the graphics
/// engine in the backend just to be able to use the correct
/// function signatures.
/// 
/// # Performance
/// Performance is in this case not as important, as only one Simulation
/// at a time will be displayed.
pub mod graphics {
    pub struct CrossingInfo;
    pub struct IONodeInfo;
    pub struct StreetInfo {
        pub lanes: u8 
    }
    pub enum Info {
        Crossing (CrossingInfo), 
        IONode (IONodeInfo),
        Street (StreetInfo)
    }

}



/// A simple crossing
#[derive(Debug)]
#[derive(Clone)]
pub struct Crossing {
    pub connections: Vec<usize>,
    pub car_lane: Traversible<RandCar>
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
    fn is_connected(&self, other: &usize) -> bool {
        self.connections.iter().find(|c| {
           *c == other 
        }).is_some()
    }

    fn connect(&mut self, other: &usize) {
        self.connections.push(*other)
    }
    
    fn get_connections(&self) -> &Vec<usize> {
        &self.connections
    }
    
    fn update_cars(&mut self, t: f64) -> Vec<RandCar> {
        self.car_lane.update_movables(t)
    }
    
    fn add_car(&mut self, car: RandCar) {
        self.car_lane.add(car)
    }
    
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Crossing(
            graphics::CrossingInfo {
               // Not much to display at the moment 
            }
        )
    }
    
}
/// A Node that represents either the start of the simulation or the end of it
/// 
/// One of its responsibilities is to add cars and passengers to the simulation
#[derive(Debug)]
#[derive(Clone)]
pub struct IONode{
    pub connections: Vec<usize>,
    pub spawn_rate: f64,
    pub time_since_last_spawn: f64
}
impl IONode{
    pub fn new() -> Self {
        Self {
            connections: vec![],
            spawn_rate: 1.0,
            time_since_last_spawn: 0.0
        }
    }
}
impl NodeTrait for IONode {
    fn is_connected(&self, other: &usize) -> bool {
        self.connections.iter().find(|c| {
           *c == other 
        }).is_some()
    }

    fn connect(&mut self, other: &usize) {
        self.connections.push(*other)
    }
    fn get_connections(&self) -> &Vec<usize> {
        &self.connections
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
    
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::IONode(
            graphics::IONodeInfo {
                
            }
        )
    }
}

/// A `Street` is mostly used to connect `IONode`s or `Crossing`s
/// 
/// # Fields
/// - `lanes` stores how many lanes the `Street` has
#[derive(Debug)]
#[derive(Clone)]
pub struct Street{
    pub connection: Vec<usize>,
    pub lanes: u8,
    pub car_lane: Traversible<RandCar>
} 

impl Street {
    pub fn new() -> Street{
        Street {
            connection: Vec::new(),
            lanes: 1,
            car_lane: Traversible::<RandCar>::new(100.0)
        }
    }
}
impl NodeTrait for Street {
    fn is_connected(&self, other: &usize) -> bool {
        self.connection.iter().find(|c| {
           *c == other 
        }).is_some()
    }

    fn connect(&mut self, other: &usize) {
        self.connection.clear();
        self.connection.push(*other)
    }
    fn get_connections(&self) -> &Vec<usize> {
        &self.connection
    }

    fn update_cars(&mut self, t: f64) -> Vec<RandCar> {
        self.car_lane.update_movables(t)
    }

    fn add_car(&mut self, car: RandCar) {
        self.car_lane.add(car);
    }
    
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Street(
            graphics::StreetInfo {
                lanes: self.lanes
            }
        )
    }
}