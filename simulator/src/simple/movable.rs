use std::{sync::Mutex, error::Error, sync::{Arc, Weak}};

use super::{super::traits::Movable, node::Node};
use rand::Rng;

#[derive(Debug)]
#[derive(Clone)]
pub struct RandPerson {
    speed: f32,
}

impl Movable for RandPerson {
    fn get_speed(&self) -> f32 {self.speed}
    fn set_speed(&mut self, s: f32) {self.speed = s}
    fn update(&mut self, _t: f64) {}
    fn decide_next(&mut self, connections: &Vec<Weak<Mutex<Node>>>) -> Result<Weak<Mutex<Node>>, Box<dyn Error>> {
        let i = rand::thread_rng().gen_range(0..connections.len());
        Ok(connections[i].clone())
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct RandCar {
    speed: f32
}

impl RandCar {
    pub fn new() -> RandCar {
        RandCar {
            speed: 2.0
        }
    }
}

impl Movable for RandCar {
    fn get_speed(&self) -> f32 {self.speed}
    fn set_speed(&mut self, s: f32) {self.speed = s}
    fn update(&mut self, _t: f64) {}
    fn decide_next(&mut self, connections: &Vec<Weak<Mutex<Node>>>) -> Result<Weak<Mutex<Node>>, Box<dyn Error>> {
        let i = rand::thread_rng().gen_range(0..connections.len());
        Ok(connections[i].clone())
    }
}
