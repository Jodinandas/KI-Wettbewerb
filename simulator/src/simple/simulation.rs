use serde::Deserialize;
use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Display};
use std::rc::Rc;
use std::{cmp, ptr, thread};
use std::time::{Duration, SystemTime};
use crate::traits::NodeTrait;
use super::node::*;
use super::super::traits::Movable;

use super::node::Node;


/// A struct representing the street network
///
/// implementing functions for simulating the traffic
/// (moving cars, spawning new ones, moving pedestrians)
#[derive(Debug)]
pub struct Simulator {
    /// A list of all the crossings
    nodes: Vec<Rc<RefCell<Node>>>,
    max_iter: Option<usize>,
    delay: u64
}


/// Is raised when the conversion `JSON` -> `Simulator` fails
#[derive(Debug, Clone)]
pub struct JsonError (String);
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for JsonError {}

#[derive(Debug, Clone)]
pub struct IndexError (String);
impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for IndexError {}

/// The simulator, the top level struct that is instaniated to simulate traffic
/// 
/// # Examples
/// ## From JSON file
/// ```
/// use simulator::simple::simulation::Simulator;
/// let json: &str = r#"
/// {"crossings": [
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]},
///     {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
/// let mut simulator = Simulator::from_json(json);
/// ```
impl Simulator {
    /// Create new node
    pub fn new() -> Simulator {
        Simulator {
            nodes: Vec::new(),
            max_iter: None,
            delay: 0
        }
    }
    /// Add a new node
    pub fn add_node(&mut self, node: Node) -> &mut Simulator {
        self.nodes.push(Rc::new(RefCell::new(node)));
        self
    }
    /// creates a `Simulator` object from a `&str` formatted in a json-like way
    ///
    /// to see how the json must be formatted, look at the fields of
    /// `JsonCrossing` and `JsonRepresentation`
    /// 
    /// #TODO
    /// Reformat the json representation
    pub fn from_json(json: &str) -> Result<Simulator, Box<dyn Error>> {
        // Generate object holding all the data, still formatted in json way
        let json_representation: JsonRepresentation = serde_json::from_str(json)?;
        let mut crossings: Vec<Rc<RefCell<Node>>> = Vec::new();    
        // generate all crossings
        for json_crossing in json_representation.crossings.iter() {
            // create nodes from json object
            let new_crossing = match json_crossing.is_io_node {
                true => Rc::new(RefCell::new(IONode::new().into())),
                false => Rc::new(RefCell::new(Crossing::new().into()))
            };
            crossings.push(new_crossing);
        }
        let mut simulator = Simulator::new(); 
        simulator.nodes = crossings;
        // save the number of inital nodes to later check if the json points
        // to existing nodes that are not streets
        let inital_nodes = simulator.nodes.len();
        // connect the crossings with streets
        for (i, json_crossing) in json_representation.crossings.iter().enumerate() {
            // form all the connections defined in `JsonCrossing.connected`
            for (connection_index, lanes) in json_crossing.connected.iter() {
                // check if `Crossing`/`IONode` the street ends in actually exists
                if *connection_index > inital_nodes {
                    return Err(
                        Box::new(
                            JsonError("Connection points to node that doesn't exist".to_string())
                        )
                    );
                }
                // Make sure the connection doesn't already exist
                if (**simulator.nodes.get(i).unwrap()).borrow().is_connected(&Rc::downgrade(&simulator.nodes[*connection_index])) {
                    return Err(
                        Box::new(
                            JsonError("Attempt to create the same connection multiple times".to_string())
                        )
                    );
                }
                simulator.connect_with_street(i, *connection_index, *lanes)?;
            }
        }
        Ok(simulator)
    }
    /// Connects two node, ONE WAY ONLY, adding a street in between 
    pub fn connect_with_street(&mut self, inode1: usize, inode2: usize, lanes: u8) -> Result<(), Box<dyn Error>>{
        // make sure the second nodes actually exist
        if inode1 >= self.nodes.len() || inode2 >= self.nodes.len() {
            return Err(
                Box::new(
                    IndexError("Node doesn't exist".to_string())
                )
            );
        } 
        // create a new street to connect them
        let new_street: Rc<RefCell<Node>> = Rc::new(RefCell::new(Street::new().lanes(lanes).into()));
        (*new_street).borrow_mut().connect(&self.nodes[inode2]); 
        (*self.nodes[inode1]).borrow_mut().connect(&new_street);
        self.nodes.push(new_street);
        Ok(())
    }
    
