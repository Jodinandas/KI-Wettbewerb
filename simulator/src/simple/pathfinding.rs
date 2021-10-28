use crate::simple::node_builder::NodeBuilderTrait;
use crate::simple::node_builder;
use crate::traits::Movable;
use std::error::Error;
use rand::prelude::*;
use rand::distributions::WeightedIndex;
use dyn_clone::DynClone;
use pathfinding::directed::dijkstra::dijkstra;
use std::fmt::{Debug, Formatter, Display};

#[derive(Debug, Clone)]
struct PathAwareCar {
    speed: f32,
    path: Vec<usize>,
}

#[derive(Debug, Error)]
struct PathError {
    msg: &'static str,
    expected_node: usize,
    available_nodes: Vec<usize>
}

impl Display for PathError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "PathError: {}", self.msg)
    }
}


impl Movable for PathAwareCar {
    fn get_speed(&self) -> f32 {
        self.speed
    }

    fn set_speed(&mut self, s: f32) {
        self.speed = s
    }

    fn update(&mut self, t: f64) {
        // lol
    }

    fn decide_next(&mut self, connections: &Vec<usize>) -> Result<usize, Box<dyn Error>> {
        // epische logik hier
        let to_return = self.path.pop()?;
        if !connections.contains(&to_return){
            return Err(Box::new(PathError{
                msg: "Requested connection not present in current available in node",
                expected_node: to_return,
                available_nodes: connections.clone(),
            }))
        }
        Ok(to_return)
    }
}

/// generates new movables with a given path
struct MovableServer{
    nodes: Vec<Box<dyn NodeBuilderTrait>>,
}

impl MovableServer{
    fn new(nodes: Vec<Box<dyn NodeBuilderTrait>>) -> MovableServer{
        MovableServer {
            nodes
        }
    }
    fn generate_movable(&self, index: usize) -> PathAwareCar{
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
        let mut weights= io_nodes.iter()
            .map(| (i,n) | { (*n).get_weight() });
        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        // you are the chosen one!
        let end_node_index = io_nodes[dist.sample(&mut rng)].0;
        let start_node_index = index;
        let mut result = dijkstra(
            &start_node_index,
            |p| self.nodes[*p].get_connections().iter()
                .map(| c_index | { (*c_index, (self.nodes[*c_index].get_weight() * 100000.0) as usize) }),
            |i| *i == end_node_index
        ).expect("Unable to compute path").0;
        // Reverse list of nodes to be able to pop off the last element
        result.reverse();
        // IONode is the first element
        result.pop();
        PathAwareCar{
            speed: 1.0,
            path: result
        }
    }
}


mod tests{
    use crate::simple::pathfinding::MovableServer;
    use crate::build_grid::build_grid_sim;

    #[test]
    fn generate_movable_test() {
        use crate::debug::build_grid_sim;
        let simbuilder = build_grid_sim(500);
        let test = MovableServer::new(simbuilder.nodes);
        test.generate_movable(1);
    }
}