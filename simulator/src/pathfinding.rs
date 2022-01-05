use crate::node_builder::NodeBuilderTrait;
use crate::traits::{CarReport, Movable, NodeTrait};
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
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/// A car with a predefined path.
#[derive(Debug, Clone)]
pub struct PathAwareCar {
    speed: f32,
    path: Vec<usize>,
    time_spent: f32,
    dist_traversed: f32,
    path_len: f32,
    id: u32,
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

    fn get_report(&self) -> CarReport {
        CarReport {
            time_taken: self.time_spent,
            distance_traversed: self.dist_traversed,
            total_dist: self.path_len,
        }
    }

    fn new() -> Self {
        PathAwareCar {
            time_spent: 0.0,
            speed: 0.0,
            path: Vec::new(),
            id: 0,
            dist_traversed: 0.0,
            path_len: 0.0,
        }
    }

    fn update(&mut self, t: f32) {
        self.time_spent += t
    }
    fn set_path(&mut self, p: Vec<usize>) {
        self.path = p;
    }
    fn set_path_len(&mut self, len: f32) {
        self.path_len = len
    }

    fn decide_next(
        &self,
        connections: &Vec<WeakIntMut<Node<Self>>>,
        current_node: &IntMut<Node<Self>>,
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
                warn!("Path Empty");
                return Err(Box::new(PathError {
                    msg: "Path is empty, but next connection was requested.",
                    expected_node: None,
                    available_nodes: connection_ids,
                }));
            }
        };

        if !connection_ids.contains(to_return) {
            warn!("Requested connection not in connections of node");
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
        match &*next_node.upgrade().get() {
            Node::Street(_) => {
                // if the next node is a street, we can simply return it
                return Ok(Some(next_node.clone()));
            }
            Node::IONode(_) => {
                // if the next node is a IONode, we can simply return it
                return Ok(Some(next_node.clone()));
            }
            Node::Crossing(crossing) => {
                // if the next node is a crossing, we need to check wether the traffic light is configured in
                // such a way that we can drive onto the next street
                let dn = crossing.get_out_connections();
                // the "overnext" node onto which the car wants to drive
                let desired_overnext_node = dn.iter().find(|out_node| {
                    if out_node.upgrade().get().id() == overnext_node_id(&self.path) {
                        true
                    } else {
                        false
                    }
                });
                // if we can reach the "overnext" node (street), we can return it, else the car will not move
                if crossing.can_out_node_be_reached(
                    current_node,
                    &desired_overnext_node
                        .expect("for some reason the overnext node does not exist despite existing")
                        .upgrade(),
                ) {
                    return Ok(Some(next_node.clone()));
                } else {
                    warn!("Red Light");
                    return Ok(None);
                }
            }
        }
    }

    fn get_id(&self) -> u32 {
        self.id
    }

    fn set_id(&mut self, id: u32) {
        self.id = id
    }
    fn advance(&mut self) {
        match self.path.pop() {
            Some(_) => {}
            None => warn!("Could not remove last element while advancing to the next node"),
        }
    }
}

fn overnext_node_id(path: &Vec<usize>) -> usize {
    if path.len() >= 2 {
        return path[path.len() - 2];
    } else {
        panic!("tried to get ;overnext; node, but it does not exist for this path")
    }
}

/// this struct saved data of a connection that is important for caching / path finding
#[derive(Debug, Clone)]
struct IndexedConnection {
    pub id: usize,
    pub cost: u32,
}

/// A Data Structure representing the connections with indices to make
/// using path finding algorithms easier
#[derive(Debug)]
struct IndexedNodeNetwork {
    /// connections acvvvvvvvvvvvvvvvn bbbbbbbbbbbbbbbbbbbbbbbbbbbb (my cat)
    ///
    /// contains a list of connections for each given index, with the first element
    /// of the contained tuple being the id of the connection, and the second one
    /// being the cost of moving to the specified connection
    pub connections: HashMap<usize, Vec<IndexedConnection>>,
    pub node_lens: HashMap<usize, f32>,
    pub io_nodes: Vec<usize>,
    pub io_node_weights: Vec<f32>,
}

