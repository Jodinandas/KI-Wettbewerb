use std::{collections::HashMap, error::Error, fmt::Debug, hash::Hash};

use crate::node::TrafficLightState;

use super::int_mut::{IntMut, WeakIntMut};
use super::{
    movable::RandCar,
    node::{Crossing, IONode, Node, Street},
    traversible::Traversible,
};
use dyn_clone::DynClone;

/// A struct that is part of the builder pattern and constructs nodes
#[derive(Debug, Clone)]
pub enum NodeBuilder {
    /// Wraps an [IONodeBuilder]
    IONode(IONodeBuilder),
    /// Wraps a [Crossing]
    Crossing(CrossingBuilder),
    /// Wraps a [StreetBuilder]
    Street(StreetBuilder),
}

/// A Trait defining the behaviour of the subvariants of [NodeBuilder]
pub trait NodeBuilderTrait: Debug + DynClone + Sync + Send {
    /// constructs a node with the same settings
    fn build(&self) -> Node;
    /// returns a list of all connected output nodes
    fn get_out_connections(&self) -> Vec<WeakIntMut<NodeBuilder>>;
    /// returns a list of all connected nodes
    fn get_all_connections(&self) -> Vec<WeakIntMut<NodeBuilder>>;
    /// returns true if the given [NodeBuilder] is in the list of connections
    fn is_connected(&self, other: &IntMut<NodeBuilder>) -> bool;
    /// returns the weight
    ///
    /// The weight is a measure of how likely cars will got through this node
    fn get_weight(&self) -> f32;
    /// id in the global list of nodebuilders
    ///
    /// This is necessary in some parts of the code to
    /// distinguish between nodes.
    /// TODO: It might be possible to remove
    /// this usize later on.
    fn get_id(&self) -> usize;
    /// sets the id to the given number
    ///
    /// This should NOT be used manually. This method
    /// is for use in the SimulationBuilder.
    fn set_id(&mut self, id: usize);
    /// removes a connection
    ///
    /// In contrast to connect, this does not need any
    /// additional information. (connect is therefor not a part
    /// of this trait, but rather implemented individually)
    fn remove_connection(&mut self, conn: &WeakIntMut<NodeBuilder>);
}

fn has_connection(node_a: &NodeBuilder, node_b: &IntMut<NodeBuilder>) -> bool {
    node_a
        .get_out_connections()
        .iter()
        .find(|n| *n == node_b)
        .is_some()
}

impl NodeBuilderTrait for NodeBuilder {
    fn build(&self) -> Node {
        match self {
            NodeBuilder::IONode(inner) => inner.build(),
            NodeBuilder::Crossing(inner) => inner.build(),
            NodeBuilder::Street(inner) => inner.build(),
        }
    }

    fn get_out_connections(&self) -> Vec<WeakIntMut<NodeBuilder>> {
        match self {
            NodeBuilder::IONode(inner) => inner.get_out_connections(),
            NodeBuilder::Crossing(inner) => inner.get_out_connections(),
            NodeBuilder::Street(inner) => inner.get_out_connections(),
        }
    }
    fn get_all_connections(&self) -> Vec<WeakIntMut<NodeBuilder>> {
        match self {
            NodeBuilder::IONode(inner) => inner.get_all_connections(),
            NodeBuilder::Crossing(inner) => inner.get_all_connections(),
            NodeBuilder::Street(inner) => inner.get_all_connections(),
        }
    }

    fn is_connected(&self, other: &IntMut<NodeBuilder>) -> bool {
        has_connection(&self, other)
    }

    fn get_weight(&self) -> f32 {
        match self {
            NodeBuilder::IONode(inner) => inner.get_weight(),
            NodeBuilder::Crossing(inner) => inner.get_weight(),
            NodeBuilder::Street(inner) => inner.get_weight(),
        }
    }

    fn get_id(&self) -> usize {
        match self {
            NodeBuilder::IONode(n) => n.get_id(),
            NodeBuilder::Crossing(n) => n.get_id(),
            NodeBuilder::Street(n) => n.get_id(),
        }
    }

    fn set_id(&mut self, id: usize) {
        match self {
            NodeBuilder::IONode(n) => n.id = id,
            NodeBuilder::Crossing(n) => n.id = id,
            NodeBuilder::Street(n) => n.id = id,
        }
    }

    fn remove_connection(&mut self, conn: &WeakIntMut<NodeBuilder>) {
        match self {
            NodeBuilder::IONode(n) => n.remove_connection(conn),
            NodeBuilder::Crossing(n) => n.remove_connection(conn),
            NodeBuilder::Street(n) => n.remove_connection(conn),
        }
    }
}

