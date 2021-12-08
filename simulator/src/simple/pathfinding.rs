use crate::simple::node_builder::NodeBuilderTrait;
use crate::traits::{Movable, NodeTrait};
use pathfinding::directed::dijkstra::dijkstra;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Mutex;
use std::sync::{Arc, Weak};

use super::node::Node;
use super::node_builder::NodeBuilder;
use super::simulation::NodeDoesntExistError;

/// A car with a predefined path.
#[derive(Debug, Clone)]
pub struct PathAwareCar {
    speed: f32,
    path: Vec<usize>,
}

#[derive(Debug)]
struct PathError {
    msg: &'static str,
    expected_node: Option<usize>,
    available_nodes: Vec<usize>,
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PathError: {}", self.msg)
    }
}

impl Error for PathError {}

impl Movable for PathAwareCar {
    fn get_speed(&self) -> f32 {
        self.speed.clone()
    }

    fn set_speed(&mut self, s: f32) {
        self.speed = s
    }

    fn update(&mut self, _t: f64) {
        panic!("Not yet implemented! Consider using decide_next() instead");
    }

    fn decide_next(
        &mut self,
        connections: &Vec<Weak<Mutex<Node>>>,
    ) -> Result<Weak<Mutex<Node>>, Box<dyn Error>> {
        // upgrade references to be able to access the id field
        let mut connections_upgraded = Vec::with_capacity(connections.len());
        for c in connections.iter() {
            connections_upgraded.push(match c.upgrade() {
                Some(value) => value,
                None => return Err(Box::new(NodeDoesntExistError {})),
            })
        }
        let connection_ids = connections_upgraded
            .iter()
            .map(|n| (**n).lock().unwrap().id())
            .collect();

        // epische logik hier
        let to_return = match self.path.pop() {
            Some(value) => value,
            None => {
                return Err(Box::new(PathError {
                    msg: "Path is empty, but next connection was requested.",
                    expected_node: None,
                    available_nodes: connection_ids,
                }))
            }
        };

        if !connection_ids.contains(&to_return) {
            return Err(Box::new(PathError {
                msg: "Requested connection not present in current available in node",
                expected_node: Some(to_return),
                available_nodes: connection_ids.clone(),
            }));
        }

        let (index, _) = connection_ids
            .iter()
            .enumerate()
            .find(|(_, i)| **i == to_return)
            .unwrap();

        Ok(connections[index].clone())
    }
}

/// A Data Structure representing the connections with indices to make
/// using path finding algorithms easier
struct IndexedNodeNetwork {
    /// connections acvvvvvvvvvvvvvvvn bbbbbbbbbbbbbbbbbbbbbbbbbbbb (my cat)
    ///
    /// contains a list of connections for each given index, with the first element
    /// of the contained tuple being the index of the connection, and the second one
    /// being the cost of moving to the specified connection
    pub connections: Vec<Vec<(usize, usize)>>,
    pub io_nodes: Vec<usize>,
    pub io_node_weights: Vec<f32>,
}

impl IndexedNodeNetwork {
    /// generates a new [IndexedNodeNetwork] from a list of [NodeBuilders](NodeBuilder)
    fn new(nodes: &Vec<Arc<Mutex<NodeBuilder>>>) -> IndexedNodeNetwork {
        let mut connections: Vec<Vec<(usize, usize)>> = Vec::with_capacity(nodes.len());
        let mut io_nodes: Vec<usize> = Vec::new();
        let mut io_node_weights: Vec<f32> = Vec::new();
        nodes.iter().for_each(|n| {
            let node = n.lock().unwrap();
            connections.push({
                // get the indices and weights of all connections
                node.get_connections()
                    .iter()
                    .map(|n| {
                        let arc_c_node = n.upgrade().unwrap();
                        let c_node = arc_c_node.lock().unwrap();
                        (
                            c_node.get_id(),
                            // funny weights calculation (dijkstra expects a cost as usize
                            // instead of the float weights we use)
                            ((1.0 / c_node.get_weight()) * 100000.0) as usize,
                        )
                    })
                    .collect()
            });
            match *node {
                NodeBuilder::IONode(_) => {
                    io_nodes.push(node.get_id());
                    io_node_weights.push(node.get_weight())
                }
                _ => {}
            }
        });
        IndexedNodeNetwork {
            connections,
            io_nodes,
            io_node_weights,
        }
    }
    /// returns all connections apart from the one specified by the index
    fn all_except(&self, i: usize) -> Vec<usize> {
        (0..(self.connections.len() - 1))
            .filter(|n| *n != i)
            .collect()
    }
}

/// generates new movables with a given path
///
/// It provides a way for multiple Simulations to request new cars
/// without paths having to generate a new path each time. It caches
/// paths.
struct MovableServer {
    nodes: Vec<Arc<Mutex<NodeBuilder>>>,
    indexed: IndexedNodeNetwork,
    cache: HashMap<(usize, usize), PathAwareCar>,
}

impl MovableServer {
    /// indexes and copies the given nodes and returns a new [MovableServer]
    ///
    /// it is important to note that this
    fn new(nodes: Vec<Arc<Mutex<NodeBuilder>>>) -> MovableServer {
        MovableServer {
            indexed: IndexedNodeNetwork::new(&nodes),
            nodes,
            cache: HashMap::new(),
        }
    }
    fn generate_movable(&mut self, index: usize) -> PathAwareCar {
        // choose random IoNode to drive to
        // prevent start node from being the end node at the same time
        let dist = WeightedIndex::new(self.indexed.io_node_weights.clone()).unwrap();
        let mut rng = thread_rng();
        // you are the chosen one!
        let start_node = self.indexed.io_nodes[index];
        let end_node = dist.sample(&mut rng);
        println!("{}, {}", start_node, end_node);
        let cache_entry = self.cache.entry((start_node, end_node));
        if let Entry::Occupied(entry) = cache_entry {
            return entry.get().clone();
        } else {
            // weight needs to be 1/weights, because dijkstra takes cost and not weight of nodes
            let mut path = dijkstra(
                &start_node,
                |p| self.indexed.connections[*p].clone(),
                |i| *i == end_node,
            )
            .expect("Unable to compute path")
            .0;
            // Reverse list of nodes to be able to pop off the last element
            path.reverse();
            // IONode is the first element
            path.pop();
            let car = PathAwareCar { speed: 1.0, path };
            self.cache.insert((start_node, end_node), car.clone());
            return car;
        }
    }
}

mod tests {
    #[test]
    #[should_panic]
    fn generate_movable_test() {
        use crate::debug::build_grid_sim;
        use crate::simple::pathfinding::MovableServer;
        let simbuilder = build_grid_sim(4);
        let mut test = MovableServer::new(simbuilder.nodes);
        println!("{:?}", test.generate_movable(4));
        println!("{:?}", test.generate_movable(4));
        println!("{:?}", test.generate_movable(4));
        println!("{:?}", test.generate_movable(4));
        println!("{:?}", test.generate_movable(4));
        println!("{:?}", test.generate_movable(4));
        println!("{:?}", test.generate_movable(4));
        println!("lol");
        println!("{:?}", test.cache);
        panic!("Not yet implemented properly. This test only serves as an example.")
    }
}
