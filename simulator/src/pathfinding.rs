use crate::node_builder::NodeBuilderTrait;
use crate::traits::{Movable, NodeTrait};
use crate::SimulatorBuilder;
use pathfinding::directed::dijkstra::dijkstra;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use super::int_mut::{IntMut, WeakIntMut};
use super::node::Node;
use super::node_builder::NodeBuilder;
use super::simulation::NodeDoesntExistError;

/// A car with a predefined path.
#[derive(Debug, Clone)]
pub struct PathAwareCar {
    speed: f32,
    path: Vec<usize>,
    id: u32
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
        connections: &Vec<WeakIntMut<Node<Self>>>,
    ) -> Result<Option<WeakIntMut<Node<Self>>>, Box<dyn Error>> {
        // upgrade references to be able to access the id field
        let mut connections_upgraded = Vec::with_capacity(connections.len());
        for c in connections.iter() {
            connections_upgraded.push(match c.try_upgrade() {
                Some(value) => value,
                None => return Err(Box::new(NodeDoesntExistError {})),
            })
        }
        let connection_ids = connections_upgraded.iter().map(|n| n.get().id()).collect();

        // epische logik hier
        let to_return = match self.path.last() {
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
                expected_node: Some(*to_return),
                available_nodes: connection_ids.clone(),
            }));
        }

        let (index, _) = connection_ids
            .iter()
            .enumerate()
            .find(|(_, i)| **i == *to_return)
            .unwrap();

        // the next node onto which a car wants to progress
        let next_node = &connections[index];
        match &*next_node.upgrade().get(){
            Node::Street(_) => {
                // if the next node is a street, we can simply return it
                self.path.pop().expect("Could not remove last element while trying to get onto Street");
                return Ok(Some(next_node.clone()))
            },
            Node::IONode(_) => {
                // if the next node is a IONode, we can simply return it
                self.path.pop().expect("Could not remove last element while trying to get onto IONode");
                return Ok(Some(next_node.clone()))
            },
            Node::Crossing(crossing) => {
                // if the next node is a crossing, we need to check wether the traffic light is configured in
                // such a way that we can drive onto the next street
                let dn = crossing.get_out_connections();
                // the "overnext" node onto which the car wants to drive
                let desired_overnext_node = dn.iter().find(
                    | out_node | {
                        if out_node.upgrade().get().id() == overnext_node_id(&self.path){
                            true
                        } else {
                            false
                        }
                    });
                // if we can reach the "overnext" node (street), we can return it, else the car will not move
                if crossing.can_out_node_be_reached(&*desired_overnext_node.expect("for some reason the overnext node does not exist despite existing").upgrade().get()){
                    self.path.pop().expect("This should really not have happened because overnext_node_id worked");
                    return Ok(Some(next_node.clone()))
                } else {
                    return Ok(None)
                }
            },
        }
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id
    }
}

fn overnext_node_id(path: &Vec<usize>) -> usize{
    if path.len() >= 2{
        return path[path.len()-2]
    } else {
        panic!("tried to get ;overnext; node, but it does not exist for this path")
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
    fn index_builder(&mut self, sbuilder: &SimulatorBuilder) -> IndexedNodeNetwork {
        let nodes = &sbuilder.nodes;
        let mut connections: Vec<Vec<(usize, usize)>> = Vec::with_capacity(nodes.len());
        let mut io_nodes: Vec<usize> = Vec::new();
        let mut io_node_weights: Vec<f32> = Vec::new();
        nodes.iter().for_each(|node| {
            connections.push({
                // get the indices and weights of all connections
                node.get()
                    .get_out_connections()
                    .iter()
                    .map(|n| {
                        let node_upgraded = n.upgrade();
                        let c_node = node_upgraded.get();
                        (
                            c_node.get_id(),
                            // funny weights calculation (dijkstra expects a cost as usize
                            // instead of the float weights we use)
                            ((1.0 / c_node.get_weight()) * 100000.0) as usize,
                        )
                    })
                    .collect()
            });
            let inner_data = node.get();
            match *inner_data {
                NodeBuilder::IONode(_) => {
                    io_nodes.push(inner_data.get_id());
                    io_node_weights.push(inner_data.get_weight())
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
    pub fn new() -> IndexedNodeNetwork {
        IndexedNodeNetwork {
            connections: Vec::new(),
            io_nodes: Vec::new(),
            io_node_weights: Vec::new(),
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
pub struct MovableServer {
    // nodes: Vec<IntMut<NodeBuilder>>,
    indexed: IndexedNodeNetwork,
    cache: HashMap<(usize, usize), PathAwareCar>,
    /// used to assign each car a unique number
    car_count: u32
}

impl MovableServer {
    /// indexes and copies the given nodes and returns a new [MovableServer]
    ///
    /// it is important to note that this
    pub fn new() -> MovableServer {
        MovableServer {
            indexed: IndexedNodeNetwork::new(),
            cache: HashMap::new(),
            car_count: 0
        }
    }
    /// generates a new movable for node with index `index`
    pub fn generate_movable(&mut self, index: usize) -> PathAwareCar {
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
            let car = PathAwareCar { speed: 1.0, path, id: self.car_count };
            self.cache.insert((start_node, end_node), car.clone());
            return car;
        }
    }
    /// index a simulation builder in the movable server so we can access it lateron
    pub fn register_simulation_buider(&mut self, nbuilder: &SimulatorBuilder) {
        self.indexed.index_builder(nbuilder);
    }
}

mod tests {
    #[test]
    #[should_panic]
    fn generate_movable_test() {
        use crate::debug::build_grid_sim;
        use crate::pathfinding::MovableServer;
        let simbuilder = build_grid_sim(4);
        let mut test = MovableServer::new();
        test.register_simulation_buider(&simbuilder);
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