dyn_clone::clone_trait_object!(NodeBuilderTrait);

/// Builder for [Street]
#[derive(Debug, Clone)]
pub struct StreetBuilder {
    /// the node the street ends in
    pub conn_out: Option<WeakIntMut<NodeBuilder>>,
    /// the node the street starts at
    pub conn_in: Option<WeakIntMut<NodeBuilder>>,
    /// the number of lanes of a Street
    pub lanes: u8,
    /// the lenght of a street
    pub lane_length: f32,
    /// the unique id of a street
    pub id: usize,
}
impl NodeBuilderTrait for StreetBuilder {
    fn build(&self) -> Node {
        Node::Street(Street {
            lanes: vec![Traversible::<RandCar>::new(self.lane_length)],
            conn_in: None,
            conn_out: None,
            id: self.id,
        })
    }
    fn get_out_connections<'a>(&'a self) -> Vec<WeakIntMut<NodeBuilder>> {
        let mut out = Vec::new();
        if let Some(conn) = &self.conn_out {
            out.push(conn.clone());
        }
        out
    }
    fn get_all_connections<'a>(&'a self) -> Vec<WeakIntMut<NodeBuilder>> {
        let mut out = Vec::new();
        if let Some(conn) = &self.conn_out {
            out.push(conn.clone());
        }
        if let Some(conn) = &self.conn_in {
            out.push(conn.clone());
        }
        out
    }
    fn get_weight(&self) -> f32 {
        self.lanes as f32
    }
    fn get_id(&self) -> usize {
        self.id
    }
    fn set_id(&mut self, id: usize) {
        self.id = id
    }

    fn is_connected(&self, other: &IntMut<NodeBuilder>) -> bool {
        match &self.conn_out {
            Some(conn) => conn == other,
            None => false,
        }
    }

    fn remove_connection(&mut self, conn: &WeakIntMut<NodeBuilder>) {
        if let Some(conn_in) = &self.conn_in {
            if *conn_in == *conn {
                self.conn_in = None;
                return;
            }
        }
        if let Some(conn_out) = &self.conn_out {
            if *conn_out == *conn {
                self.conn_out = None;
                return;
            }
        }
    }
}
impl StreetBuilder {
    /// sets the connection to the new value
    ///
    /// if the specified connection is already present, it is simply overwritten
    ///
    /// FIXME: Check if the [NodeBuilder] the connection points to already exists. If
    /// this is the case, remove the connection from this [NodeBuilder]
    pub fn connect(&mut self, conn_type: InOut, other: &IntMut<NodeBuilder>) -> &mut Self {
        let new_item = Some(other.downgrade());
        match conn_type {
            InOut::IN => self.conn_in = new_item,
            InOut::OUT => self.conn_out = new_item,
        }
        self
    }
}

impl StreetBuilder {
    /// sets the length
    pub fn with_length(mut self, length: f32) -> Self {
        self.lane_length = length;
        self
    }
    /// sets the number of lanes
    pub fn with_lanes(mut self, lanes: u8) -> Self {
        self.lanes = lanes;
        self
    }
    /// returns a new [StreetBuilder] that is connected to nothing
    pub fn new() -> Self {
        Self {
            conn_out: None,
            conn_in: None,
            lanes: 1,
            lane_length: 100.0,
            id: 0,
        }
    }
}

/// [IONode]s represent either an input or an output of the simulation
///
/// # Usage
/// ## Creating IONodes
#[derive(Debug, Clone)]
pub struct IONodeBuilder {
    /// the output connections of an IONode
    pub connections_out: Vec<WeakIntMut<NodeBuilder>>,
    /// the input connections of a IONode
    pub connections_in: Vec<WeakIntMut<NodeBuilder>>,
    /// The spawn rate (probability per timestep)
    pub spawn_rate: f64,
    /// the unique id of a IONode
    pub id: usize,
}
impl NodeBuilderTrait for IONodeBuilder {
    fn build(&self) -> Node {
        Node::IONode(IONode {
            connections: Vec::new(),
            spawn_rate: self.spawn_rate,
            time_since_last_spawn: 0.0,
            absorbed_cars: 0,
            id: self.id,
        })
    }
    fn get_out_connections(&self) -> Vec<WeakIntMut<NodeBuilder>> {
        self.connections_out.clone()
    }
    fn get_all_connections(&self) -> Vec<WeakIntMut<NodeBuilder>> {
        let mut out = self.connections_out.clone();
        out.append(&mut self.connections_in.clone());
        out
    }
    fn get_weight(&self) -> f32 {
        self.spawn_rate as f32
    }
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id
    }

    fn is_connected(&self, other: &IntMut<NodeBuilder>) -> bool {
        self.connections_out.iter().find(|n| *n == other).is_some()
    }

    fn remove_connection(&mut self, conn: &WeakIntMut<NodeBuilder>) {
        self.connections_out.retain(|c| c != conn);
        self.connections_in.retain(|c| c != conn);
    }
}
impl IONodeBuilder {
    /// returns a new Builder with id set to zero
    pub fn new() -> IONodeBuilder {
        IONodeBuilder {
            connections_out: Vec::new(),
            connections_in: Vec::new(),
            spawn_rate: 1.0,
            id: 0,
        }
    }
    /// set spawn rate in cars / second
    pub fn spawn_rate(&mut self, rate: f64) -> &mut Self {
        self.spawn_rate = rate;
        self
    }
    /// connects to other nodes. An IONode can have an indefinite amount of connections
    pub fn connect(&mut self, in_out: InOut, n: &IntMut<NodeBuilder>) {
        match in_out {
            InOut::IN => self.connections_in.push(n.downgrade()),
            InOut::OUT => self.connections_out.push(n.downgrade()),
        }
    }
}

