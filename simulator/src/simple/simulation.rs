use serde::Deserialize;
use std::error::Error;
use std::fmt;
use super::node::Node;


/// A struct representing the street network
///
/// implementing functions for simulating the traffic
/// (moving cars, spawning new ones, moving pedestrians)
#[derive(Debug)]
pub struct Simulator {
    /// A list of all the crossings
    nodes: Vec<Node>,
}


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

impl Simulator {
    /// Create new node
    pub fn new() -> Simulator {
        Simulator {
            nodes: Vec::new()
        }
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
        let mut crossings: Vec<Node> = Vec::new();    
        // generate all crossings
        for json_crossing in json_representation.crossings.iter() {
            // create nodes from json object
            let new_crossing = match json_crossing.is_io_node {
                true => Node::IONode{connections: Vec::new()},
                false => Node::Crossing{connections: Vec::new()}
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
                match simulator.nodes.get(i).unwrap() {
                    Node::Crossing {connections } => {
                        if connections.contains(connection_index) {
                            return Err(
                                Box::new(
                                    JsonError("Attempt to create the same connection multiple times".to_string())
                                )
                            );
                        }
                    },
                    _ => {} 
                }
                simulator.connect_with_street(i, *connection_index, *lanes)?;
            }
        }
        Ok(simulator)
    }
    // /// Connects two nodes, ONE WAY ONLY
    // /// 
    // /// It is pretty inefficient, but relatively easy to use
    // /// # Examples
    // ///
    // /// ```
    // /// use super::*;
    // /// let sim = Simulator{
    // ///     vec![Node::Cros]
    // /// };
    // ///
    // /// ```
    // pub fn connect(&mut self, node1: &Node, node2: &Node) {

    // }
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
        {
            let new_street = Node::Street { connection: inode2, lanes};
            self.nodes.push(new_street);
        }
        let street_index = self.nodes.len() - 1;
        // get the starting node
        // TODO: Remember, remember, remember this way of doing things
        match &mut self.nodes[inode1] {
            Node::Crossing {connections} => {connections.push(street_index)},
            Node::IONode {connections} => {connections.push(street_index)},
            Node::Street {connection, lanes: _} => {*connection = street_index}
        }
        Ok(())
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
    use super::*;
    #[test]
    fn street_data_from_json() {
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        let data = Simulator::from_json(json).unwrap();
        println!("{:?}", &data);
    }
    
    #[test]
    fn connect_with_streets() {
        let mut simulator = Simulator::new();
    }
}