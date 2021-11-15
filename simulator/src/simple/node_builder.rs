use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    hash::Hash,
    ptr,
    sync::{Arc, Mutex, Weak},
    vec,
};

use super::node::graphics;
use super::{
    movable::RandCar,
    node::{Crossing, IONode, Node, Street},
    traversible::Traversible,
};
use dyn_clone::DynClone;

#[derive(Debug, Clone)]
pub enum NodeBuilder {
    IONode(IONodeBuilder),
    Crossing(CrossingBuilder),
    Street(StreetBuilder),
}

pub trait NodeBuilderTrait: Debug + DynClone + Sync + Send {
    fn build(&self) -> Node;
    fn get_connections(&self) -> Vec<Weak<Mutex<NodeBuilder>>>;
    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool;
    // fn connect(&mut self, i: &Arc<Mutex<NodeBuilder>>);
    fn generate_graphics_info(&self) -> graphics::Info;
    fn get_weight(&self) -> f32;
    // id in the global list of nodebuilders
    fn get_id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}

fn has_connection(node_a: &NodeBuilder, node_b: &Arc<Mutex<NodeBuilder>>) -> bool {
    node_a
        .get_connections()
        .iter()
        .find(|n| ptr::eq(n.as_ptr(), &**node_b))
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

    fn get_connections(&self) -> Vec<Weak<Mutex<NodeBuilder>>> {
        match self {
            NodeBuilder::IONode(inner) => inner.get_connections(),
            NodeBuilder::Crossing(inner) => inner.get_connections(),
            NodeBuilder::Street(inner) => inner.get_connections(),
        }
    }

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        has_connection(&self, other)
    }

    fn generate_graphics_info(&self) -> graphics::Info {
        match self {
            NodeBuilder::IONode(inner) => inner.generate_graphics_info(),
            NodeBuilder::Crossing(inner) => inner.generate_graphics_info(),
            NodeBuilder::Street(inner) => inner.generate_graphics_info(),
        }
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
}

dyn_clone::clone_trait_object!(NodeBuilderTrait);

#[derive(Debug, Clone)]
pub struct StreetBuilder {
    conn_out: Option<Weak<Mutex<NodeBuilder>>>,
    conn_in: Option<Weak<Mutex<NodeBuilder>>>,
    lanes: u8,
    lane_length: f32,
    id: usize,
}
impl NodeBuilderTrait for StreetBuilder {
    fn build(&self) -> Node {
        Node::Street(Street {
            lanes: self.lanes,
            car_lane: Traversible::<RandCar>::new(self.lane_length),
            conn_in: None,
            conn_out: None,
            id: self.id,
        })
    }
    fn get_connections<'a>(&'a self) -> Vec<Weak<Mutex<NodeBuilder>>> {
        let mut out = Vec::new();
        if let Some(conn) = &self.conn_out {
            out.push(Weak::clone(conn));
        }
        out
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Street(graphics::StreetInfo { lanes: self.lanes })
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

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        match &self.conn_out {
            Some(conn) => ptr::eq(conn.as_ptr(), &**other),
            None => false,
        }
    }
}
impl StreetBuilder {
    pub fn connect(&mut self, conn_type: InOut, other: &Arc<Mutex<NodeBuilder>>) -> &mut Self {
        let new_item = Some(Arc::downgrade(other));
        match conn_type {
            InOut::IN => self.conn_in = new_item,
            InOut::OUT => self.conn_out = new_item,
        }
        self
    }
}

impl StreetBuilder {
    
