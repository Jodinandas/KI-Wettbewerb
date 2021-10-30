use std::collections::hash_map::Entry;
use crate::simple::node_builder::NodeBuilderTrait;
use crate::simple::node_builder;
use crate::traits::Movable;
use std::error::Error;
use rand::prelude::*;
use rand::distributions::WeightedIndex;
use pathfinding::directed::dijkstra::dijkstra;
use std::fmt::{Debug, Formatter, Display};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct PathAwareCar {
    speed: f32,
    path: Vec<usize>,
}

#[derive(Debug)]
struct PathError {
    msg: &'static str,
    expected_node: Option<usize>,
    available_nodes: Vec<usize>
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

    fn update(&mut self, t:f64) {
        panic!("Not yet implemented! Consider using decide_next() instead");
    }

    fn decide_next(&mut self, connections: &Vec<usize>) -> Result<usize, Box<dyn Error>> {
        // epische logik hier
        let to_return = match self.path.pop() {
            Some(value) => value,
            None => {return Err(Box::new(PathError {
                msg: "Path is empty, but next connection was requested.",
                expected_node: None,
                available_nodes: connections.clone()
            }))}
        };

        if !connections.contains(&to_return){
            return Err(Box::new(PathError{
                msg: "Requested connection not present in current available in node",
                expected_node: Some(to_return),
                available_nodes: connections.clone(),
            }))
        }
        Ok(to_return)
    }
}

/// generates new movables with a given path
struct MovableServer{
    nodes: Vec<Box<dyn NodeBuilderTrait>>,
    cache: HashMap<(usize, usize), PathAwareCar>,
}

impl MovableServer{
    fn new(nodes: Vec<Box<dyn NodeBuilderTrait>>) -> MovableServer{
        MovableServer {
            nodes,
            cache: HashMap::new(),
        }
    }
    fn generate_movable(&mut self, index: usize) -> PathAwareCar{
        // choose random IoNode to drive to
        let io_nodes: Vec<(usize, &Box<dyn NodeBuilderTrait>)> = self.nodes.iter().enumerate().filter(
            | (i, node) | {
                match node.get_node_type() {
                    node_builder::NodeBuilderType::IONode => {
                        *i != index
                    },
                    _ => {false}
                }
            }
        ).collect();
        let weights= io_nodes.iter()
            .map(| (_i,n) | { (*n).get_weight() });
        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        // you are the chosen one!
        let end_node_index = io_nodes[dist.sample(&mut rng)].0;
        let start_node_index = index;
        println!("{}, {}", start_node_index, end_node_index);
        let cache_entry = self.cache.entry((start_node_index, end_node_index));
        if let Entry::Occupied( entry ) = cache_entry{
            return entry.get().clone()
        }else{
            // weight needs to be 1/weights, because dijkstra takes cost and not weight of nodes
            let mut path = dijkstra(
                &start_node_index,
                |p| self.nodes[*p].get_connections().iter()
                    .map(| c_index | { (*c_index, ((1.0 / self.nodes[*c_index].get_weight()) * 100000.0) as usize) }),
                |i| *i == end_node_index
            ).expect("Unable to compute path").0;
            // Reverse list of nodes to be able to pop off the last element
            path.reverse();
            // IONode is the first element
            path.pop();
            let car = PathAwareCar{
                speed: 1.0,
                path,
            };
            self.cache.insert((start_node_index, end_node_index), car.clone());
            return car   
        }
    }
}


mod tests{
    use crate::simple::pathfinding::MovableServer;
    use crate::build_grid::build_grid_sim;

    #[test]
    fn generate_movable_test() {
        use crate::debug::build_grid_sim;
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