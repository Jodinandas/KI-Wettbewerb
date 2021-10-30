use std::sync::{Arc, Weak, Mutex};
use std::{ptr, vec};

use super::traversible::Traversible;
use super::movable::RandCar;
use crate::traits::NodeTrait;

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

#[derive(Debug, Clone)]
pub enum Node {
    Street(Street),
    IONode(IONode),
    Crossing(Crossing)
}

impl NodeTrait for Node {
    fn is_connected(&self, other: &Arc<Mutex<Node>>) -> bool {
        match self {
            Node::Street(street) => {
                street.end.iter().find(
                | n |
                    ptr::eq(n.as_ptr(), &**other)
                ).is_some()
            },
            Node::IONode(io_node) => {
                io_node.connections.iter().find(
                | n |
                    ptr::eq(n.as_ptr(), &**other)
                ).is_some()
            },
            Node::Crossing(crossing) => {
                crossing.connections.iter().find(
                | n |
                    ptr::eq(n.as_ptr(), &**other)
                ).is_some()
            }
        }
    }

    fn connect(&mut self, other: &Arc<Mutex<Node>>) {
        match self {
            Node::Street(street) => {
                street.end = vec!(Arc::downgrade(other))
            },
            Node::IONode(io_node) => {
                io_node.connections.push(
                    Arc::downgrade(other)
                )
            },
            Node::Crossing(crossing) => {
                crossing.connections.push(
                    Arc::downgrade(other)
                )
            }
        }
    }
    
    fn get_connections<'a>(&'a self) -> &'a Vec<Weak<Mutex<Node>>> {
        match self {
            Node::Street(street) => &street.end,
            Node::IONode(io_node) => &io_node.connections,
            Node::Crossing(crossing) => &crossing.connections
        }
    }
    
    fn update_cars(&mut self, t: f64) -> Vec<RandCar> {
        match self {
            Node::Street(street) => street.car_lane.update_movables(t),
            Node::IONode(io_node) => {
                // create new car
                io_node.time_since_last_spawn += t;
                let mut new_cars = Vec::<RandCar>::new();
                if io_node.time_since_last_spawn >= io_node.spawn_rate {
                    new_cars.push(
                        RandCar::new()
                    )
                }
                new_cars
            },
            Node::Crossing(crossing) => crossing.car_lane.update_movables(t),
        }
        
    }
    
    fn add_car(&mut self, car: RandCar) {
        match self {
            Node::Street(street) => street.car_lane.add(car),
            Node::IONode(io_node) => {io_node.absorbed_cars += 1}
            Node::Crossing(crossing) => crossing.car_lane.add(car),
        }
    }
    
    fn generate_graphics_info(&self) -> graphics::Info {
        match self {
            Node::Street(street) => 
                graphics::Info::Street(
                    graphics::StreetInfo {
                        lanes: street.lanes
                    }
                ),
            Node::IONode(io_node) => 
                graphics::Info::IONode(
                    graphics::IONodeInfo {
                        
                    }
                ),
            Node::Crossing(crossing) => 
                graphics::Info::Crossing(
                    graphics::CrossingInfo {
                       // Not much to display at the moment 
                    }
                )
        }
    }
    fn id(&self) -> usize {
        match self {
            Node::Street(inner) => inner.id,
            Node::IONode(inner) => inner.id,
            Node::Crossing(inner) => inner.id,
        }
    }
}

/// A simple crossing
#[derive(Debug)]
#[derive(Clone)]
pub struct Crossing {
    pub connections: Vec<Weak<Mutex<Node>>>,
    pub car_lane: Traversible<RandCar>,
    pub id: usize
}
impl Crossing {
    pub fn new() -> Crossing {
        Crossing {
            connections: Vec::new(),
            car_lane: Traversible::<RandCar>::new(100.0),
            id: 0
        }
    }
}
/// A Node that represents either the start of the simulation or the end of it
/// 
/// One of its responsibilities is to add cars and passengers to the simulation
#[derive(Debug)]
#[derive(Clone)]
pub struct IONode{
    pub connections: Vec<Weak<Mutex<Node>>>,
    pub spawn_rate: f64,
    pub time_since_last_spawn: f64,
    pub absorbed_cars: usize,
    pub id: usize
}
impl IONode{
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            spawn_rate: 1.0,
            time_since_last_spawn: 0.0,
            absorbed_cars: 0,
            id: 0
        }
    }
}

/// A `Street` is mostly used to connect `IONode`s or `Crossing`s
/// 
/// # Fields
/// - `lanes` stores how many lanes the `Street` has
#[derive(Debug)]
#[derive(Clone)]
pub struct Street{
    pub end: Vec<Weak<Mutex<Node>>>,
    pub lanes: u8,
    pub car_lane: Traversible<RandCar>,
    pub id: usize
} 

impl Street {
    pub fn new(end: &Arc<Mutex<Node>>) -> Street{
        Street {
            end: vec!(Arc::downgrade(end)),
            lanes: 1,
            car_lane: Traversible::<RandCar>::new(100.0),
            id: 0
        }
    }
}