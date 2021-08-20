use std::fmt::Debug;
use enum_dispatch::enum_dispatch;
use crate::simple::movable::RandCar;

use super::simple::node::*;


/// This is a trait defining all functionality a Node needs
///
/// All Node variants must implement this trait
/// `enum_dispatch` is used for easy of usability. 
/// 
/// **Without `enum_dispatch`** (doesn't work, but this is a part cut out
/// of the old implementation of `Simulator::connect_with_street`)
/// ```ignore
/// match &mut self.nodes[inode1] {
///     Node::Crossing {connections} => {connections.push(street_index)},
///     Node::IONode {connections} => {connections.push(street_index)},
///     Node::Street {connection, lanes: _} => {*connection = street_index}
/// }
/// ```
/// **With `enum_dispatch`**
/// ```ignore 
/// self.nodes[inode1].connect(street_index);
/// ```
/// (Of course, all the trait implementations are ommited, but even with,
/// using traits the first example wouldn't be too different)
#[enum_dispatch]
pub trait NodeTrait : Debug {
    fn is_connected(&self, other: usize) -> bool;
    fn connect(&mut self, other: usize);
    fn update_cars(&mut self, t: f64) -> Vec<RandCar>;
    fn get_connections(&self) -> Vec<usize>;
    fn add_car(&mut self, car: RandCar);
}

/// This trait represents some kind of movable
/// 
/// idea for movables:
///  use the delta t when updating to weigh a chance of 
/// come action taking place internally. Example: Going into a shop
/// for 10 min or maybe someone tripping 
pub trait Movable : Debug {
    fn get_speed(&self) -> f32;
    fn set_speed(&mut self, s: f32);
    fn update(&mut self, t: f64);
    fn decide_next<'a>(&mut self, connections: &'a Vec<usize>) -> &'a usize;
}
