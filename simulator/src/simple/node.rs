use std::error::Error;
use std::sync::{Arc, Mutex, Weak};
use std::ptr;

use super::movable::RandCar;
use super::node_builder::{CrossingConnections, Direction, InOut};
use super::traversible::Traversible;
use crate::simple::pathfinding::PathAwareCar;
use crate::traits::NodeTrait;

/// Objects for storing data relevant for rendering
/// different nodes
///
/// # Performance
/// Performance is in this case not as important, as only one Simulation
/// at a time will be displayed.
pub mod graphics {
    /// Info for Crossing
    pub struct CrossingInfo;
    /// Info for IONode
    pub struct IONodeInfo;
    /// Info for Street
    pub struct StreetInfo {
        /// info for the number of lanes
        pub lanes: u8,
    }
    /// Contains graphics Info mor different Node Types
    pub enum Info {
        /// Wrapper
        Crossing(CrossingInfo),
        /// Wrapper
        IONode(IONodeInfo),
        /// Wrapper
        Street(StreetInfo),
    }
}

/// A node is any kind of logical object in the Simulation
///  ([Streets](Street), [IONodes](IONode), [Crossings](Crossing))
///
/// # Examples
/// ## How to create a node
/// Nodes are typically created by a [NodeBuilder](super::node_builder::NodeBuilder) objects using
/// the build method.
/// ```
///
/// ```
#[derive(Debug, Clone)]
pub enum Node {
    /// Wrapper
    Street(Street),
    /// Wrapper
    IONode(IONode),
    /// Wrapper
    Crossing(Crossing),
}

impl NodeTrait for Node {
    fn is_connected(&self, other: &Arc<Mutex<Node>>) -> bool {
        self.get_connections()
            .iter()
            .find(|n| ptr::eq(n.as_ptr(), &**other))
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
            Node::IONode(_io_node) => graphics::Info::IONode(graphics::IONodeInfo {}),
            Node::Crossing(_crossing) => graphics::Info::Crossing(graphics::CrossingInfo {
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
    /// The other nodes the Crossing is connected to
    /// 
    /// A crossing is a rectangle and each of the 4 sides
    /// can have one input and one output connection
    pub connections: CrossingConnections<Node>,
    /// cars are stored in this field
    pub car_lane: Traversible<RandCar>,
    /// a number to differentiate different nodes
    pub id: usize,
}
impl Crossing {
    /// Returns a new Crossing with no connections and id=0
    pub fn new() -> Crossing {
        Crossing {
            connections: CrossingConnections::new(),
            car_lane: Traversible::<PathAwareCar>::new(1.0),
            id: 0,
        }
    }
    /// Returns a list of only OUTPUT connecitons
    /// 
    /// This function is deprecated and will be removed soon
    pub fn get_connections(&self) -> Vec<std::sync::Weak<Mutex<Node>>> {
        self.connections
            .output
            .values()
            .map(|c| Weak::clone(c))
            .collect()
    }
    /// Tries to add a connections at the specified position and raises
    /// an error if this is not possible
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
    /// All the nodes where cars should be redirected
    pub connections: Vec<Weak<Mutex<Node>>>,
    /// new Cars/Second
    pub spawn_rate: f64,
    /// time since last spawn in seconds
    pub time_since_last_spawn: f64,
    /// Tracks how many cars have reached their destination in this node
    pub absorbed_cars: usize,
    /// To differentiate different nodes. Should be set to the positions in the
    /// list of all nodes in the simulation
    pub id: usize,
}
impl IONode {
    /// Returns a new IONode
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            spawn_rate: 1.0,
            time_since_last_spawn: 0.0,
            absorbed_cars: 0,
            id: 0,
        }
    }
    /// adds a connections
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
    /// The connection leading to the node at the end of the road
    pub conn_out: Option<Weak<Mutex<Node>>>,
    /// The node where the road starts at
    pub conn_in: Option<Weak<Mutex<Node>>>,
    /// The number of lanes the road has. This can later be used
    /// to calculate the throughput
    pub lanes: u8,
    /// This field handles the actual logic of moving cars etc.
    pub car_lane: Traversible<RandCar>,
    /// The index in the simulation
    pub id: usize,
}

impl Street {
    /// Create new street
    pub fn new(_end: &Arc<Mutex<Node>>) -> Street {
        Street {
            conn_out: None,
            conn_in: None,
            lanes: 1,
            id: 0,
            car_lane: Traversible::<PathAwareCar>::new(100.0),
        }
    }
    /// Connects a node at the specifed position. If a node is already
    /// connected at the position, it is simply overwritten
    /// FIXME: Raise an error if there is already a connection, or unconnect
    ///     the node the street is connected to as well
    pub fn connect(&mut self, conn_type: InOut, other: &Arc<Mutex<Node>>) -> &mut Self {
        let new_item = Some(Arc::downgrade(other));
        match conn_type {
            InOut::IN => self.conn_in = new_item,
            InOut::OUT => self.conn_out = new_item,
        }
        self
    }
    /// Returns the out connection in a Vec of length 1 (or 0 if there is none)
    pub fn get_connections<'a>(&'a self) -> Vec<std::sync::Weak<Mutex<Node>>> {
        let mut out = Vec::new();
        if let Some(conn) = &self.conn_out {
            out.push(Weak::clone(conn));
        }
        out
    }
}
