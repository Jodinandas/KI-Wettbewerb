use std::fmt::Debug;
use std::rc::Rc;
use std::cell::RefCell;
use super::connection::Connection;


pub trait Node : Debug {
    fn is_connected(&mut self, other: usize) -> bool;
    fn connect(&mut self, other: usize);
}
pub trait Crossing : Node {
}
pub trait IONode: Node {

}