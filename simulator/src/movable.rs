use crate::datastructs::IntMut;

use super::{int_mut::WeakIntMut, node::Node, traits::Movable};
use rand::Rng;
use std::{error::Error, sync::MutexGuard};

/// A person that takes turn at random
#[derive(Debug, Clone)]
pub struct RandPerson {
    speed: f32,
    current_speed: f32,
    id: u32,
}

impl Movable for RandPerson {
    fn get_speed(&self) -> [f32;2] {
        [self.current_speed.clone(), self.speed.clone()]
    }
    fn set_speed(&mut self, s: f32) {
        self.speed = s
    }
    fn update(&mut self, _t: f32) {}
    fn decide_next(
        &self,
        connections: &Vec<WeakIntMut<Node<Self>>>,
        _current_node: &IntMut<Node<RandPerson>>,
    ) -> Result<Option<WeakIntMut<Node<Self>>>, Box<dyn Error>> {
        let i = rand::thread_rng().gen_range(0..connections.len());
        Ok(Some(connections[i].clone()))
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id
    }

    fn new() -> Self {
        RandPerson { speed: 0.0, id: 0, current_speed:0.0 }
    }

    fn set_current_speed(&mut self, cs: f32) {
        self.current_speed = cs
    }
}

/// A car that takes turn at random
#[derive(Debug, Clone)]
pub struct RandCar {
    current_speed: f32,
    speed: f32,
    id: u32,
}

impl RandCar {
    /// returns a car with default speed
    pub fn new() -> RandCar {
        RandCar { id: 0, speed: 2.0, current_speed: 0.0}
    }
}

impl Movable for RandCar {
    fn get_speed(&self) -> [f32;2] {
        [self.current_speed.clone(), self.speed.clone()]
    }
    fn set_speed(&mut self, s: f32) {
        self.speed = s
    }
    fn update(&mut self, _t: f32) {}
    fn decide_next(
        &self,
        connections: &Vec<WeakIntMut<Node<Self>>>,
        _current_node: &IntMut<Node>,
    ) -> Result<Option<WeakIntMut<Node>>, Box<dyn Error>> {
        let i = rand::thread_rng().gen_range(0..connections.len());
        Ok(Some(connections[i].clone()))
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id
    }

    fn new() -> Self {
        RandCar { speed: 0.0, id: 0, current_speed: 0.0}
    }

    fn set_current_speed(&mut self, cs: f32) {
        self.current_speed = cs
    }
}

/// This struct encapsulates data for a [Movable] (to render it later)
#[derive(Debug)]
pub struct MovableStatus {
    /// the Movable's position on the street (crossings and ionodes are not supported yet) as float
    /// between 0 and 1
    pub position: f32,
    /// random index that is used differently by different nodes
    pub lane_index: u8,
    /// each movable has a unique id
    pub movable_id: u32,
    /// should the node be deleted?
    pub delete: bool
}
