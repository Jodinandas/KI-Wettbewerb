use crate::movable::MovableStatus;
use crate::movable::RandCar;
use crate::pathfinding::MovableServer;
use crate::pathfinding::PathAwareCar;
use crate::traits::Movable;
use crate::traits::NodeTrait;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};
use std::time::{Duration, SystemTime};
use std::{cmp, ptr, thread};

use super::int_mut::{IntMut, WeakIntMut};
use super::node::Node;

/// A struct representing the street network
///
/// implementing functions for simulating the traffic
/// (moving cars, spawning new ones, moving pedestrians)
///
/// The Street System is defined as as list of Nodes that
/// can hold traffic and people and move them. These Nodes
/// are connected to each other. Cars are spawned and destroyed
/// by so called IONodes
#[derive(Debug)]
pub struct Simulator<Car = PathAwareCar>
where Car: Movable
{
    /// A list of all the nodes.
    ///
    /// The nodes themselves save the index themselves
    pub nodes: Vec<IntMut<Node<Car>>>,
    /// The simulation can be set to stop after simulation
    /// a set amount of steps
    pub max_iter: Option<usize>,
    /// An optional delay between each iteration
    pub delay: u64,
}

/// Error is thrown when a node that should exist, doesn't exist anymore
#[derive(Debug)]
pub struct NodeDoesntExistError;
impl Error for NodeDoesntExistError {}
impl Display for NodeDoesntExistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tried to access Node that doesnt't exist (anymore).")
    }
}

/// The simulator, the top level struct that is instaniated to simulate traffic

impl<Car: Movable> Simulator<Car> {
    /// Update all nodes moving the cars and people to the next
    /// nodes
    pub fn update_all_nodes(&mut self, dt: f64) {
        for i in 0..self.nodes.len() {
            let node = &mut self.nodes[i].get();
            let mut cars_at_end = node.update_cars(dt);
            // TODO: Use something more efficient than cloning the whole Vec here
            let options = node.get_out_connections();
            for j in cars_at_end.len()..0 {
                let next: Result<Option<WeakIntMut<Node<Car>>>, Box<dyn Error>> =
                    cars_at_end[j].decide_next(&options);
                match next {
                    Err(_) => {
                        println!(
                            "Unable to decide next node for car with index {} at node {}",
                            j, i
                        );
                    }
                    Ok(next_node) => {
                        match next_node{
                            Some(nn) => {(nn
                                .try_upgrade()
                                .expect("Referenced connection does not exist"))
                                .get()
                                .add_car(cars_at_end.pop().unwrap())
                                },
                            None => {
                                // Nothing to do here. If car can not be moved, we will not move it
                            },
                        }
                    }     
                        
                }
            }
        }
    }

    /// Simulates until a stop condition is met
    pub fn simulation_loop(&mut self) -> Result<(), Box<dyn Error>> {
        let mut counter = 0;
        let mut iteration_compute_time;
        loop {
            let now = SystemTime::now();
            if let Some(max_iter) = self.max_iter {
                if counter > max_iter {
                    break;
                };
            }

            iteration_compute_time = now.elapsed()?.as_millis();
            // Convert the time to seconds and wait either as long as the
            // last iteration took if the iteration took longer than the
            // specified delay or update using the delay
            let dt = cmp::max(self.delay as u128, iteration_compute_time) as f64 / 1000.0;
            self.sim_iter(dt);

            counter += 1;
            // TODO: Could case the system to wait an unnecessary millisecond
            thread::sleep(Duration::from_millis(cmp::min(
                self.delay - iteration_compute_time as u64,
                0,
            )));
        }
        Ok(())
    }

    /// a single iteration
    pub fn sim_iter(&mut self, dt: f64) {
        // At the moment all nodes are updated
        self.update_all_nodes(dt);
    }

    /// returns status information for all of the cars in the simulation
    ///
    /// the key of the HashMap is the node index
    pub fn get_car_status(&self) -> HashMap<usize, Vec<MovableStatus>> {
        let mut mapped_node = HashMap::new();
        for n in self.nodes.iter() {
            let n = n.get();
            let car_status = n.get_car_status();
            if car_status.len() != 0 {
                mapped_node.insert(n.id(), car_status);
            }
        }
        mapped_node
    }
}

/// just returns the name of the of the type passed in
fn get_type_of<T>(_: &T) -> &'static str {
    std::any::type_name::<T>()
}

/// Display to make it easier to check the connections etc.
///
/// # Intended look
/// `
/// Simulator{
///     nodes: [
///         0: Crossing -> 1, 2, 3
///     ]
/// }
/// `
impl Display for Simulator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::from("Simulator {\n\tnodes: [\n");
        for (i, n) in self.nodes.iter().enumerate() {
            let name = get_type_of(&*n);
            s.push_str(&format!("\t\t{}: {} ->\t", i, name));
            for _connection in n.get().get_out_connections().iter() {
                // find the index
                let mut index = 0;
                for (i, node) in self.nodes.iter().enumerate() {
                    if ptr::eq(n, node) {
                        index = i;
                        break;
                    }
                }

                s.push_str(&format!("{}, ", &index));
            }
            s.push_str("\n")
        }
        s.push_str("\t]\n}");
        write!(f, "{}", s)
    }
}

mod tests {
    #[test]
    fn test_simloop() {
        use crate::pathfinding::PathAwareCar;
        use crate::pathfinding::MovableServer;
        use crate::datastructs::IntMut;
        use super::super::simulation_builder::SimulatorBuilder;
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        let mut sim_builder = SimulatorBuilder::<PathAwareCar>::from_json(&json).unwrap();
        let mut mv_server = MovableServer::<PathAwareCar>::new();
        mv_server.register_simulator_builder(&sim_builder);
        let mv_server = IntMut::new(mv_server);
        sim_builder.with_delay(1).with_max_iter(Some(1000));
        let mut sim = sim_builder.build(&mv_server);
        sim.simulation_loop().unwrap();
    }
}
