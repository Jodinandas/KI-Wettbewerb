use std::{fmt::Debug, ptr, sync::{Arc, Mutex, Weak}, vec};

use super::{movable::RandCar, node::{Crossing, IONode, Node, Street}, traversible::Traversible};
use super::node::graphics;
use dyn_clone::DynClone;

#[derive(Debug, Clone)]
pub enum NodeBuilder {
    IONode(IONodeBuilder),
    Crossing(CrossingBuilder),
    Street(StreetBuilder),
}

fn has_connection(node_a: &NodeBuilder, node_b: &Arc<Mutex<NodeBuilder>>) -> bool {
    node_a.get_connections().iter().find(
        | n | { 
            ptr::eq(n.as_ptr(), &**node_b)
        }
    ).is_some()
}

impl NodeBuilderTrait for NodeBuilder {
    fn build(&self) -> Node {
        match self {
            NodeBuilder::IONode(inner) => inner.build(),
            NodeBuilder::Crossing(inner) => inner.build(),
            NodeBuilder::Street(inner) => inner.build(),
        }
    }

    fn get_connections(&self) -> &Vec<Weak<Mutex<NodeBuilder>>> {
        match self {
            NodeBuilder::IONode(inner) => inner.get_connections(),
            NodeBuilder::Crossing(inner) => inner.get_connections(),
            NodeBuilder::Street(inner) => inner.get_connections(),
        }
    }

    fn connect(&mut self, i: &Arc<Mutex<NodeBuilder>>) {
        match self {
            NodeBuilder::IONode(inner) => inner.connect(i),
            NodeBuilder::Crossing(inner) => inner.connect(i),
            NodeBuilder::Street(inner) => inner.connect(i),
        }
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

    fn id(&self) -> usize {
        todo!()
    }

    fn set_id(&mut self, id: usize) {
        todo!()
    }

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        has_connection(&self, other)
    }
}


pub trait NodeBuilderTrait : Debug + DynClone + Sync + Send {
    fn build(&self) -> Node;
    fn get_connections(&self) -> &Vec<Weak<Mutex<NodeBuilder>>>;
    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool;
    fn connect(&mut self, i: &Arc<Mutex<NodeBuilder>>);
    fn generate_graphics_info(&self) -> graphics::Info;
    fn get_weight(&self) -> f32;
    // id in the global list of nodebuilders
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}

dyn_clone::clone_trait_object!(NodeBuilderTrait);

#[derive(Debug, Clone)]
pub struct StreetBuilder {
    /// indices in SimulationBuilder
    connection: Vec<Weak<Mutex<NodeBuilder>>>,
    lanes: u8,
    lane_length: f32,
    id: usize
}
impl NodeBuilderTrait for StreetBuilder {
    fn build(&self) -> Node {
        Node::Street(Street {
            lanes: self.lanes,
            car_lane: Traversible::<RandCar>::new(self.lane_length),
            end: vec!(Weak::new()),
            id: self.id
        })
    }
    fn get_connections(&self) -> &Vec<Weak<Mutex<NodeBuilder>>> {
        &self.connection
    }
    fn connect(&mut self, i: &Arc<Mutex<NodeBuilder>>) {
        self.connection = vec!(Arc::downgrade(i))
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Street( graphics::StreetInfo {
            lanes: self.lanes
        })
    }
    fn get_weight(&self) -> f32 {
        self.lanes as f32
    }
    fn id(&self) -> usize {self.id}
    fn set_id(&mut self, id: usize) {
        self.id = id
    }

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        self.connection.iter().find(
        | n |
            ptr::eq(n.as_ptr(), &**other)
        ).is_some()
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
            connection: Vec::new(),
            lanes: 1,
            lane_length: 100.0,
            id: 0
        }
    }
}

#[derive(Debug, Clone)]
pub struct IONodeBuilder {
    connections: Vec<Weak<Mutex<NodeBuilder>>>,
    spawn_rate: f64,
    id: usize
}
impl NodeBuilderTrait for IONodeBuilder {
    fn build(&self) -> Node{
        Node::IONode(IONode {
            connections: Vec::new(),
            spawn_rate: self.spawn_rate,
            time_since_last_spawn: 0.0,
            absorbed_cars: 0,
            id: self.id
        })
    }
    fn get_connections(&self) -> &Vec<std::sync::Weak<Mutex<NodeBuilder>>> {
        &self.connections
    }
    fn connect(&mut self, n: &Arc<Mutex<NodeBuilder>>) {
        self.connections.push(Arc::downgrade(n));
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::IONode( graphics::IONodeInfo{

        })
    }
    fn get_weight(&self) -> f32 {
        self.spawn_rate as f32
    }
    fn id(&self) -> usize {self.id}

    fn set_id(&mut self, id: usize) {
        self.id = id
    }

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        self.connections.iter().find(
        | n |
            ptr::eq(n.as_ptr(), &**other)
        ).is_some()
    }
}
impl IONodeBuilder {
    pub fn new() -> IONodeBuilder {
        IONodeBuilder {
            connections: Vec::new(),
            spawn_rate: 1.0,
            id: 0
        }
    }
    // Spawn rate in cars / second
    pub fn spawn_rate(&mut self, rate: f64) -> &mut Self {
        self.spawn_rate = rate;
        self
    }
}

#[derive(Debug, Clone)]
pub struct CrossingBuilder {
    connections: Vec<Weak<Mutex<NodeBuilder>>>,
    /// ???? why length,
    length: f32,
    id: usize
}
impl NodeBuilderTrait for CrossingBuilder {
    fn build(&self) -> Node {
        Node::Crossing(Crossing {
            connections: Vec::new(),
            car_lane: Traversible::<RandCar>::new(self.length),
            id: self.id
        })
    }
    fn get_connections(&self) -> &Vec<std::sync::Weak<Mutex<NodeBuilder>>> {
        &self.connections
    }
    fn connect(&mut self, n: &Arc<Mutex<NodeBuilder>>) {
        self.connections.push(Arc::downgrade(n));
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Crossing( graphics::CrossingInfo{

        })
    }
    fn get_weight(&self) -> f32 {
        1.0
    }
    fn id(&self) -> usize {self.id}

    fn set_id(&mut self, id: usize) {
        self.id = id
    }

    fn is_connected(&self, other: &Arc<Mutex<NodeBuilder>>) -> bool {
        self.connections.iter().find(
        | n |
            ptr::eq(n.as_ptr(), &**other)
        ).is_some()
    }
}

impl CrossingBuilder {
    pub fn length(mut self, length: f32) -> CrossingBuilder {
        self.length = length;
        self
    }
    pub fn new() -> CrossingBuilder {
        CrossingBuilder {
            connections: Vec::new(),
            length: 10.0,
            id: 0
        }
    }
}