/// This enum represents all types of simulation data types
///
/// the connections are not safed as references, but rather as
/// indices in the list of all parts of the simulation, to avoid
/// the overhead (and tremendous complexity and annoyance of using
/// these types of e.g. a nested ```Weak<RefCell<Node>>```
///
/// The consequence of this way of organizing the data is that
/// things like moving cars from one ```Node``` to another has
/// to be done by the simulator and not by functions implemented
/// in the Node.
#[derive(Debug)]
pub enum Node {
    Crossing {
        connections: Vec<usize>
    },
    IONode{
        connections: Vec<usize>
    },
    Street{
        connection: usize,
        lanes: u8
    } 
}
