use std::error::Error;
use std::fmt::{self, Display};
use std::{cmp, ptr, thread};
use std::time::{Duration, SystemTime};
use crate::traits::NodeTrait;
use crate::traits::Movable;


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
pub struct Simulator {
    /// A list of all the nodes.
    ///
    /// The nodes themselves save the index themselves
    pub nodes: Vec<Box<dyn NodeTrait>>,
    /// The simulation can be set to stop after simulation
    /// a set amount of steps
    pub max_iter: Option<usize>,
    /// An optional delay between each iteration
    pub delay: u64
}



/// The simulator, the top level struct that is instaniated to simulate traffic
/// 
/// # Examples
/// ## From JSON file
/// ```
/// use simulator::simple::simulation::Simulator;
/// let json: &str = r#"
/// {"crossings": [
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]},
///     {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]},
///     {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
/// let mut simulator = Simulator::from_json(json);
/// ```
impl Simulator {
    
    /// Update all nodes moving the cars and people to the next 
    /// nodes
    pub fn update_all_nodes(&mut self, dt: f64) {
        for i in 0..self.nodes.len() {
            let node = &mut self.nodes[i];
            let mut cars_at_end = node.update_cars(dt);
            // TODO: Use something more efficient than cloning the whole Vec here
            let options = node.get_connections().clone();
            for j in cars_at_end.len()..0 {
                let next_i: Result<usize, Box<dyn Error>> = cars_at_end[j].decide_next(&options);
                match next_i {
                    Err(_) => {
                        println!("Unable to decide next node for car with index {} at node {}", j, i);
                    },
                    Ok(next_node) => {
                        let node = &mut self.nodes[next_node];
                        node.add_car(cars_at_end.pop().unwrap())
                    }

                }
            }
        }
    }
    
    pub fn simulation_loop(&mut self) -> Result<(), Box<dyn Error>>{
        let mut counter = 0;
        let mut iteration_compute_time;
        loop {
            let now = SystemTime::now();
            if let Some(max_iter) = self.max_iter {
                if counter > max_iter {break};
            }
            


            iteration_compute_time = now.elapsed()?.as_millis();
            // Convert the time to seconds and wait either as long as the
            // last iteration took if the iteration took longer than the 
            // specified delay or update using the delay
            let dt = cmp::max(self.delay as u128, iteration_compute_time) as f64 / 1000.0;
            self.sim_iter(dt);
            
            counter += 1;
            // TODO: Could case the system to wait an unnecessary millisecond
            thread::sleep(Duration::from_millis(
                cmp::min(self.delay - iteration_compute_time as u64, 0)
            ));
        }
        Ok(())
    }

    /// a single iteration
    pub fn sim_iter(&mut self, dt: f64) {
        // At the moment all nodes are updated
        self.update_all_nodes(dt);
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
            s.push_str(
                &format!("\t\t{}: {} ->\t", i, name)
            );
            for _connection in (**n).get_connections().iter() {
                // find the index
                let mut index = 0;
                for (i, node) in self.nodes.iter().enumerate() {
                    if ptr::eq(n, node) {
                        index = i;
                        break;
                    }
                }
                
                s.push_str(
                    &format!("{}, ", &index)
                );
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
        use super::super::simulation_builder::SimulatorBuilder;
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        let mut sim_builder = SimulatorBuilder::from_json(&json).unwrap();
        sim_builder.delay(1).max_iter(Some(1000));
        let mut sim = sim_builder.build();
        sim.simulation_loop().unwrap();
    }
}