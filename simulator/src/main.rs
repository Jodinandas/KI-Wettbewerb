use std::env;
use std::rc::{Rc, Weak};
use std::ptr;
use std::cell::RefCell;


/// A struct representing the street network
///
/// The `StreetData` Struct itself holds a strong reference (`Rc` as opposed to `Weak`)
/// to the Crossings, while the Connections only hold weak references
/// to prevent reference cycles.
/// If the Connections held strong references, the memory wouldn't be cleaned
/// up when the StreetData goes out of scope, as the connections would form a
/// cycle
struct StreetData {
    crossings: Vec<Rc<RefCell<Crossing>>>,
}

impl StreetData {
    // fn from_json(path: &str) -> Result<StreetData, Error{
    //     
    // }
}

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
/// ```
/// // create a crossing
/// let mut c = Rc::new(Crossing::new(false));
/// let connection = Connection::new(&c);
/// ```
#[derive(Debug)]
struct Connection {
    /// A connection/street needs to point to a crossing
    ///
    /// As to why we use these nested types:
    /// [Rust tutorial -- Interior Mutability](https://doc.rust-lang.org/stable/book/ch15-05-interior-mutability.html)
    /// 
    /// Weak is used here to prevent [reference cycles](https://doc.rust-lang.org/stable/book/ch15-06-reference-cycles.html#preventing-reference-cycles-turning-an-rct-into-a-weakt)
    crossing: Weak<RefCell<Crossing>>,
    /// the lanes determine how much througput a connection has
    lanes: u8
}
impl Connection {
    fn new(crossing: &Rc<RefCell<Crossing>>) -> Connection {
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

#[derive(Debug, PartialEq)]
struct Crossing {
    connections: Vec<Connection>,
    is_io_node: bool,
}

impl Crossing {
    pub fn new(is_io_node: bool) -> Crossing {
        Crossing {
            is_io_node,
            connections: Vec::new()
        } 
    }
    /// Get `Connection` to a crossing if it exists
    pub fn get_connection(&self, other: &Rc<RefCell<Crossing>>) -> Option<&Connection> {
        for c in self.connections.iter() {
            // check if the two Rc point to the same crossing
            // Omg, this is soo unreadable
            // It is adapted from [this part](https://doc.rust-lang.org/std/rc/struct.Weak.html#examples-1) of the documentation
            // 
            // The extra `*` needs to be used because in the example we don't have a reference, and
            // here we need to dereference it
            if ptr::eq(&**other, c.crossing.as_ptr()) {
                return Some(c)
            }
        }
        None
    }
    /// connects to another `Crossing`
    /// 
    /// **Be careful**: This only forms a one-way connection and 
    /// **DOES NOT** check if the connection already exists, because this
    /// would create a performance penalty
    ///
    pub fn connect(&mut self, other: &Rc<RefCell<Crossing>>, lane_count: u8){
        // create new connection with reference to other
        let new_connection = Connection {
            crossing: Rc::downgrade(&other),
            lanes: lane_count
        };
        self.connections.push(new_connection)
    }
}


fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*; 

    #[test]
    fn create_crossing() {
        Crossing::new(true);
    }
    #[test]
    fn street_data_from_json() {
    }
    #[test]
    fn create_new_connection() {
        let c = Rc::new(RefCell::new(Crossing::new(false)));
        Connection::new(&c);
    }
    #[test]
    #[should_panic]
    fn connect_already_connected_crossings() {
        // create two crossings, connect them, and
        // check if the lane counts are as expected
        let c1 = Rc::new(RefCell::new(Crossing::new(false)));
        let c2 = Rc::new(RefCell::new(Crossing::new(false)));
        c1.borrow_mut().connect(&c2, 1);
        c2.borrow_mut().connect(&c1, 3);
        c1.borrow_mut().connect(&c2, 1);
        let lane_count_c1 = (*c1.borrow()).connections.get(0).unwrap().lanes;
        let lane_count_c2 = (*c2.borrow()).connections.get(0).unwrap().lanes;
        assert_eq!(lane_count_c1, 2);
        assert_eq!(lane_count_c2, 3);
    }
    #[test]
    fn connect_crossings() {
        // create two crossings, connect them, and
        // check if the lane counts are as expected
        let c1 = Rc::new(RefCell::new(Crossing::new(false)));
        let c2 = Rc::new(RefCell::new(Crossing::new(false)));
        c1.borrow_mut().connect(&c2, 1);
        c2.borrow_mut().connect(&c1, 3);
        let lane_count_c1 = (*c1.borrow()).connections.get(0).unwrap().lanes;
        let lane_count_c2 = (*c2.borrow()).connections.get(0).unwrap().lanes;
        assert_eq!(lane_count_c1, 1);
        assert_eq!(lane_count_c2, 3);
    }
    #[test]
    fn get_connection() {
        // Make a lot of connections and then make sure the method finds the right one
        let c1 = Rc::new(RefCell::new(Crossing::new(false)));
        let c2 = Rc::new(RefCell::new(Crossing::new(false)));
        for _ in 0..50 {
            let c = Rc::new(RefCell::new(Crossing::new(false)));
            c1.borrow_mut().connect(&c, 1);
        }
        c1.borrow_mut().connect(&c2, 1);
        for _ in 0..50 {
            let c = Rc::new(RefCell::new(Crossing::new(false)));
            c1.borrow_mut().connect(&c, 1);
        }
        assert_eq!(Connection::new(&c2), *c1.borrow().get_connection(&c2).unwrap());
    }
}