/// North, East, South, West
#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum Direction {
    ///
    N,
    ///
    E,
    ///
    S,
    ///
    W,
}

/// Used to define wether connections are an input or output
#[derive(Debug, Clone)]
pub enum InOut {
    /// Input
    IN,
    /// Output
    OUT,
}

/// This struct encapsulates logic to hande the rather complex way
/// Crossings can be connected to streets or other nodes
///
/// The idea is that a Crossing has a square shape of which each
/// side can connect to one input street and one output street.
#[derive(Clone, Debug)]
pub struct CrossingConnections<T = NodeBuilder> {
    /// output streets by direction
    pub input: HashMap<Direction, WeakIntMut<T>>,
    /// output streets by direction
    pub output: HashMap<Direction, WeakIntMut<T>>,
}

impl<T> CrossingConnections<T> {
    /// Creates a new [CrossingConnections] holding connections of type `T`
    pub fn new() -> CrossingConnections<T> {
        CrossingConnections {
            input: HashMap::<Direction, WeakIntMut<T>>::new(),
            output: HashMap::<Direction, WeakIntMut<T>>::new(),
        }
    }
    /// Adds a new connection at the specified position
    ///
    /// Returns an error if the connection already exsists
    pub fn add(
        &mut self,
        dir: Direction,
        conn_type: InOut,
        conn: &IntMut<T>,
    ) -> Result<(), String> {
        let connection: &mut HashMap<Direction, WeakIntMut<T>>;
        match conn_type {
            InOut::IN => {
                assert!(!self.is_connected(InOut::OUT, conn));
                connection = &mut self.input
            }
            InOut::OUT => {
                assert!(!self.is_connected(InOut::IN, conn));
                connection = &mut self.output
            }
        }
        println!("Trying to set {:?}, {:?}", conn_type, dir);
        match connection.get(&dir) {
            Some(_value) => {
                return Err(format!(
                    "Connection at ({:?}, {:?}) already exists",
                    conn_type, dir
                ))
            }
            None => {
                connection.insert(dir, conn.downgrade());
            }
        }
        Ok(())
    }
    /// Removes the specified connection and returns Some(connections) if it exists
    /// or None if there is no such connection
    pub fn pop(&mut self, dir: Direction, conn_type: InOut) -> Option<WeakIntMut<T>> {
        let connection: &mut HashMap<Direction, WeakIntMut<T>>;
        match conn_type {
            InOut::IN => connection = &mut self.input,
            InOut::OUT => connection = &mut self.output,
        }
        connection.remove(&dir)
    }

