use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::crossing::Crossing;

/// A struct representing a street connected to another crossing
/// 
/// The crossing the connection points to is represented as a `Rc` type
/// holding a `RefCell` to enable mutable access to the crossing the
/// connection points to.
///
/// At the moment the lanes determine how much throughput a street/connection
/// has. 
///
/// # Examples
/// ```rust
/// use std::rc::Rc;
/// use std::cell::RefCell;
/// use simulator::connection::Connection;
/// use simulator::crossing::Crossing;
/// // create a crossing
/// let mut c = Rc::new(RefCell::new(Crossing::new(false)));
/// let connection = Connection::new(&c);
/// ```
#[derive(Debug)]
pub struct Connection {
    /// A connection/street needs to point to a crossing
    ///
    /// As to why we use these nested types:
    /// [Rust tutorial -- Interior Mutability](https://doc.rust-lang.org/stable/book/ch15-05-interior-mutability.html)
    /// 
    /// `Weak` is used here to prevent [reference cycles](https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html#preventing-reference-cycles-turning-an-rct-into-a-weakt)
    pub crossing: Weak<RefCell<Crossing>>,
    /// the lanes determine how much througput a connection has
    pub lanes: u8,
}
impl Connection {
    pub fn new(crossing: &Rc<RefCell<Crossing>>) -> Connection {
        Connection {
            crossing: Rc::downgrade(&crossing),
            lanes: 1,
        }
    }
}

/// Implement `PartialEq` to make it possible to compare Connections
impl PartialEq for Connection {
    /// Make sure they point to the same `Crossing` and also
    /// have the same amount of lanes
    fn eq(&self, other: &Connection) -> bool{
        self.crossing.as_ptr() == other.crossing.as_ptr() &&
        self.lanes == other.lanes
    }
    fn ne(&self, other: &Connection) -> bool{
        self.crossing.as_ptr() != other.crossing.as_ptr() ||
        self.lanes != other.lanes
    }
}
mod tests { 
    use super::*;
    #[test]
    fn create_new_connection() {
        let c = Rc::new(RefCell::new(Crossing::new(false)));
        Connection::new(&c);
    }
}