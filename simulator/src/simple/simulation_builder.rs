use super::super::traits::NodeTrait;
use super::node_builder::{CrossingBuilder, IONodeBuilder, StreetBuilder};
use super::node_builder::NodeBuilderTrait;
use super::simulation::Simulator;
use std::fmt;
use std::error::Error;

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
#[derive(Debug)]
pub struct SimulatorBuilder{
    /// A list of all the nodes
    nodes: Vec<Box<dyn NodeBuilderTrait>>,
    max_iter: Option<usize>,
    cache: Option<Vec<Box<dyn NodeTrait>>>,
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
        let mut crossings: Vec<Box<dyn NodeBuilderTrait>> = Vec::new();    
        // generate all crossings
        for json_crossing in json_representation.crossings.iter() {
            // create nodes from json object
            let new_crossing: Box<dyn NodeBuilderTrait>  = match json_crossing.is_io_node {
                true => Box::new(IONodeBuilder::new()),
                false => Box::new(CrossingBuilder::new())
            };
            crossings.push(new_crossing);
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
                // Make sure the connection doesn't already exist
                if builder.nodes[i].get_connections().contains(connection_index) {
                    return Err(
                        Box::new(
                            JsonError("Attempt to create the same connection multiple times".to_string())
                        )
                    );
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
        // create a new street to connect them
        let mut new_street= StreetBuilder::new().lanes(lanes);
        new_street.connect(inode2);
        self.nodes.push(Box::new(new_street));
        let street_i = self.nodes.len() - 1;
        self.nodes[inode1].connect(street_i);
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
        let mut sim_nodes: Vec<Box<dyn NodeTrait>> = Vec::new();
        sim_nodes.reserve(self.nodes.len());
        // create the nodes
        self.nodes.iter().for_each(|n| {
            sim_nodes.push(
                n.build()
            )
        });
        // create the connections
        self.nodes.iter().enumerate().for_each(|(i, n)| {
            let starting_node = &mut sim_nodes[i];
            n.get_connections().iter().for_each(|c| {
                starting_node.connect(c);
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

    pub fn add_node(&mut self, node: Box<dyn NodeBuilderTrait>) -> &mut SimulatorBuilder {
        // the cache cannot be used if
        // the internals change
        self.drop_cache();
        self.nodes.push(node);
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
    pub fn iter_nodes(&self) -> std::slice::Iter<'_, Box<dyn NodeBuilderTrait>> {
        self.nodes.iter()
    }
    pub fn get_node(&self, i: usize) -> &Box<dyn NodeBuilderTrait> {
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
        use crate::simple::node_builder::{CrossingBuilder, IONodeBuilder, StreetBuilder};
        let mut simulator = SimulatorBuilder::new();
        simulator.add_node(Box::new(IONodeBuilder::new()))
        .add_node(Box::new(CrossingBuilder::new()))
        .add_node(Box::new(StreetBuilder::new()));
        simulator.connect_with_street(0, 1, 2).unwrap();
        simulator.connect_with_street(1, 2, 3).unwrap();
        simulator.connect_with_street(2, 0, 4).unwrap();
    }
}