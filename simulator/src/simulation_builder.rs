use crate::node_builder::InOut;

use super::int_mut::IntMut;
use super::node::Node;
use super::node_builder::{CrossingBuilder, IONodeBuilder, NodeBuilder, StreetBuilder};
use super::node_builder::{Direction, NodeBuilderTrait};
use super::simulation::Simulator;
use std::error::Error;
use std::fmt::{self};

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
    crossings: Vec<JsonCrossing>,
}
/// Is raised when the conversion `JSON` -> `Simulator` fails
#[derive(Debug, Clone)]
pub struct JsonError(String);
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for JsonError {}

/// An error that is raised when the simulation fails to connect nodes
#[derive(Debug, Clone)]
pub struct ConnectionError {
    start: usize,
    end: usize,
    msg: Option<String>,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.msg {
            Some(msg) => write!(
                f,
                "ConnectionError: {} -> {} ({})",
                self.start, self.end, msg
            ),
            None => write!(f, "ConnectionError: {} -> {}", self.start, self.end),
        }
    }
}

impl Error for ConnectionError {}

///
#[derive(Debug, Clone)]
pub struct IndexError(String);
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
pub struct SimulatorBuilder {
    /// A list of all the nodes
    pub nodes: Vec<IntMut<NodeBuilder>>,
    max_iter: Option<usize>,
    cache: Option<Vec<IntMut<Node>>>,
    delay: u64,
    /// The id of the next node. This is necessary, as the length of the nodes
    /// vector is not always the id. (because nodes can be deleted as well)
    next_id: usize,
}

