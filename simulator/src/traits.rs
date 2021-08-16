use std::fmt::Debug;
use enum_dispatch::enum_dispatch;
use super::simple::node::*;


#[enum_dispatch]
pub trait NodeTrait : Debug {
    fn is_connected(&self, other: usize) -> bool;
    fn connect(&mut self, other: usize);
}