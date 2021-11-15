use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, Weak};
use std::{ptr, vec};

use super::movable::RandCar;
use super::node_builder::{CrossingConnections, Direction, InOut};
use super::traversible::Traversible;
use crate::traits::NodeTrait;
use crate::simple::pathfinding::PathAwareCar;

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
        pub lanes: u8,
    }
    pub enum Info {
        Crossing(CrossingInfo),
        IONode(IONodeInfo),
        Street(StreetInfo),
    }
}

#[derive(Debug, Clone)]
pub enum Node {
    Street(Street),
    IONode(IONode),
    Crossing(Crossing),
}

impl NodeTrait for Node {
    fn is_connected(&self, other: &Arc<Mutex<Node>>) -> bool {
        self
            .get_connections()
            .iter()
            .find( |n| ptr::eq(n.as_ptr(), &**other))
            .is_some()
    }
    fn get_connections(&self) -> Vec<Weak<Mutex<Node>>> {
        match self {
            Node::Street(street) => street.get_connections(),
            Node::IONode(io_node) => io_node.connections.clone(),
            Node::Crossing(crossing) => crossing.get_connections(),
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
                    new_cars.push(RandCar::new())
                }
                new_cars
            }
            Node::Crossing(crossing) => crossing.car_lane.update_movables(t),
        }
    }

    fn add_car(&mut self, car: RandCar) {
        match self {
            Node::Street(street) => street.car_lane.add(car),
            Node::IONode(io_node) => io_node.absorbed_cars += 1,
            Node::Crossing(crossing) => crossing.car_lane.add(car),
        }
    }

    fn generate_graphics_info(&self) -> graphics::Info {
        match self {
            Node::Street(street) => graphics::Info::Street(graphics::StreetInfo {
                lanes: street.lanes,
            }),
            Node::IONode(io_node) => graphics::Info::IONode(graphics::IONodeInfo {}),
            Node::Crossing(crossing) => graphics::Info::Crossing(graphics::CrossingInfo {
                       // Not much to display at the moment 
                    }),
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
#[derive(Debug, Clone)]
pub struct Crossing {
    pub connections: CrossingConnections<Node>,
    pub car_lane: Traversible<RandCar>,
    pub id: usize,
}
impl Crossing {
    pub fn new() -> Crossing {
        Crossing {
            connections: CrossingConnections::new(),
            car_lane: Traversible::<PathAwareCar>::new(1.0),
            id: 0,
        }
    }
    pub fn get_connections(&self) -> Vec<std::sync::Weak<Mutex<Node>>> {
        self.connections
            .output
            .values()
            .map(|c| Weak::clone(c))
            .collect()
    }
    pub fn connect(
        &mut self,
        dir: Direction,
        conn_type: InOut,
        other: &Arc<Mutex<Node>>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        self.connections.add(dir, conn_type, other)?;
        Ok(self)
    }
}
/// A Node that represents either the start of the simulation or the end of it
///
/// One of its responsibilities is to add cars and passengers to the simulation
#[derive(Debug, Clone)]
pub struct IONode {
    pub connections: Vec<Weak<Mutex<Node>>>,
    pub spawn_rate: f64,
    pub time_since_last_spawn: f64,
    pub absorbed_cars: usize,
    pub id: usize,
}
impl IONode {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            spawn_rate: 1.0,
            time_since_last_spawn: 0.0,
            absorbed_cars: 0,
            id: 0,
        }
    }
    pub fn connect(&mut self, n: &Arc<Mutex<Node>>) {
        self.connections.push(Arc::downgrade(n))
    }
}

/// A `Street` is mostly used to connect `IONode`s or `Crossing`s
///
/// # Fields
/// - `lanes` stores how many lanes the `Street` has
#[derive(Debug, Clone)]
pub struct Street {
    pub conn_out: Option<Weak<Mutex<Node>>>,
    pub conn_in: Option<Weak<Mutex<Node>>>,
    pub lanes: u8,
    pub car_lane: Traversible<RandCar>,
    pub id: usize,
}

impl Street {
    pub fn new(end: &Arc<Mutex<Node>>) -> Street {
        Street {
            conn_out: None,
            conn_in: None,
            lanes: 1,
            id: 0,
            car_lane: Traversible::<PathAwareCar>::new(100.0)
        }
    }
    pub fn connect(&mut self, conn_type: InOut, other: &Arc<Mutex<Node>>) -> &mut Self {
        let new_item = Some(Arc::downgrade(other));
        match conn_type {
            InOut::IN => self.conn_in = new_item,
            InOut::OUT => self.conn_out = new_item,
        }
        self
    }
    pub fn get_connections<'a>(&'a self) -> Vec<std::sync::Weak<Mutex<Node>>> {
        let mut out = Vec::new();
        if let Some(conn) = &self.conn_out {
            out.push(Weak::clone(conn));
        }
        out
    }
}