impl SimulatorBuilder {
    /// Create new node
    pub fn new() -> SimulatorBuilder {
        SimulatorBuilder {
            nodes: Vec::new(),
            max_iter: None,
            delay: 0,
            cache: None,
            next_id: 0,
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
        let mut crossings: Vec<IntMut<NodeBuilder>> = Vec::new();
        // generate all crossings
        for json_crossing in json_representation.crossings.iter() {
            // create nodes from json object
            let new_crossing = match json_crossing.is_io_node {
                true => NodeBuilder::IONode(IONodeBuilder::new()),
                false => NodeBuilder::Crossing(CrossingBuilder::new()),
            };
            crossings.push(IntMut::new(new_crossing));
        }
        let mut builder = SimulatorBuilder::new();
        builder.nodes = crossings;
        // save the number of inital nodes to later check if the json points
        // to existing nodes that are not streets
        let inital_nodes = builder.nodes.len();
        // connect the crossings with streets
        for (i, json_crossing) in json_representation.crossings.iter().enumerate() {
            // form all the connections defined in `JsonCrossing.connected`
            for (connection_index, _lanes) in json_crossing.connected.iter() {
                // check if `Crossing`/`IONode` the street ends in actually exists
                if *connection_index > inital_nodes {
                    return Err(Box::new(JsonError(
                        "Connection points to node that doesn't exist".to_string(),
                    )));
                }
                {
                    // Make sure the connection doesn't already exist
                    let node = &builder.nodes[i];
                    let connected = &builder.nodes[*connection_index];
                    if node.get().is_connected(&connected) {
                        return Err(Box::new(JsonError(
                            "Attempt to create the same connection multiple times".to_string(),
                        )));
                    }
                }
                todo!("Not implemented (anymore). Why would anyone use the python frontend!?");
                // builder.connect_with_street((i, InOut), *connection_index, *lanes)?;
            }
        }
        Ok(builder)
    }
    /// Connects two nodes, ONE WAY ONLY, adding a street in between
    pub fn connect_with_street(
        &mut self,
        node_info1: (usize, Direction),
        node_info2: (usize, Direction),
        lanes: u8,
    ) -> Result<&IntMut<NodeBuilder>, Box<dyn Error>> {
        let (idnode1, dir1) = node_info1;
        let (idnode2, dir2) = node_info2;
        // make sure the second nodes actually exist and get their indices
        let mut inode1: Option<usize> = None;
        let mut inode2: Option<usize> = None;
        for (i, node) in self.nodes.iter().enumerate() {
            if node.get().get_id() == idnode1 {
                inode1 = Some(i);
            } else if node.get().get_id() == idnode2 {
                inode2 = Some(i);
            }
            if inode1.is_some() && inode2.is_some() {
                break;
            }
        }
        if inode1.is_none() || inode2.is_none() {
            return Err(Box::new(IndexError("Node doesn't exist".to_string())));
        }
        let inode1 = inode1.unwrap();
        let inode2 = inode2.unwrap();

        let node1 = &self.nodes[inode1];
        let node2 = &self.nodes[inode2];
        // create a new street to connect them
        let mut new_street = StreetBuilder::new().with_lanes(lanes);
        new_street
            .connect(InOut::IN, node1)
            .connect(InOut::OUT, node2);
        new_street.set_id(self.next_id);
        self.next_id += 1;

        // wrap the street (this is how it is stored internally)
        let new_street = IntMut::new(NodeBuilder::Street(new_street));
        // add the connection the the street in the nodes
        match &mut *node1.get() {
            NodeBuilder::IONode(inner) => {
                inner.connect(InOut::OUT, &new_street);
            }
            NodeBuilder::Crossing(inner) => {
                inner.connect(dir1, InOut::OUT, &new_street).map_err(|er| {
                    Box::new(ConnectionError {
                        start: inode1,
                        end: inode2,
                        msg: Some(format!(
                            "Unable to connect OUT -> IN. (Failed at OUT): {}",
                            er
                        )),
                    })
                })?;
            }
            NodeBuilder::Street(_) => panic!("Can't connect street with street"),
        }
        match &mut *node2.get() {
            NodeBuilder::IONode(inner) => {
                inner.connect(InOut::OUT, &new_street);
            }
            NodeBuilder::Crossing(inner) => {
                inner.connect(dir2, InOut::IN, &new_street).map_err(|er| {
                    Box::new(ConnectionError {
                        start: inode1,
                        end: inode2,
                        msg: Some(format!(
                            "Unable to connect OUT -> IN. (Failed at IN): {}",
                            er
                        )),
                    })
                })?;
            }
            NodeBuilder::Street(_) => panic!("Can't connect street with street"),
        }
        println!("Connecting: {}->{}", inode1, inode2);
        self.nodes.push(new_street);
        Ok(self.nodes.last().unwrap())
    }

    /// Creates a new simulator from the templates
    pub fn build(&mut self) -> Simulator {
        if let Some(cache) = &self.cache {
            return Simulator {
                nodes: cache.clone(),
                max_iter: self.max_iter,
                delay: self.delay,
            };
        }
        let mut sim_nodes: Vec<IntMut<Node>> = Vec::new();
        sim_nodes.reserve(self.nodes.len());
        // create the nodes
        self.nodes
            .iter()
            .for_each(|n| sim_nodes.push(IntMut::new(n.get().build())));
        // create the connections
        self.nodes.iter().enumerate().for_each(|(i, start_node_arc)| {
            start_node_arc
                .get()
                .get_out_connections()
                .iter()
                .for_each(|c| {
                    // get strong reference to get the id
                    let end_node_builder_int_mut = &*c;
                    let end_node_builder = &*end_node_builder_int_mut;
                    let starting_node = &sim_nodes[i];
                    let end_node = &sim_nodes[end_node_builder.upgrade().get().get_id()];
                    let starting_node_unwrapped = &mut *starting_node.get();
                    // we will connect using the out connections and set the in connections
                    // at the same time
                    match &mut *start_node_arc.get() {
                        NodeBuilder::Street(_street_builder) => {
                            let street = match starting_node_unwrapped{
                                Node::Street(s) => s,
                                Node::IONode(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                Node::Crossing(_) => panic!("NodeBuilders and Nodes not in same position in list.")
                            };
                            // streets are only connected to one other node
                            // set out connection
                            street.connect(InOut::OUT, end_node);
                            // set in connection of target node
                            match &mut *end_node.get() {
                                Node::Street(target) => {
                                    target.connect(InOut::IN, &starting_node);
                                },
                                Node::IONode(_target) => {
                                    // doesn't have an in connection
                                },
                                Node::Crossing(target) => {
                                    let end_node_builder = end_node_builder.upgrade();
                                    let data = end_node_builder.get();
                                    let crossing_builder= match &*data {
                                        NodeBuilder::Street(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                        NodeBuilder::IONode(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                        NodeBuilder::Crossing(n) => n
                                    };
                                    // Here we need to keep the structure that is saved
                                    // in the CrossingConnections of the NodeBuilder

                                    // get direction of input
                                    let direction = crossing_builder.connections.get_direction_for_item(InOut::IN, start_node_arc)
                                        .expect("Street output is connected to Crossing, but the Crossing input is not connected to street.");
                                    target.connect(direction, InOut::IN, starting_node)
                                        .unwrap();
                                },
                            }

                        }
                        // We will work under the assumption that crossings and io_nodes can only
                        // be connected with streets and not directly between each other
                        NodeBuilder::IONode(_io_node_builder) => {
                            let io_node = match starting_node_unwrapped {
                                Node::Street(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                Node::IONode(n) => n,
                                Node::Crossing(_) => panic!("NodeBuilders and Nodes not in same position in list.")
                            };
                            io_node.connect(end_node);
                        },
                        NodeBuilder::Crossing(crossing_builder) => {
                            let crossing = match starting_node_unwrapped {
                                Node::Street(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                Node::IONode(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                Node::Crossing(n) => n
                            };
                            // TODO: Make more efficient
                            // 
                            // At the moment, the connection needs to be upgraded to get the direction
                            // This is pretty unnecessary, as the comparison can take place without it being upgraded
                            let direction = crossing_builder.connections.get_direction_for_item(InOut::OUT, &end_node_builder_int_mut.upgrade()
                                                                                                ).unwrap();
                            crossing.connect(direction, InOut::OUT, end_node).unwrap();
                            match &mut *end_node.get() {
                                Node::Street(street) => {street.connect(InOut::IN, starting_node);},
                                Node::IONode(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                                Node::Crossing(_) => panic!("NodeBuilders and Nodes not in same position in list."),
                            }
                        },
                    }
                });
        });
        self.cache = Some(sim_nodes.clone());
        Simulator {
            nodes: sim_nodes,
            max_iter: self.max_iter,
            delay: self.delay,
        }
    }
    /// Drops the internal node cache
    pub fn drop_cache(&mut self) {
        self.cache = None
    }

    /// adds a node to the Simulation and sets the correct id
    pub fn add_node(&mut self, mut node: NodeBuilder) -> &IntMut<NodeBuilder> {
        // the cache cannot be used if
        // the internals change
        self.drop_cache();
        // set the internal id. Is later used for calculating paths
        node.set_id(self.next_id);
        self.next_id += 1;
        let new_node_index = self.nodes.len();
        self.nodes.push(IntMut::new(node));
        &self.nodes[new_node_index]
    }
    /// an optional delay between each iteration
    pub fn with_delay(&mut self, value: u64) -> &mut SimulatorBuilder {
        self.delay = value;
        self
    }
    /// Makes the simulation stop after `value` iterations
    pub fn with_max_iter(&mut self, value: Option<usize>) -> &mut SimulatorBuilder {
        self.max_iter = value;
        self
    }
    /// Returns an iterator over all nodes
    pub fn iter_nodes(&self) -> std::slice::Iter<'_, IntMut<NodeBuilder>> {
        self.nodes.iter()
    }
    /// returns a reference to the node with id `i`
    pub fn get_node(&self, i: usize) -> &IntMut<NodeBuilder> {
        &self.nodes[i]
    }
    /// removes a node by it's id
    ///
    /// For IONodes and Crossings, the connected Streets are removed as well, for
    /// streets not, as this would cause recursion
    pub fn remove_node_and_connected_by_id(
        &mut self,
        id: usize,
    ) -> Result<Vec<IntMut<NodeBuilder>>, &'static str> {
        // get the index of the specified node
        let i = match self
            .nodes
            .iter()
            .enumerate()
            .find(|(i, n)| n.get().get_id() == id)
        {
            Some((i, _n)) => i,
            None => return Err("Specified node does not exist"),
        };
        self.remove_node(i, true)
    }
    /// removes a node using the internal node index
    ///
    /// This is used by the functions `remove_node_by_id` and `remove_node_by_ref`
    ///
    /// For IONodes and Crossings, the connected Streets are removed as well, for
    /// streets not, as this would cause recursion
    fn remove_node(
        &mut self,
        node_index: usize,
        remove_connections: bool,
    ) -> Result<Vec<IntMut<NodeBuilder>>, &'static str> {
        if node_index >= self.nodes.len() {
            return Err("Node index out of bounds");
        }
        // TODO: Replace with swap_remove for better performance (check if this
        // doesn't break anything)
        let node = self.nodes.remove(node_index);
        if !remove_connections {
            return Ok(vec![node]);
        }
        let mut removed_nodes = Vec::new();
        let mut to_remove = Vec::new();

        // remove connected nodes as well
        match &*node.get() {
            NodeBuilder::IONode(inner) => {
                let connections = inner.get_all_connections();

                // remove the connected streets as well
                // in addition, the street references need to be removed from
                // their connection as well

                to_remove = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(i, rnode)| {
                        // only retain nodes that are not connected
                        let remove = connections.iter().any(|c| c == *rnode);
                        // before the node is removed, remove the references to it from all
                        // the nodes that are connected to it, to avoid having dead references
                        if remove {
                            for connection in rnode.get().get_all_connections() {
                                if connection != node {
                                    connection
                                        .upgrade()
                                        .get()
                                        .remove_connection(&rnode.downgrade());
                                }
                            }
                        }

                        remove
                    })
                    .map(|(i, n)| i)
                    .collect();
            }
            NodeBuilder::Crossing(inner) => {
                let connections = inner.get_all_connections();

                for c in connections.iter() {}
                // remove the connected streets as well
                // in addition, the street references need to be removed from
                // their connection as well

                to_remove = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(i, rnode)| {
                        // only retain nodes that are not connected
                        let remove = connections.iter().any(|c| c == *rnode);
                        // println!("removing {}", node.get().get_id());
                        // before the node is removed, remove the references to it from all
                        // the nodes that are connected to it, to avoid having dead references
                        if remove {
                            println!("{}", rnode.get().get_id());
                            for connection in rnode.get().get_all_connections() {
                                if connection != node {
                                    connection
                                        .upgrade()
                                        .get()
                                        .remove_connection(&rnode.downgrade());
                                }
                            }
                        }

                        remove
                    })
                    .map(|(i, n)| i)
                    .collect();
            }
            NodeBuilder::Street(_) => {}
        }
        removed_nodes.push(node);
        // Make sure the elements that are the rightmost get removed first
        // this is CRUCIAL to ensure that the right elements are removed
        // (when removing, the index of all the elements on the right gets one
        // lower, making all the other saved indicis invalid)
        to_remove.sort();
        for index in to_remove.iter().rev() {
            removed_nodes.push(self.nodes.remove(*index));
        }
        return Ok(removed_nodes);
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
        use crate::node_builder::Direction;
        use crate::node_builder::{CrossingBuilder, IONodeBuilder, NodeBuilder};
        use crate::simulation_builder::SimulatorBuilder;
        let mut simulator = SimulatorBuilder::new();
        simulator.add_node(NodeBuilder::IONode(IONodeBuilder::new()));
        simulator.add_node(NodeBuilder::Crossing(CrossingBuilder::new()));
        simulator
            .connect_with_street((0, Direction::E), (1, Direction::W), 2)
            .unwrap();
        simulator
            .connect_with_street((1, Direction::S), (0, Direction::N), 3)
            .unwrap();
    }
}
