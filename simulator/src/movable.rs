use crate::datastructs::IntMut;

use super::{int_mut::WeakIntMut, node::Node, traits::Movable};
use rand::Rng;
use std::error::Error;

/// A person that takes turn at random
#[derive(Debug, Clone)]
pub struct RandPerson {
    speed: f32,
}

impl Movable for RandPerson {
    fn get_speed(&self) -> f32 {
        self.speed
    }
    fn set_speed(&mut self, s: f32) {
        self.speed = s
    }
    fn update(&mut self, _t: f64) {}
    fn decide_next(
        &mut self,
        connections: &Vec<WeakIntMut<Node<Self>>>,
        current_node: &IntMut<Node<Self>>,
    ) -> Result<Option<WeakIntMut<Node<Self>>>, Box<dyn Error>> {
        let i = rand::thread_rng().gen_range(0..connections.len());
        Ok(Some(connections[i].clone()))
    }
}

/// A car that takes turn at random
#[derive(Debug, Clone)]
pub struct RandCar {
    speed: f32,
}

impl RandCar {
    /// returns a car with default speed
    pub fn new() -> RandCar {
        RandCar { speed: 2.0 }
    }
}

impl Movable for RandCar {
    fn get_speed(&self) -> f32 {
        self.speed
    }
    fn set_speed(&mut self, s: f32) {
        self.speed = s
    }
    fn update(&mut self, _t: f64) {}
    fn decide_next(
        &mut self,
        connections: &Vec<WeakIntMut<Node<Self>>>,
        current_node: &IntMut<Node<Self>>,
    ) -> Result<Option<WeakIntMut<Node>>, Box<dyn Error>> {
        let i = rand::thread_rng().gen_range(0..connections.len());
        Ok(Some(connections[i].clone()))
    }
}

/// This struct encapsulates data for a [Movable] (to render it later)
pub struct MovableStatus {
    /// the Movable's position on the street (crossings and ionodes are not supported yet)
    pub position: f32,
    /// random index that is used differently by different nodes
    pub lane_index: u8,
}
