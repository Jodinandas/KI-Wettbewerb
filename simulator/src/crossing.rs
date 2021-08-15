use std::rc::Rc;
use std::ptr;
use std::cell::RefCell;
use super::connection::Connection;

#[derive(Debug, PartialEq)]
pub struct Crossing {
    connections: Vec<Connection>,
    pub is_io_node: bool,
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
            lanes: lane_count,
            speed_limit: 60.0
        };
        self.connections.push(new_connection)
    }
}


mod tests {
    use super::*;
    #[test]
    fn create_crossing() {
        Crossing::new(true);
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