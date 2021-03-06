use crate::node::CostCalcParameters;
use crate::node_builder::InOut;
use crate::pathfinding::{MovableServer, PathAwareCar};
use crate::traits::{Movable, NodeTrait};

use super::int_mut::IntMut;
use super::node::Node;
use super::node_builder::{CrossingBuilder, IONodeBuilder, NodeBuilder, StreetBuilder};
use super::node_builder::{Direction, NodeBuilderTrait};
use super::simulation::Simulator;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self};

use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
struct JsonCrossingConnections {
    pub input: HashMap<Direction, usize>,
    pub output: HashMap<Direction, usize>
}

/// This is just used to deserialize the JSON File t    input: HashMap<Direction, u32>o
/// an object that can be conveniently used in
/// `StreetData::from_json`
///
#[derive(Debug, Deserialize, Serialize)]
struct JsonCrossing {
    pub connected: JsonCrossingConnections,
    pub id: usize,
    pub length: f32
}
#[derive(Debug, Deserialize, Serialize)]
struct JsonIONode {
    pub connected_in: Vec<usize>,
    pub connected_out: Vec<usize>,
    pub spawn_rate: f64,
    pub id: usize
}
#[derive(Debug, Deserialize, Serialize)]
struct JsonStreet {
    pub conn_in: Option<usize>,
    pub conn_out: Option<usize>,
    pub lanes: u8,
    pub length: f32,
    pub id: usize
}

#[derive(Debug, Serialize, Deserialize)]
enum JsonNode {
    Crossing(JsonCrossing),
    IONode(JsonIONode),
    Street(JsonStreet),
}