    pub fn length(mut self, length: f32) -> Self {
        self.lane_length = length;
        self
    }
    pub fn lanes(mut self, lanes: u8) -> Self {
        self.lanes = lanes;
        self
    }
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

#[derive(Debug, Clone)]
pub struct IONodeBuilder {
    connections: Vec<Weak<Mutex<NodeBuilder>>>,
    spawn_rate: f64,
    id: usize,
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
    fn get_connections(&self) -> Vec<std::sync::Weak<Mutex<NodeBuilder>>> {
        self.connections.clone()
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::IONode(graphics::IONodeInfo {})
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

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        self.connections
            .iter()
            .find(|n| ptr::eq(n.as_ptr(), &**other))
            .is_some()
    }
}
impl IONodeBuilder {
    pub fn new() -> IONodeBuilder {
        IONodeBuilder {
            connections: Vec::new(),
            spawn_rate: 1.0,
            id: 0,
        }
    }
    // Spawn rate in cars / second
    pub fn spawn_rate(&mut self, rate: f64) -> &mut Self {
        self.spawn_rate = rate;
        self
    }
    pub fn connect(&mut self, n: &Arc<Mutex<NodeBuilder>>) {
        self.connections.push(Arc::downgrade(n));
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum Direction {
    N,
    E,
    S,
    W,
}

#[derive(Debug, Clone)]
pub enum InOut {
    IN,
    OUT,
}

#[derive(Clone, Debug)]
pub struct CrossingConnections<T = NodeBuilder> {
    pub input: HashMap<Direction, Weak<Mutex<T>>>,
    pub output: HashMap<Direction, Weak<Mutex<T>>>,
}

impl<T> CrossingConnections<T> {
    pub fn new() -> CrossingConnections<T> {
        CrossingConnections {
            input: HashMap::<Direction, Weak<Mutex<T>>>::new(),
            output: HashMap::<Direction, Weak<Mutex<T>>>::new(),
        }
    }
    /// Adds a new connection at the specified position
    ///
    /// Returns an error if the connection already exsists
    pub fn add(
        &mut self,
        dir: Direction,
        conn_type: InOut,
        conn: &Arc<Mutex<T>>,
    ) -> Result<(), String> {
        let connection: &mut HashMap<Direction, Weak<Mutex<T>>>;
        match conn_type {
            InOut::IN => connection = &mut self.input,
            InOut::OUT => connection = &mut self.output,
        }
        match connection.get(&dir) {
            Some(_value) => {
                return Err(format!(
                    "Connection at ({:?}, {:?}) already exists",
                    conn_type, dir
                ))
            }
            None => {
                connection.insert(dir, Arc::downgrade(conn));
            }
        }
        Ok(())
    }
    pub fn pop(&mut self, dir: Direction, conn_type: InOut) -> Option<Weak<Mutex<T>>> {
        let connection: &mut HashMap<Direction, Weak<Mutex<T>>>;
        match conn_type {
            InOut::IN => connection = &mut self.input,
            InOut::OUT => connection = &mut self.output,
        }
        connection.remove(&dir)
    }
    pub fn remove_connection(&mut self, conn_type: InOut, conn: &Arc<Mutex<T>>) {
        let connection: &mut HashMap<Direction, Weak<Mutex<T>>>;
        match conn_type {
            InOut::IN => connection = &mut self.input,
            InOut::OUT => connection = &mut self.output,
        }
        // remove all connections that point to the same object as `conn`
        connection.retain(|k, v| !ptr::eq(v.as_ptr(), &**conn));
    }
    pub fn is_connected(&self, conn_type: InOut, node: &Arc<Mutex<T>>) -> bool {
        let connection: &HashMap<Direction, Weak<Mutex<T>>>;
        match conn_type {
            InOut::IN => connection = &self.input,
            InOut::OUT => connection = &self.output,
        }
        connection
            .values()
            .find(|v| ptr::eq(v.as_ptr(), &**node))
            .is_some()
    }
    /// Returns `Some(Direction)` for an item if it is saved in the connections
    pub fn get_direction_for_item(&self, conn_type: InOut, item: &Arc<Mutex<T>>) -> Option<Direction>{
        let connection: &HashMap<Direction, Weak<Mutex<T>>>;
        match conn_type {
            InOut::IN => connection = &self.input,
            InOut::OUT => connection = &self.output,
        }
        let search_results = connection.iter().find(| &(k, v) |
            {
                // Both point to the same internal T
                ptr::eq(v.as_ptr(), &**item)
            }
        );
        // Transform the results to match the function signature
        match search_results {
            Some((k, v)) => Some(k.clone()),
            None => None,
        }

    }
}
// impl Debug for CrossingConnections {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut reformatted_in = HashMap::<Direction, usize>::new();
//         let mut reformatted_out = HashMap::<Direction, usize>::new();
//         for (k, v) in self.input.iter() {
//             let next_node = v.upgrade().unwrap().lock().unwrap().get_connections()[0].upgrade().unwrap();
//             let next_node = next_node.try_lock();
//             println!("Beföre Löck");
//             if let Ok(index) = next_node{
//                 println!("Äftör Löck");
//                 reformatted_in.insert(k.clone(), index.get_id());
//             }
//         }
//         for (k, v) in self.output.iter() {
//             let next_node = v.upgrade().unwrap().lock().unwrap().get_connections()[0].upgrade().unwrap();
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

#[derive(Debug, Clone)]
pub struct CrossingBuilder {
    pub connections: CrossingConnections,
    /// ???? why length,
    length: f32,
    id: usize,
}
impl NodeBuilderTrait for CrossingBuilder {
    fn build(&self) -> Node {
        Node::Crossing(Crossing {
            connections: CrossingConnections::new(),
            car_lane: Traversible::<RandCar>::new(self.length),
            id: self.id,
        })
    }
    fn get_connections(&self) -> Vec<std::sync::Weak<Mutex<NodeBuilder>>> {
        self.connections
            .output
            .values()
            .map(|c| Weak::clone(c))
            .collect()
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Crossing(graphics::CrossingInfo {})
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

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        self.connections.is_connected(InOut::OUT, other)
    }
}

impl CrossingBuilder {
    pub fn length(mut self, length: f32) -> CrossingBuilder {
        self.length = length;
        self
    }
    pub fn new() -> CrossingBuilder {
        CrossingBuilder {
            connections: CrossingConnections::new(),
            length: 10.0,
            id: 0,
        }
    }
    pub fn connect(
        &mut self,
        dir: Direction,
        conn_type: InOut,
        other: &Arc<Mutex<NodeBuilder>>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        self.connections.add(dir, conn_type, other)?;
        Ok(self)
    }
}
