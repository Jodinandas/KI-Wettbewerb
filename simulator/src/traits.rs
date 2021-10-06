use std::fmt::Debug;
use std::error::Error;
use dyn_clone::DynClone;


use crate::simple::movable::RandCar;


/// This is a trait defining all functionality a Node needs
///
/// All Node variants must implement this trait
/// 
/// The nodes are mostly used in the form of `Box<dyn Node>`
pub trait NodeTrait<Car=RandCar> : Debug + DynClone {
    fn is_connected(&self, other: &usize) -> bool;
    fn connect(&mut self, other: &usize);
    fn update_cars(&mut self, t: f64) -> Vec<Car>;
    fn get_connections(&self) -> &Vec<usize>;
    fn add_car(&mut self, car: Car);
}

// make it possible to derive Clone for structs with Box<dyn NodeTrait>
dyn_clone::clone_trait_object!(NodeTrait);


/// This trait represents some kind of movable
/// 
/// idea for movables:
///  use the delta t when updating to weigh a chance of 
/// come action taking place internally. Example: Going into a shop
/// for 10 min or maybe someone tripping 
pub trait Movable : Debug + DynClone {
    fn get_speed(&self) -> f32;
    fn set_speed(&mut self, s: f32);
    fn update(&mut self, t: f64);
    /// Decides the next node for the movable to move to 
    ///
    /// It can very well happen that the next node can't be determined
    /// if the part of the program that figures out the paths makes a mistake
    fn decide_next(&mut self, connections: &Vec<usize>) -> Result<usize, Box<dyn Error>>;
}

// make it possible to derive Clone for structs with Box<dyn Movable>
dyn_clone::clone_trait_object!(Movable);