    /// Update all nodes moving the cars and people to the next 
    /// nodes
    pub fn update_all_nodes(&mut self, dt: f64) {
        for i in 0..self.nodes.len() {
            let mut cars_at_end = (*self.nodes[i]).borrow_mut().update_cars(dt);
            let node =  (*self.nodes[i]).borrow();
            let options = node.get_connections();
            for j in cars_at_end.len()..0 {
                let next_i = cars_at_end[j].decide_next(&options);
                if let Some(reference) = next_i.upgrade() {
                    (*reference).borrow_mut().add_car(cars_at_end.pop().unwrap());
                }

            }
        }
    }
    
    pub fn simulation_loop(&mut self) -> Result<(), Box<dyn Error>>{
        let mut counter = 0;
        let mut iteration_compute_time;
        loop {
            let now = SystemTime::now();
            if let Some(max_iter) = self.max_iter {
                if counter > max_iter {break};
            }
            


            iteration_compute_time = now.elapsed()?.as_millis();
            // Convert the time to seconds and wait either as long as the
            // last iteration took if the iteration took longer than the 
            // specified delay or update using the delay
            let dt = cmp::max(self.delay as u128, iteration_compute_time) as f64 / 1000.0;
            self.sim_iter(dt);
            
            counter += 1;
            // TODO: Could case the system to wait an unnecessary millisecond
            thread::sleep(Duration::from_millis(
                cmp::min(self.delay - iteration_compute_time as u64, 0)
            ));
        }
        Ok(())
    }
    pub fn sim_iter(&mut self, dt: f64) {
        self.update_all_nodes(dt);
    }
    
    pub fn delay(&mut self, value: u64) -> &mut Simulator{
        self.delay = value;
        self
    }
    pub fn max_iter(&mut self, value: Option<usize>) -> &mut Simulator {
        self.max_iter = value;
        self
    }
}
/// Display to make it easier to check the connections etc.
/// 
/// # Intended look
/// `
/// Simulator{
///     nodes: [
///         0: Crossing -> 1, 2, 3
///     ]
/// }
/// `
impl Display for Simulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::from("Simulator {\n\tnodes: [\n");
        for (i, n) in self.nodes.iter().enumerate() {
            let name = (**n).borrow().name();
            s.push_str(
                &format!("\t\t{}: {} ->\t", i, name)
            );
            for _connection in (**n).borrow().get_connections().iter() {
                // find the index
                let mut index = 0;
                for (i, node) in self.nodes.iter().enumerate() {
                    if ptr::eq(n, node) {
                        index = i;
                        break;
                    }
                }
                
                s.push_str(
                    &format!("{}, ", &index)
                );
            }
            s.push_str("\n")
        } 
        s.push_str("\t]\n}");
        write!(f, "{}", s)
    }
}

/// This is just used to deserialize the JSON File to
/// an object that can be conveniently used in 
/// `StreetData::from_json`
/// 
#[derive(Debug, Deserialize)]
struct JsonCrossing {
    traffic_lights: bool,
    is_io_node: bool,
    connected: Vec<(usize, u8)>,
}
#[derive(Debug, Deserialize)]
/// Just for Deserialisation
struct JsonRepresentation {
    crossings: Vec<JsonCrossing>
}

/// This trait should be implemented for a frontend and signal
/// 
/// TODO: Actually implement it
///  It should be thread safe, potentially using a channel
///  Performance is not a priority, as this will be called only
///  if we look at an agent in detail
pub trait StreetDisplay {
    
}


mod tests {

    #[test]
    fn street_data_from_json() {
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        let data = super::Simulator::from_json(json).unwrap();
        println!("{:?}", &data);
    }
    
    #[test]
    fn connect_with_streets() {
        use super::{Street, IONode, Simulator};
        use super::super::node::Crossing;
        let mut simulator = Simulator::new();
        simulator.add_node(IONode::new().into())
        .add_node(Crossing::new().into())
        .add_node(Street::new().into());
        simulator.connect_with_street(0, 1, 2).unwrap();
        simulator.connect_with_street(1, 2, 3).unwrap();
        simulator.connect_with_street(2, 0, 4).unwrap();
    }
    
    #[test]
    fn test_simloop() {
        use super::Simulator;
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        let mut sim = Simulator::from_json(&json).unwrap();
        sim.max_iter(Some(1000)).delay(0);
        sim.simulation_loop().unwrap();
    }
}