use crate::movable::MovableStatus;
use crate::movable::RandCar;
use crate::node::CostCalcParameters;
use crate::pathfinding::MovableServer;
use crate::pathfinding::PathAwareCar;
use crate::traits::CarReport;
use crate::traits::Movable;
use crate::traits::NodeTrait;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{self, Display};
use std::time::{Duration, SystemTime};
use std::{cmp, ptr, thread};

use super::int_mut::{IntMut, WeakIntMut};
use super::node::Node;
use art_int::LayerTopology;
use rand::prelude::ThreadRng;
use tracing::event;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use rand::thread_rng;

/// Error is thrown when a node that should exist, doesn't exist anymore
#[derive(Debug)]
pub struct NodeDoesntExistError;
impl Error for NodeDoesntExistError {}
impl Display for NodeDoesntExistError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tried to access Node that doesnt't exist (anymore).")
    }
}

/// source: https://www.econologie.de/Emissions-co2-Liter-Kraftstoff-Benzin-oder-Diesel-gpl/
pub fn fuel_to_tonnesco2(liters: f32) -> f32 {
    2.6 * liters / 1000.0
}

/// source of function : https://theconversation.com/climate-explained-does-your-driving-speed-make-any-difference-to-your-cars-emissions-140246
/// 
/// We modelled the function displayed in the graph using geogebra 
/// 
/// speed is given in km/h
/// 
/// the result is in litres
pub fn average_speed_to_fuel(speed: f64) -> f64 {
    - 0.000000000138841 * speed.powf(6.0)
    + 0.000000049189085 * speed.powf(5.0)
    - 0.000006513465616 * speed.powf(4.0)
    + 0.000375629640659 * speed.powf(3.0)
    - 0.005677092214215 * speed.powf(2.0)
    - 0.300161202539628 * speed.powf(1.0)
    + 15.954046194749404
}

/// calculates the cost of a car
pub fn calculate_cost(report: CarReport, params: &CostCalcParameters) -> [f64; 2] {
    // is in m/s
    let average_speed = report.distance_traversed / report.time_taken;
    // distance that the car has yet to traverse
    let dist_remaining = report.total_dist - report.distance_traversed;
    let dist_penalty = dist_remaining.powf(2.0);
    // to km/h
    let average_speed = average_speed * 3.6;
    let fuel_consumption = average_speed_to_fuel(average_speed.into());
    let tonnes_co2 = fuel_to_tonnesco2(fuel_consumption as f32);
    // lerp
    [
        average_speed as f64 * (1.0 - params.speed_to_co2) as f64 + tonnes_co2 as f64* params.speed_to_co2 as f64+ dist_penalty as f64,
        tonnes_co2 as f64
    ]
}

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
where
    Car: Movable,
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
    /// how much the simulation is advanced each step
    pub dt: f32,
    /// The parameters used for cost calculation
    pub calc_params: CostCalcParameters,
    /// Movables servlsaöe
    pub mv_server: MovableServer<Car>
}

/// The simulator, the top level struct that is instaniated to simulate traffic
impl<Car: Movable> Simulator<Car> {
    /// Update all nodes moving the cars and people to the next
    /// nodes
    #[tracing::instrument(skip(self))]
    pub fn update_all_nodes(&mut self, dt: f64) {
        let mut rng = ThreadRng::default();
        for i in 0..self.nodes.len() {
            let node = &self.nodes[i];
            let options = node.get().get_out_connections();
            let mut cars_at_end = node.get().update_cars(dt, &mut self.mv_server, &mut rng);
            // make sure that the rightmost elements get removed first to avoid
            // the indices becoming invalid
            cars_at_end.sort();
            // cars_at_end.reverse();
            // TODO: Use something more efficient than cloning the whole Vec here
            for j in (0..cars_at_end.len()).rev() {
                let next: Result<Option<WeakIntMut<Node<Car>>>, Box<dyn Error>> = node
                    .get()
                    .get_car_by_index(cars_at_end[j])
                    .decide_next(&options, node);
                match next {
                    Err(err) => {
                        warn!(
                            "Unable to decide next node for car with index {} at node {}. Error: {}",
                            j, i, err
                        );
                    }
                    Ok(next_node) => {
                        match next_node {
                            Some(nn) => {
                                let mut car = node.get().remove_car(cars_at_end[j]);
                                car.advance();
                                nn.upgrade()
                                    .get()
                                    .add_car(car);
                                // println!("{:?}", nn.try_upgrade().expect("asdof").get())
                            }
                            None => {
                                // Nothing to do here. If car can not be moved, we will not move it
                                debug!("aösldk")
                            }
                        }
                    }
                }
            }
        }
    }
    /// resets all cars
    pub fn reset_cars(&mut self) {
        self.nodes.iter().for_each(| n| {
            n.get().reset_cars();
        });
    }
    /// used the output from the genetic algorithm to set the neural networks
    pub fn set_neural_networks(&mut self, mut nns: Vec<art_int::Network>) {
        nns.reverse();
        self.nodes.iter_mut().for_each(|n| match &mut *n.get() {
            Node::Crossing(crossing) => crossing.set_neural_network(
                nns.pop()
                    .expect("Cannot set neural network because there are too few nns as input"),
            ),
            _ => {}
        });
    }