    /// removes all connections that point to `conn`
    pub fn remove_connection(&mut self, conn_type: InOut, conn: &WeakIntMut<T>) {
        let connection: &mut HashMap<Direction, WeakIntMut<T>>;
        match conn_type {
            InOut::IN => connection = &mut self.input,
            InOut::OUT => connection = &mut self.output,
        }
        // remove all connections that point to the same object as `conn`
        connection.retain(|_k, v| !(v == conn));
    }
    /// returns true if the connection at the given position exists
    pub fn is_connected(&self, conn_type: InOut, node: &IntMut<T>) -> bool {
        let connection: &HashMap<Direction, WeakIntMut<T>>;
        match conn_type {
            InOut::IN => connection = &self.input,
            InOut::OUT => connection = &self.output,
        }
        connection.values().find(|v| *v == node).is_some()
    }
    /// Returns `Some(Direction)` for an item if it is saved in the connections
    pub fn get_direction_for_item(&self, conn_type: InOut, item: &IntMut<T>) -> Option<Direction> {
        let connection: &HashMap<Direction, WeakIntMut<T>>;
        match conn_type {
            InOut::IN => connection = &self.input,
            InOut::OUT => connection = &self.output,
        }
        let search_results = connection.iter().find(|&(_k, v)| {
            // Both point to the same internal T
            v == item
        });
        // Transform the results to match the function signature
        match search_results {
            Some((k, _v)) => Some(k.clone()),
            None => None,
        }
    }
    /// Returns true, if there is a conneciton at the specified position
    pub fn has_connection(&self, conn_type: InOut, dir: Direction) -> bool {
        let connection: &HashMap<Direction, WeakIntMut<T>>;
        match conn_type {
            InOut::IN => connection = &self.input,
            InOut::OUT => connection = &self.output,
        }
        connection.contains_key(&dir)
    }
}
// impl Debug for CrossingConnections {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut reformatted_in = HashMap::<Direction, usize>::new();
//         let mut reformatted_out = HashMap::<Direction, usize>::new();
//         for (k, v) in self.input.iter() {
//             let next_node = v.upgrade().unwrap().lock().unwrap().get_out_connections()[0].upgrade().unwrap();
//             let next_node = next_node.try_lock();
//             println!("Beföre Löck");
//             if let Ok(index) = next_node{
//                 println!("Äftör Löck");
//                 reformatted_in.insert(k.clone(), index.get_id());
//             }
//         }
//         for (k, v) in self.output.iter() {
//             let next_node = v.upgrade().unwrap().lock().unwrap().get_out_connections()[0].upgrade().unwrap();
//             let next_node = next_node.try_lock();
//             println!("Beföre Löck");
//             if let Ok(index) = next_node{
//                 println!("Äftör Löck");
//                 reformatted_in.insert(k.clone(), index.get_id());
//             }
//         }
//         f.debug_struct("CrossingConnections")
//             .field("input", &reformatted_in)
//             .field("output", &reformatted_out)
//             .finish()
//     }
// }

/// Defines the settings for a Crossing to later on construct it with the build method
#[derive(Debug, Clone)]
pub struct CrossingBuilder {
    /// the [CrossingConnections] struct is used to abstract the logic of adding and removing connections
    pub connections: CrossingConnections,
    /// The length a car has to traverse when traversing
    /// the crossing
    length: f32,
    /// the id of a crossing builder in the simulation
    pub id: usize,
}
impl NodeBuilderTrait for CrossingBuilder {
    fn build(&self) -> Node {
        Node::Crossing(Crossing {
            connections: CrossingConnections::new(),
            car_lane: Traversible::<RandCar>::new(self.length),
            id: self.id,
            traffic_light_state: TrafficLightState::S0
        })
    }
    fn get_out_connections(&self) -> Vec<WeakIntMut<NodeBuilder>> {
        self.connections
            .output
            .values()
            .map(|c| c.clone())
            .collect()
    }
    fn get_all_connections(&self) -> Vec<WeakIntMut<NodeBuilder>> {
        let mut cout: Vec<WeakIntMut<NodeBuilder>> = self
            .connections
            .output
            .values()
            .map(|c| c.clone())
            .collect();
        let mut cin = self.connections.input.values().map(|c| c.clone()).collect();
        cout.append(&mut cin);
        cout
    }
    fn get_weight(&self) -> f32 {
        1.0
    }
    fn get_id(&self) -> usize {
        self.id
    }

    fn set_id(&mut self, id: usize) {
        self.id = id
    }

    fn is_connected(&self, other: &IntMut<NodeBuilder>) -> bool {
        self.connections.is_connected(InOut::OUT, other)
    }

    fn remove_connection(&mut self, conn: &WeakIntMut<NodeBuilder>) {
        self.connections.remove_connection(InOut::IN, conn);
        self.connections.remove_connection(InOut::OUT, conn);
    }
}

impl CrossingBuilder {
    /// set the side length of the crossing (it is assumed to be a square)
    pub fn with_length(mut self, length: f32) -> CrossingBuilder {
        self.length = length;
        self
    }
    /// Constructs a new [CrossingBuilder] with id=0
    pub fn new() -> CrossingBuilder {
        CrossingBuilder {
            connections: CrossingConnections::new(),
            length: 10.0,
            id: 0,
        }
    }
    /// connects to node
    pub fn connect(
        &mut self,
        dir: Direction,
        conn_type: InOut,
        other: &IntMut<NodeBuilder>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        self.connections.add(dir, conn_type, other)?;
        Ok(self)
    }
    /// returns true, if there a connection is present at the specified position
    pub fn has_connection(&self, conn_type: InOut, dir: Direction) -> bool {
        self.connections.has_connection(conn_type, dir)
    }
}
