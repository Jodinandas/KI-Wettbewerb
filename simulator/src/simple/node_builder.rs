use std::{fmt::Debug, sync::Weak};

use super::{movable::RandCar, node::{Crossing, IONode, Node, Street}, traversible::Traversible};
use super::node::graphics;
use dyn_clone::DynClone;


pub enum NodeBuilderType {
    IONode,
    Crossing,
    Street,
}
pub trait NodeBuilderTrait : Debug + DynClone + Sync + Send {
    fn build(&self) -> Node;
    fn get_connections(&self) -> &Vec<usize>;
    fn connect(&mut self, i: usize);
    fn generate_graphics_info(&self) -> graphics::Info;
    fn get_node_type(&self) -> NodeBuilderType;
    fn get_weight(&self) -> f32;
    // id in the global list of nodebuilders
    fn id(&self) -> usize;
    fn set_id(&mut self, id: usize);
}

dyn_clone::clone_trait_object!(NodeBuilderTrait);

#[derive(Debug, Clone)]
pub struct StreetBuilder {
    /// indices in SimulationBuilder
    connection: Vec<usize>,
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
    fn get_connections(&self) -> &Vec<usize> {
        &self.connection
    }
    fn connect(&mut self, i: usize) {
        self.connection.clear();
        self.connection.push(i);
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Street( graphics::StreetInfo {
            lanes: self.lanes
        })
    }
    fn get_node_type(&self) -> NodeBuilderType {
        NodeBuilderType::Street
    }

    fn get_weight(&self) -> f32 {
        self.lanes as f32
    }
    fn id(&self) -> usize {self.id}
    fn set_id(&mut self, id: usize) {
        self.id = id
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
    connections: Vec<usize>,
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
    fn get_connections(&self) -> &Vec<usize> {
        &self.connections
    }
    fn connect(&mut self, i: usize) {
        self.connections.push(i);
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::IONode( graphics::IONodeInfo{

        })
    }
    fn get_node_type(&self) -> NodeBuilderType {
        NodeBuilderType::IONode
    }

    fn get_weight(&self) -> f32 {
        self.spawn_rate as f32
    }
    fn id(&self) -> usize {self.id}

    fn set_id(&mut self, id: usize) {
        self.id = id
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
    connections: Vec<usize>,
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
    fn get_connections(&self) -> &Vec<usize> {
        &self.connections
    }
    fn connect(&mut self, i: usize) {
        self.connections.push(i);
    }
    fn generate_graphics_info(&self) -> graphics::Info {
        graphics::Info::Crossing( graphics::CrossingInfo{

        })
    }
    fn get_node_type(&self) -> NodeBuilderType {
        NodeBuilderType::Crossing
    }

    fn get_weight(&self) -> f32 {
        1.0
    }
    fn id(&self) -> usize {self.id}

    fn set_id(&mut self, id: usize) {
        self.id = id
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