impl JsonNode {
    pub fn to_unfinished_builder(&self) -> NodeBuilder {
        match self {
            JsonNode::Crossing(crossing) => {
                let mut builder = CrossingBuilder::new()
                    .with_length(crossing.length);
                builder.set_id(crossing.id);
                NodeBuilder::Crossing(builder)
            },
            JsonNode::IONode(ionode) => {
                let mut ionodeb = IONodeBuilder::new();
                ionodeb.spawn_rate = ionode.spawn_rate;
                ionodeb.set_id(ionode.id);
                NodeBuilder::IONode(ionodeb)
            },
            JsonNode::Street(jstreet) => {
                let mut street = StreetBuilder::new()
                    .with_lanes(jstreet.lanes)
                    .with_length(jstreet.length);
                street.set_id(jstreet.id);
                // println!("Creating STreet with id: {}", street.get_id());
                NodeBuilder::Street(street)
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
/// Just for Deserialisation
struct JsonRepresentation {
    pub nodes: Vec<JsonNode>,
    pub next_id: usize,
    pub dt: f32,
    pub delay: u64
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
#[derive(Debug, Clone)]
pub struct SimulatorBuilder<Car = PathAwareCar>
where
    Car: Movable,
{
    /// A list of all the nodes
    pub nodes: Vec<IntMut<NodeBuilder>>,
    max_iter: Option<usize>,
    cache: Option<Vec<IntMut<Node<Car>>>>,
    /// public so it can be more easily changed in the front end
    pub delay: u64,
    /// The id of the next node. This is necessary, as the length of the nodes
    /// vector is not always the id. (because nodes can be deleted as well)
    next_id: usize,
    /// how much a simulation is advanced each step
    pub dt: f32,
    ///
    pub speed_to_co2: f32
}

impl<Car: Movable> SimulatorBuilder<Car> {
    /// Create new node
    pub fn new() -> SimulatorBuilder {
        SimulatorBuilder {
            nodes: Vec::new(),
            max_iter: None,
            delay: 0,
            cache: None,
            next_id: 0,
            dt: 0.1,
            speed_to_co2: 0.5,
        }
    }



 
    /// Connects two nodes, ONE WAY ONLY, adding a street in between
    pub fn connect_with_street(
        &mut self,
        node_info1: (usize, Direction),
        node_info2: (usize, Direction),
        lanes: u8,
        street_length: f32
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
        new_street.lane_length = street_length;
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
        // println!("Connecting: {}->{}", inode1, inode2);
        self.nodes.push(new_street);
        Ok(self.nodes.last().unwrap())
    }

    /// Creates a new simulator from the templates
    pub fn build(&mut self, mv_server: &MovableServer<Car>) -> Simulator<Car> {
        if let Some(cache) = &self.cache {
            return Simulator {
                nodes: cache.iter().map(|n| n.deep_copy()).collect(),
                max_iter: self.max_iter,
                delay: self.delay,
                dt: self.dt,
                calc_params: CostCalcParameters {
                    speed_to_co2: self.speed_to_co2
                },
                mv_server: mv_server.clone()
            };
        }
        // create the nodes
        let sim_nodes: Vec<IntMut<Node<Car>>> = self
            .nodes
            .iter()
            .map(|n| {
                let mut new_node = n.get().build();
                IntMut::new(new_node)
            })
            .collect();
        // create the connections
        self.nodes.iter().for_each(|start_node_arc| {
            let mut start_node = start_node_arc.get();
            let start_id = start_node.get_id();
            start_node
                .get_out_connections()
                .iter()
                .for_each(|c| {
                    // get strong reference to get the id
                    let end_node_builder_int_mut = &*c;
                    let end_node_builder = &*end_node_builder_int_mut;
                    // find the node with the correct id
                    let end_id = end_node_builder.upgrade().get().get_id();
                    let starting_node = sim_nodes.iter().find( | n | n.get().id() == start_id).unwrap();
                    let end_node = sim_nodes.iter().find( | n | n.get().id() == end_id).unwrap();
                    let starting_node_unwrapped = &mut *starting_node.get();
                    // we will connect using the out connections and set the in connections
                    // at the same time
                    match &mut *start_node {
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
                                        .expect(&format!("Street(id={}) output is connected to Crossing (id={}), but the Crossing input is not connected to street.", start_id, target.id));
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
        // self.cache = Some(sim_nodes.iter().map(|n| n.deep_copy()).collect());
        Simulator {
            nodes: sim_nodes,
            max_iter: self.max_iter,
            delay: self.delay,
            dt: self.dt,
            calc_params: CostCalcParameters {
                speed_to_co2: self.speed_to_co2,
            },
            mv_server: mv_server.clone(),
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
    pub fn with_delay(&mut self, value: u64) -> &mut Self {
        self.delay = value;
        self
    }
    /// sets the time step
    pub fn with_dt(&mut self, value: f32) -> &mut Self {
        self.dt = value;
        self
    }
    /// Makes the simulation stop after `value` iterations
    pub fn with_max_iter(&mut self, value: Option<usize>) -> &mut Self {
        self.max_iter = value;
        self
    }
    /// Returns an iterator over all nodes
    pub fn iter_nodes(&self) -> std::slice::Iter<'_, IntMut<NodeBuilder>> {
        self.nodes.iter()
    }
    /// returns a reference to the node with id `i`
    pub fn get_node(&self, i: usize) -> Option<&IntMut<NodeBuilder>> {
        self.nodes.iter().find( | n | {
            n.get().get_id() == i
        })
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
            .find(|(_i, n)| n.get().get_id() == id)
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
                    .filter(|(_i, rnode)| {
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
                    .map(|(i, _n)| i)
                    .collect();
            }
            NodeBuilder::Crossing(inner) => {
                let connections = inner.get_all_connections();

                // remove the connected streets as well
                // in addition, the street references need to be removed from
                // their connection as well

                to_remove = self
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_i, rnode)| {
                        // only retain nodes that are not connected
                        let remove = connections.iter().any(|c| c == *rnode);
                        // println!("removing {}", node.get().get_id());
                        // before the node is removed, remove the references to it from all
                        // the nodes that are connected to it, to avoid having dead references
                        if remove {
                            // println!("{}", rnode.get().get_id());
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
                    .map(|(i, _n)| i)
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

impl<'de> Deserialize<'de> for SimulatorBuilder {
    /// creates a `SimulatorBuilder` object from a `&str` formatted in a json-like way
    ///
    /// to see how the json must be formatted, look at the fields of
    /// `JsonCrossing` and `JsonRepresentation`
    ///
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        // Generate object holding all the data, still formatted in json way
        let json_representation: JsonRepresentation = JsonRepresentation::deserialize(deserializer)?;
        let mut nodes: Vec<IntMut<NodeBuilder>> = Vec::new();
        // generate all crossings
        for json_node in json_representation.nodes.iter() {
            nodes.push(IntMut::new(json_node.to_unfinished_builder()));
        }
        let mut builder = SimulatorBuilder::<PathAwareCar>::new();
        builder.with_delay(json_representation.delay)
            .with_dt(json_representation.dt);
        builder.next_id = json_representation.next_id;

        builder.nodes = nodes;
        // info!("nodes: {:#?}", builder.nodes);
        // connect the crossings with streets
        for (i, node) in json_representation.nodes.iter().enumerate() {
            match node {
                JsonNode::Crossing(jcrossing) => {
                        for (dir, n_id) in jcrossing.connected.input.iter() {
                            let node = builder.nodes.iter().find( | n | {
                                n.get().get_id() == *n_id
                            }).unwrap();
                            if let NodeBuilder::Crossing(crossing) = &mut *builder.nodes[i].get() {
                                crossing.connect(*dir, InOut::IN, node).unwrap();
                            } else {panic!()}
                        }
                        for (dir, n_id) in jcrossing.connected.output.iter() {
                            let node = builder.nodes.iter().find( | n | {
                                n.get().get_id() == *n_id
                            }).unwrap();
                            if let NodeBuilder::Crossing(crossing) = &mut *builder.nodes[i].get() {
                                crossing.connect(*dir, InOut::OUT, node).unwrap();
                            } else {panic!()}
                        }
                },
                JsonNode::IONode(jio_node) => {
                    for id_in in jio_node.connected_in.iter() {
                        let target = builder.get_node(*id_in).unwrap();
                        if let NodeBuilder::IONode(io_node) = &mut *builder.nodes[i].get() {
                            io_node.connect(InOut::IN, target);
                        } else {panic!()}
                    }
                    for id_out in jio_node.connected_out.iter() {
                        // info!("Parsing info for node with id: {}, conn_id: {}", jio_node.id, id_out);
                        let target = builder.nodes.iter().find( | n | n.get().get_id() == *id_out).unwrap();
                        if let NodeBuilder::IONode(io_node) = &mut *builder.nodes[i].get() {
                            io_node.connect(InOut::OUT, target);
                        } else {panic!()}
                    }
                },
                JsonNode::Street(jstreet) => {
                    if let Some(id_in) = jstreet.conn_in {
                        let target = builder.nodes.iter().find( | n | n.get().get_id() == id_in).unwrap();
                        if let NodeBuilder::Street(street) = &mut *builder.nodes[i].get() {
                            street.connect(InOut::IN, target);
                        } else {panic!()}
                    }
                    if let Some(id_out) = jstreet.conn_out {
                        let target = builder.nodes.iter().find( | n | n.get().get_id() == id_out).unwrap();
                        if let NodeBuilder::Street(street) = &mut *builder.nodes[i].get() {
                            street.connect(InOut::OUT, target);
                        } else {panic!()}
                    }
                },
            }
        }
        Ok(builder)
    }
}


impl Serialize for SimulatorBuilder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        let nodes = self.nodes.iter().map(  | n| {
            let node_inner = &*n.get();
            let id = node_inner.get_id();
            match node_inner {
                NodeBuilder::IONode(n) => {
                    JsonNode::IONode(
                        JsonIONode {
                            connected_in: n.connections_in.iter().map( | c | c.upgrade().get().get_id()).collect(),
                            connected_out: n.connections_out.iter().map( | c | c.upgrade().get().get_id()).collect(),
                            spawn_rate: n.spawn_rate,
                            id,
                        }
                    )
                },
                NodeBuilder::Crossing(n) => {
                    let json_conns = JsonCrossingConnections {
                        input: n.connections.input.iter().map( | (dir, c) | (*dir, c.upgrade().get().get_id()) ).collect(),
                        output : n.connections.output.iter().map( | (dir, c) | (*dir, c.upgrade().get().get_id()) ).collect(),
                    };
                    JsonNode::Crossing(
                        JsonCrossing {
                            connected: json_conns,
                            id,
                            length: n.length,
                        }
                    )
                },
                NodeBuilder::Street(n) => {
                    JsonNode::Street(
                        JsonStreet {
                            conn_in: n.conn_in.as_ref().map(| c | c.upgrade().get().get_id()),
                            conn_out: n.conn_out.as_ref().map(| c | c.upgrade().get().get_id()),
                            lanes: n.lanes,
                            length: n.lane_length,
                            id,
                        }
                    )
                },
            }
        }).collect::<Vec<JsonNode>>();
        let json_representation = JsonRepresentation {
            nodes,
            next_id: self.next_id,
            dt: self.dt,
            delay: self.delay,
        };
        json_representation.serialize(serializer)
    }
}

mod tests {
    #[test]
    fn connect_with_streets() {
        use crate::node_builder::Direction;
        use crate::node_builder::{CrossingBuilder, IONodeBuilder, NodeBuilder};
        use crate::pathfinding::PathAwareCar;
        use crate::simulation_builder::SimulatorBuilder;
        let mut simulator = SimulatorBuilder::<PathAwareCar>::new();
        simulator.add_node(NodeBuilder::IONode(IONodeBuilder::new()));
        simulator.add_node(NodeBuilder::Crossing(CrossingBuilder::new()));
        simulator
            .connect_with_street((0, Direction::E), (1, Direction::W), 2, 100.0)
            .unwrap();
        simulator
            .connect_with_street((1, Direction::S), (0, Direction::N), 3, 100.0)
            .unwrap();
    }
}
