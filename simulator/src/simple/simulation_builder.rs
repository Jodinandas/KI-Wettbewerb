use super::super::traits::NodeTrait;
use super::node::{self, IONode, Node};
use super::node_builder::{CrossingBuilder, IONodeBuilder, NodeBuilder, StreetBuilder};
use super::node_builder::NodeBuilderTrait;
use super::simulation::Simulator;
use std::sync::Mutex;
use std::fmt;
use std::error::Error;
use std::sync::Arc;

use serde::Deserialize;

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

/// A struct for creating simulators
/// 
/// To seperate simulation creation from actual simulation logic,
/// the builder pattern is used. This enables additional optimisations,
/// as things like nodes can be cached
///
/// # Examples
/// ## From JSON file
/// ```
/// use simulator::simple::simulation_builder::SimulatorBuilder;
/// let json: &str = r#"
/// {"crossings": [
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]},
///     {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
/// let mut simulator = SimulatorBuilder::from_json(json);
/// ```
#[derive(Debug)]
pub struct SimulatorBuilder{
    /// A list of all the nodes
    pub nodes: Vec<Arc<Mutex<NodeBuilder>>>,
    max_iter: Option<usize>,
    cache: Option<Vec<Arc<Mutex<Node>>>>,
    delay: u64
}

impl SimulatorBuilder {
    /// Create new node
    pub fn new() -> SimulatorBuilder {
        SimulatorBuilder {
            nodes: Vec::new(),
            max_iter: None,
            delay: 0,
            cache: None
        }
    }
    /// creates a `Simulator` object from a `&str` formatted in a json-like way
    ///
    /// to see how the json must be formatted, look at the fields of
    /// `JsonCrossing` and `JsonRepresentation`
    /// 
    /// # NOTE:
    /// This function is deprecated, as the old python frontend
    /// is being replaced by a new frontend 
    pub fn from_json(json: &str) -> Result<SimulatorBuilder, Box<dyn Error>> {
        // Generate object holding all the data, still formatted in json way
        let json_representation: JsonRepresentation = serde_json::from_str(json)?;
        let mut crossings: Vec<Arc<Mutex<NodeBuilder>>> = Vec::new();    
        // generate all crossings
        for json_crossing in json_representation.crossings.iter() {
            // create nodes from json object
            let new_crossing  = match json_crossing.is_io_node {
                true => NodeBuilder::IONode(IONodeBuilder::new()),
                false => NodeBuilder::Crossing(CrossingBuilder::new())
            };
            crossings.push(Arc::new(Mutex::new(new_crossing)));
        }
        let mut builder = SimulatorBuilder::new(); 
        builder.nodes = crossings;
        // save the number of inital nodes to later check if the json points
        // to existing nodes that are not streets
        let inital_nodes = builder.nodes.len();
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
                {
                    // Make sure the connection doesn't already exist
                    let node = &builder.nodes[i].lock().unwrap();
                    let connected = &builder.nodes[*connection_index];
                    if node.is_connected(&connected) {
                        return Err(
                            Box::new(
                                JsonError("Attempt to create the same connection multiple times".to_string())
                            )
                        );
                    }
                }
                builder.connect_with_street(i, *connection_index, *lanes)?;
            }
        }
        Ok(builder)
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
        let mut node2 = &self.nodes[inode2];
        // create a new street to connect them
        let mut new_street= StreetBuilder::new().lanes(lanes);
        new_street.connect(node2);
        self.nodes.push(Arc::new(Mutex::new(NodeBuilder::Street(new_street))));
        // self.nodes.push(Arc::new(Mutex::new(NodeBuilder::Street(new_street))));
        let street_i = self.nodes.len() - 1;
        let mut node1 = &self.nodes[inode1];
        node1.lock().unwrap().connect(&self.nodes[street_i]);
        Ok(())
    }

    /// Creates a new simulator from the templates
    pub fn build(&mut self) -> Simulator {
        if let Some(cache) = &self.cache {
            return Simulator {
                nodes: cache.clone(),
                max_iter: self.max_iter,
                delay: self.delay
            }
        }
        let mut sim_nodes: Vec<Arc<Mutex<Node>>> = Vec::new();
        sim_nodes.reserve(self.nodes.len());
        // create the nodes
        self.nodes.iter().for_each(|n| {
            sim_nodes.push(
                Arc::new(Mutex::new((**n).lock().unwrap().build()))
            )
        });
        // create the connections
        self.nodes.iter().enumerate().for_each(|(i, n)| {
            let mut starting_node = sim_nodes[i].lock().unwrap();
            (**n).lock().unwrap().get_connections().iter().for_each(|c| {
                // get strong reference to get the id
                let arc_node = c.upgrade().unwrap();
                let node = arc_node.lock().unwrap();
                starting_node.connect(&sim_nodes[node.id()]);
            });
        });
        self.cache = Some(sim_nodes.clone());
        Simulator {
            nodes: sim_nodes,
            max_iter: self.max_iter,
            delay: self.delay
        }
    }
    /// Drops the internal node cache
    pub fn drop_cache(&mut self) {
        self.cache = None
    }

    pub fn add_node(&mut self, mut node: NodeBuilder) -> &mut SimulatorBuilder {
        // the cache cannot be used if
        // the internals change
        self.drop_cache();
        // set the internal id. Is later used for calculating paths
        node.set_id(self.nodes.len());
        self.nodes.push(Arc::new(Mutex::new(node)));
        self
    }
    pub fn delay(&mut self, value: u64) ->  &mut SimulatorBuilder{
        self.delay = value;
        self
    }
    pub fn max_iter(&mut self, value: Option<usize>) ->  &mut SimulatorBuilder {
        self.max_iter = value;
        self
    }
    pub fn iter_nodes(&self) -> std::slice::Iter<'_, Arc<Mutex<NodeBuilder>>> {
        self.nodes.iter()
    }
    pub fn get_node(&self, i: usize) -> &Arc<Mutex<NodeBuilder>> {
        &self.nodes[i]
    }
}

mod tests {


    #[test]
    fn simulation_builder_from_json() {
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        let data = super::SimulatorBuilder::from_json(json).unwrap();
        println!("{:?}", &data);
    }
    
    #[test]
    fn connect_with_streets() {
        use crate::simple::simulation_builder::SimulatorBuilder;
        use crate::simple::node_builder::{CrossingBuilder, IONodeBuilder, StreetBuilder, NodeBuilder};
        let mut simulator = SimulatorBuilder::new();
        simulator.add_node(NodeBuilder::IONode(IONodeBuilder::new()))
        .add_node(NodeBuilder::Crossing(CrossingBuilder::new()))
        .add_node(NodeBuilder::Street(StreetBuilder::new()));
        simulator.connect_with_street(0, 1, 2).unwrap();
        simulator.connect_with_street(1, 2, 3).unwrap();
        simulator.connect_with_street(2, 0, 4).unwrap();
    }
}