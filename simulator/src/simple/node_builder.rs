use std::fmt::Debug;

use enum_dispatch::enum_dispatch;
use crate::traits::NodeTrait;

use super::{movable::RandCar, node::{IONode, Street, Crossing}, traversible::Traversible};


#[enum_dispatch]
pub trait NodeBuilderTrait : Debug + Send + Sync {
    fn build(&self) -> Box<dyn NodeTrait>;
    fn get_connections(&self) -> &Vec<usize>;
    fn connect(&mut self, i: usize);
}

#[derive(Debug)]
pub struct StreetBuilder {
    /// indices in SimulationBuilder
    connection: Vec<usize>,
    lanes: u8,
    lane_length: f32 
}
impl NodeBuilderTrait for StreetBuilder {
    fn build(&self) -> Box<dyn NodeTrait> {
        Box::new(Street {
            connection: Vec::new(),
            lanes: self.lanes,
            car_lane: Traversible::<RandCar>::new(self.lane_length),
        })
    }
    fn get_connections(&self) -> &Vec<usize> {
        &self.connection
    }
    fn connect(&mut self, i: usize) {
        self.connection.clear();
        self.connection.push(i);
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
            lane_length: 100.0
        }
    }
}

#[derive(Debug)]
pub struct IONodeBuilder {
    connections: Vec<usize>,
    spawn_rate: f64
}
impl NodeBuilderTrait for IONodeBuilder {
    fn build(&self) -> Box<dyn NodeTrait>{
        Box::new(IONode {
            connections: Vec::new(),
            spawn_rate: self.spawn_rate,
            time_since_last_spawn: 0.0
        })
    }
    fn get_connections(&self) -> &Vec<usize> {
        &self.connections
    }
    fn connect(&mut self, i: usize) {
        self.connections.push(i);
    }
}
impl IONodeBuilder {
    pub fn new() -> IONodeBuilder {
        IONodeBuilder {
            connections: Vec::new(),
            spawn_rate: 1.0
        }
    }
    // Spawn rate in cars / second
    pub fn spawn_rate(&mut self, rate: f64) -> &mut Self {
        self.spawn_rate = rate;
        self
    }
}

#[derive(Debug)]
pub struct CrossingBuilder {
    connections: Vec<usize>,
    /// ???? why length,
    length: f32
}
impl NodeBuilderTrait for CrossingBuilder {
    fn build(&self) -> Box<dyn NodeTrait> {
        Box::new(Crossing {
            connections: Vec::new(),
            car_lane: Traversible::<RandCar>::new(self.length)
        })
    }
    fn get_connections(&self) -> &Vec<usize> {
        &self.connections
    }
    fn connect(&mut self, i: usize) {
        self.connections.push(i);
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
            length: 10.0
        }
    }
}