    /// returns a copy of all nns in the simulation
    pub fn get_all_neural_networks(&self) -> Vec<art_int::Network> {
        let mut nns = Vec::new();
        self.nodes.iter().for_each(|n| {
            match &*n.get() {
                Node::Crossing(crossing) => {
                    if let Some(nn) = &crossing.nn {
                        nns.push(nn.clone());
                    } else {
                        warn!("Removing all neural networks but crossing doesn't have a neural network")
                    }
                }
                _ => {}
            }
        });
        nns
    }

    /// returns all neural networks in the simulation and removes them from the crossings
    ///
    /// the nns of crossings that are first in the list of nodes are first
    pub fn remove_all_neural_networks(&mut self) -> Vec<art_int::Network> {
        let mut nns = Vec::new();
        self.nodes.iter_mut().for_each(|n| match &mut *n.get() {
            Node::Crossing(crossing) => {
                if let Ok(nn) = crossing.remove_neural_network() {
                    nns.push(nn)
                } else {
                    warn!("Removing all neural networks but crossing doesn't have a neural network")
                }
            }
            _ => {}
        });
        nns
    }

     
    /// initialises all NNs with random values
    pub fn init_neural_networks_random(&mut self, topology: &[LayerTopology]) {
        let mut rng = thread_rng();
        self.nodes.iter_mut().for_each(|n| match &mut *n.get() {
            Node::Crossing(crossing) => {
                crossing.set_neural_network(art_int::Network::random(&mut rng, topology))
            }
            _ => {}
        });
    }
    
    /// returns the total cost of all the cars in the simulation 
    /// (including those that have already been destroyed)
    #[tracing::instrument(skip(self))]
    pub fn calculate_sim_cost(&self) -> [f64; 2] {
        self.nodes
            .iter()
            .map(|n| match &*n.get() {
                Node::Street(s) => s
                    .lanes
                    .iter()
                    .map(|l| l.calculate_cost_of_movables(&self.calc_params))
                    .fold([0.0, 0.0], | [sumcost, sumco2], [cost, co2] | {
                        [
                            sumcost + cost,
                            sumco2 + co2
                        ]
                    }),
                Node::IONode(n) => n.total_cost,
                Node::Crossing(c) => c.car_lane.calculate_cost_of_movables(&self.calc_params),
            })
            .fold([0.0, 0.0], | [sumcost, sumco2], [cost, co2] | {
                [
                    sumcost + cost,
                    sumco2 + co2
                ]
            })
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
            // let dt = cmp::max(self.delay as u128, iteration_compute_time) as f64 / 1000.0;
            self.sim_iter();

            counter += 1;
            // TODO: Could case the system to wait an unnecessary millisecond
            //thread::sleep(Duration::from_millis(cmp::max(
            //    self.delay - iteration_compute_time as u64,
            //    0,
            //)));
            // thread::sleep(Duration::from_millis(self.delay));
        }
        Ok(())
    }
    /// counts all cars in the simulation
    pub fn count_cars(&mut self) -> usize {
        self.nodes.iter().map( | n | {
            match &*n.get() {
                Node::Street(street) => street.lanes.iter().map(| l | l.num_movables()).sum(),
                Node::IONode(node) => 0,
                Node::Crossing(cross) => cross.car_lane.num_movables(),
            }
        }).sum()
    }

    /// a single iteration
    #[tracing::instrument(skip(self))]
    pub fn sim_iter(&mut self) {
        // At the moment all nodes are updated
        // error!("{}", self.delay);
        self.update_all_nodes(self.dt.into());
        thread::sleep(Duration::from_millis(self.delay));
    }

    /// returns status information for all of the cars in the simulation
    ///
    /// the key of the HashMap is the node index
    #[tracing::instrument(skip(self))]
    pub fn get_car_status(&mut self) -> HashMap<usize, Vec<MovableStatus>> {
        let mut mapped_node = HashMap::new();
        for n in self.nodes.iter_mut() {
            let mut n = n.get();
            let car_status = n.get_car_status();
            if car_status.len() != 0 {
                mapped_node.insert(n.id(), car_status);
            }
        }
        info!("Status: {:#?} ", mapped_node);
        mapped_node
    }
    /// sets all IONodes to record the cars that have reached the end to
    ///  created a correct car status message reporting that the cars at
    ///  the end should be deleted
    /// 
    /// this function should be called once before beginning to report car stati
    /// and once again with record set to false
    /// 
    /// **Be very careful!**: If the recording feature is enabled, the IONodes will
    /// fill up a list of cars that have reached the end. These cars will only ever
    /// be deleted if one calls the `get_car_status` method
    #[tracing::instrument(skip(self))]
    pub fn set_car_recording(&mut self, record: bool) {
        self.nodes.iter_mut().for_each(| n | {
            match &mut *n.get() {
                Node::IONode(node) => node.set_car_recording(record),
                _ => {}
            }
        });
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
        use super::super::simulation_builder::SimulatorBuilder;
        use crate::datastructs::IntMut;
        use crate::pathfinding::MovableServer;
        use crate::pathfinding::PathAwareCar;
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        // let mut sim_builder = SimulatorBuilder::<PathAwareCar>::from_json(&json).unwrap();
        // let mut mv_server = MovableServer::<PathAwareCar>::new();
        // mv_server.register_simulator_builder(&sim_builder);
        // let mv_server = IntMut::new(mv_server);
        // sim_builder.with_delay(1).with_max_iter(Some(1000));
        // let mut sim = sim_builder.build(&mv_server);
        // sim.simulation_loop().unwrap();
    }
}