impl IndexedNodeNetwork {
    /// generates a new [IndexedNodeNetwork] from a list of [NodeBuilders](NodeBuilder)
    fn index_builder<Car: Movable>(&mut self, sbuilder: &SimulatorBuilder<Car>) {
        let nodes = &sbuilder.nodes;
        let mut connections: HashMap<usize, Vec<IndexedConnection>> =
            HashMap::with_capacity(nodes.len());
        let mut io_nodes: Vec<usize> = Vec::new();
        let mut node_lens = HashMap::with_capacity(nodes.len());
        let mut io_node_weights: Vec<f32> = Vec::new();
        println!("Started to index");
        nodes.iter().for_each(|node| {
            // TODO: Find a way to avoid using .get() 2 times
            let id = node.get().get_id();
            connections.insert(id, {
                // get the indices and weights of all connections
                node.get()
                    .get_out_connections()
                    .iter()
                    .map(|n| {
                        let node_upgraded = n.upgrade();
                        let c_node = node_upgraded.get();

                        node_lens.insert(id, c_node.get_node_dist());
                        IndexedConnection {
                            id: c_node.get_id(),
                            // funny weights calculation (dijkstra expects a cost as usize
                            // instead of the float weights we use)
                            cost: ((1.0 / c_node.get_weight()) * 100000.0) as u32,
                        }
                    })
                    .collect()
            });
            let inner_data = node.get();
            match *inner_data {
                NodeBuilder::IONode(_) => {
                    io_nodes.push(inner_data.get_id());
                    io_node_weights.push(inner_data.get_weight());
                    // trace!("doing magic node weight thingy");
                }
                _ => {}
            }
        });
        println!("Indexed");
        *self = IndexedNodeNetwork {
            connections,
            io_nodes,
            io_node_weights,
            node_lens,
        };
    }
    pub fn new() -> IndexedNodeNetwork {
        IndexedNodeNetwork {
            connections: HashMap::new(),
            node_lens: HashMap::new(),
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

/// Is raised when it is not possible to compute a path
#[derive(Debug)]
pub struct NoPathError {
    pub start: usize,
    pub end: usize,
}

impl Display for NoPathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to compute path between IONodes: {} and {}",
            self.start, self.end
        )
    }
}

impl Error for NoPathError {}

/// generates new movables with a given path
///
/// It provides a way for multiple Simulations to request new cars
/// without paths having to generate a new path each time. It caches
/// paths.
#[derive(Debug)]
pub struct MovableServer<Car = PathAwareCar>
where
    Car: Movable,
{
    // nodes: Vec<IntMut<NodeBuilder>>,
    indexed: IndexedNodeNetwork,
    cache: HashMap<(usize, usize), Car>,
    /// used to assign each car a unique number
    car_count: u32,
}

impl<Car: Movable> MovableServer<Car> {
    /// indexes and copies the given nodes and returns a new [MovableServer]
    ///
    /// it is important to note that this
    pub fn new() -> MovableServer {
        MovableServer {
            indexed: IndexedNodeNetwork::new(),
            cache: HashMap::new(),
            car_count: 0,
        }
    }
    /// index a simulation builder in the movable server so we can access it lateron
    pub fn register_simulator_builder(&mut self, nbuilder: &SimulatorBuilder) {
        self.indexed.index_builder(nbuilder);
    }
    /// generates a new movable for node with id `id`
    pub fn generate_movable(&mut self, id: usize) -> Result<Car, NoPathError> {
        // choose random IoNode to drive to
        // prevent start node from being the end node at the same time
        // trace!("IONode Weights (indexed) : {:?}", self.indexed.io_node_weights);
        let mut weights = self.indexed.io_node_weights.clone();
        let mut ids = self.indexed.io_nodes.clone();
        let self_index = ids
            .iter()
            .enumerate()
            .find(|(_i, nid)| id == **nid)
            .expect("Input id does not exist")
            .0;
        weights.remove(self_index);
        ids.remove(self_index);
        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        // you are the chosen one!
        let start_node = id; // self.indexed.io_nodes[index];
        let end_node = ids[dist.sample(&mut rng)];
        // println!("{}, {}", start_node, end_node);
        let cache_entry = self.cache.entry((start_node, end_node));
        if let Entry::Occupied(entry) = cache_entry {
            // even though the car is cached, it is still a new car
            //  therefor, the count has to be incremented to ensure the new car won't conflict
            //  with the car that was originally cached
            let mut car = entry.get().clone();
            car.set_id(self.car_count);
            self.car_count += 1;
            return Ok(car);
        } else {
            // weight needs to be 1/weights, because dijkstra takes cost and not weight of nodes
            let mut path = match dijkstra(
                &start_node,
                |p| {
                    let conn = &self.indexed.connections[p];
                    conn.iter()
                        .map(|iconn| (iconn.id, iconn.cost))
                        .collect::<Vec<(usize, u32)>>()
                },
                |i| *i == end_node,
            ) {
                Some((p, _)) => p,
                None => {
                    let perror = NoPathError {
                        start: start_node,
                        end: end_node,
                    };
                    trace!("{:?}", perror);
                    return Err(perror);
                }
            };
            let path_len: f32 = path.iter().map(|id| self.indexed.node_lens[id]).sum();
            // Reverse list of nodes to be able to pop off the last element
            path.reverse();
            // IONode is the first element
            // println!("Path: {:?}", path);
            path.pop();
            let mut car = Car::new(); // PathAwareCar { speed: 1.0, path, id: self.car_count };
            car.set_speed(1.0);
            car.set_path_len(path_len);
            car.set_path(path);
            car.set_id(self.car_count);
            self.car_count += 1;
            self.cache.insert((start_node, end_node), car.clone());
            return Ok(car);
        }
    }
}

mod tests {

    #[test]
    #[should_panic]
    fn generate_movable_test() {
        use crate::debug::build_grid_sim;
        use crate::pathfinding::MovableServer;
        use crate::pathfinding::PathAwareCar;
        let simbuilder = build_grid_sim(4);
        let mut test = MovableServer::<PathAwareCar>::new();
        test.register_simulator_builder(&simbuilder);
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
