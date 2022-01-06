use crate::datastructs::{IntMut, MovableStatus};
use crate::path::MovableServer;
use crate::pathfinding::PathAwareCar;
use crate::{SimulatorBuilder, Simulator};
use art_int::genetics::{crossover_sim_nns, mutate_sim_nns};
use art_int::{LayerTopology, ActivationFunc, Network};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use rand::prelude::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::panic;
use std::sync::{mpsc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use rayon::prelude::*;



/// saves a handle to the thread performing the simulation
/// and provides ways of communication
struct Simulating {
    /// Car updates are received from this part of the channel if the simulators are 
    /// set to report updates with `report_updates`
    ///
    /// Unfortunatly, this field has to be wrapped  in a Mutex so it implements the
    /// [Sync] trait. (Which is required by bevy)
    pub car_updates: Mutex<mpsc::Receiver<HashMap<usize, Vec<MovableStatus>>>>,
    /// if this bool is set to true, the Simulators will terminate. This is forceful termination
    pub terminate: IntMut<bool>,
    /// this bool is set by the thread executing the simulations and reports if all simulation has ended
    /// this variable is not public to ensure it is only modified by the simulator
    terminated: IntMut<bool>,
    report_updates: Vec<IntMut<bool>>,
    pub current_generation: IntMut<u32>,
    /// if set to true, the current Generation will be evolved forcefully
    pub terminate_generation: IntMut<bool>,
    pub generation_thread_handle: JoinHandle<Vec<SimData>>
}

/// used to encapsulate data used when creating a Simulator
struct SimData {
    pub simulator: Simulator,
    pub channel: mpsc::Sender<HashMap<usize, Vec<MovableStatus>>>,
    pub report_updates:  IntMut<bool>,
    pub terminate: IntMut<bool>,
    pub terminate_generation: IntMut<bool>,
}
   
impl Simulating {
    /// Creates new simulations and runs them in different threads using the rayon crate
    pub fn new(
        sim_builder: &mut SimulatorBuilder,
        mv_server: &IntMut<MovableServer>,
        population: usize,
        generations: usize,
        mutation_chance: f32,
        mutation_coeff: f32
    ) -> Simulating {
        debug!("creating new Simulating");
        // create all the necessary variables for the simulation thread to later use them in a
        // parallel iterator
        let terminate_generation = IntMut::new(false);
        let report_updates = (0..population).map( | _ | IntMut::new(false)).collect::<Vec<IntMut<bool>>>();
        let (car_tx, car_rx) = mpsc::channel();
        let terminate = IntMut::new(false);
        let simulation_information: Vec<SimData> =  (0..population).map( | i | {
            let mut sim = sim_builder.build(mv_server);
            sim.init_neural_networks_random(
            &[
                    LayerTopology::new(8),
                    LayerTopology::new(6),
                    LayerTopology::new(4),
                    LayerTopology::new(0).with_activation(ActivationFunc::SoftMax),
                ]
            );
            SimData {
                simulator: sim,
                channel: car_tx.clone(),
                report_updates: report_updates[i].clone(),
                terminate: terminate.clone(),
                terminate_generation: terminate_generation.clone()
            }
        }).collect();
        // drop the inital transmitter to prevent having a transmitter that does nothing
        drop(car_tx);
        // Now use this data to simulate in parallel
        let terminated = IntMut::new(false);
        let terminated_ref = terminated.clone();
        let handle = thread::spawn(move || {
            panic::set_hook(Box::new(|e| {
                error!("Simulation panicked! Backtrace: {}", e);
            }));
            let mut rng = thread_rng();
            let mut terminated_sims: Vec<SimData> = simulation_information;
            for _generation in 0..generations {
                terminated_sims = terminated_sims.into_par_iter()
                 .map( move | mut data | {
                    info!("starting Simulation thread");
                    println!("Starting sim thread");
                    panic::set_hook(Box::new(|e| {
                        error!("Simulation panicked! Backtrace: {}", e);
                    }));
                    let mut _i = 0;
                    while !*data.terminate_generation.get() &&  !*data.terminate.get() {
                        _i += 1;
                        data.simulator.sim_iter();
                        // report car position updates
                        if *data.report_updates.get() {
                            let updates = data.simulator.get_car_status();
                            data.channel.send(updates).expect("Unable to send car status updates, even though report_updates is set to true");
                        }
                    }
                    data
                }).collect();
                // TODO: Maybe make this more efficient
                let old_nns_and_costs: Vec<(f32, Vec<Network>)> = terminated_sims.iter_mut().map(
                    | s | (s.simulator.calculate_sim_cost(), s.simulator.remove_all_neural_networks())
                ).collect();
                terminated_sims.iter_mut().for_each( | s | {
                    let parent_a = &old_nns_and_costs.choose_weighted(&mut rng, | (cost, _nns) | 1.0/cost).expect("Empty population").1;
                    let parent_b = &old_nns_and_costs.choose_weighted(&mut rng, | (cost, _nns) | 1.0/cost).expect("Empty population").1;
                    let mut crossed = crossover_sim_nns(parent_a, parent_b, &mut rng);
                    mutate_sim_nns(&mut rng, &mut crossed, mutation_chance, mutation_coeff);
                    s.simulator.set_neural_networks(crossed);
                });
            }
            *terminated_ref.get() = true;
            terminated_sims
        });
        Simulating {
            car_updates: Mutex::new(car_rx),
            terminate,
            terminated,
            current_generation: IntMut::new(0),
            generation_thread_handle: handle,
            report_updates,
            terminate_generation,
        }
    }
    /// True, if the simulation has terminated
    pub fn has_terminated(&self) -> bool {
        *self.terminated.get()
    }
    /// tracks the specified simulation if it exists
    ///  (and untracks all other simulations)
    pub fn track_simulation(&mut self, i: usize) -> Result<(), String> {
        if i >= self.report_updates.len() {
            let err = format!("Index is higher than the number of simulations (got: {}, n sims: {})", i, self.report_updates.len());
            return Err(err);
        }
        self.report_updates.iter_mut().enumerate().for_each( | (j, do_report) | {
            *do_report.get() = i == j
        });
        Ok(())
    }
}

/// This struct saves a list of currently simulating Simulators
/// It also provides the ability to get car updates one of the currently
/// simulating Simulations
pub struct SimManager {
    /// The movable server provides cars to the simulators
    movable_server: IntMut<MovableServer>,
    /// the sim builder generates new simulations and can be used to
    /// configure them (before simulating)
    sim_builder: SimulatorBuilder, // <PathAwareCar>, TODO: Finally implement generics in the simulator struct
    /// A list of currently running Simulators
    simulations: Option<Simulating>,
    /// how likely the nn is to mutate
    mutation_chance: f32,
    /// how strongly it mutates if it mutates
    mutation_coeff: f32,
    /// 
    is_simulating: bool,
    /// the number of generations that should be simulated
    pub generations: usize,
    /// the size of each population in a generation
    pub population: usize
}

/// This error is returned if one tries to modify the SimulatorBuilder while a Simulation is running
///
///  (Otherwise, the SimulationBuilder and Simulator would go out of sync causing all kinds of errors)
#[derive(Debug)]
pub struct SimulationRunningError {
    pub msg: &'static str,
}

impl Display for SimulationRunningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for SimulationRunningError {}

impl Error for SimulationDoesNotExistError {}

/// This error is returned if one tries to modify the SimulatorBuilder while a Simulation is running
///
///  (Otherwise, the SimulationBuilder and Simulator would go out of sync causing all kinds of errors)
#[derive(Debug)]
pub struct SimulationDoesNotExistError {}

impl Display for SimulationDoesNotExistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The simulation has been tried to access does not exist")
    }
}

impl SimManager {
    /// creates a new SimManager with an empty SimulationBuilder
    pub fn new() -> SimManager {
        let mut sim_builder = SimulatorBuilder::<PathAwareCar>::new();
        let movable_server = MovableServer::<PathAwareCar>::new();
        SimManager {
            movable_server: IntMut::new(movable_server),
            sim_builder: sim_builder,
            simulations: None,
            mutation_chance: 0.01,
            mutation_coeff: 0.3,
            is_simulating: false,
            population: 10,
            generations: 10
        }
    }
    /// Returns a mutable reference to the SimulatorBuilder, if no Simulation
    /// is currently running
    pub fn modify_sim_builder(&mut self) -> Result<&mut SimulatorBuilder, SimulationRunningError> {
        // are any simulations still running?
        let any_sims = self.simulations.iter().any(|s| !s.has_terminated());
        if any_sims {
            return Err(SimulationRunningError {
                msg: "Cannot modify SimulatorBuilder, as Simulations are running.",
            });
        }
        return Ok(&mut self.sim_builder);
    }
    /// Starts simulating 
    pub fn simulate(&mut self) -> Result<(), Box<dyn Error>> {
        // are any simulations still running?
        let any_sims = self.simulations.iter().any(|s| !s.has_terminated());
        if any_sims {
            return Err(Box::new(SimulationRunningError {
                msg: "Can not start new simulations while old ones are still running.",
            }));
        }
        // index nodes
        self.movable_server
            .get()
            .register_simulator_builder(&self.sim_builder);
        self.simulations = Some(
            Simulating::new(
                &mut self.sim_builder,
                &self.movable_server,
                self.population, 
                self.generations,
                self.mutation_chance,
                self.mutation_coeff
            )
        );
        self.is_simulating = true;
        Ok(())
    }

    /// Are Simulations currently running?
    pub fn is_simulating(&self) -> bool {
        self.is_simulating
    }


    /// Terminates the current generation and performs Crossover
    pub fn terminate_generation(&mut self) {
        if let Some(sim) = &mut self.simulations {
            *sim.terminate_generation.get() = true;
        }
    }

    /// returns a status update, if it is found in the channel, else
    /// None is returned. None is also returned, if no Simulation is tracked
    ///
    /// TODO: Add some way of handling the case where the Simulation computes
    ///  status updates faster than the UI can display it (This could cause the
    ///  Receiver to fill up.)
    pub fn get_status_updates(&self) -> Option<HashMap<usize, Vec<MovableStatus>>> {
        if let Some(sim) = &self.simulations {
            if let Ok(value) = sim.car_updates.lock().expect("Unable to aquire lock on Car Update Receiver")
            .recv_timeout(Duration::from_millis(20))
                {
                    return Some(value)
                }
        }
        None
    }

    /// tracks the car_updates of the simulation with the given index#
    /// raises an error, if no simulation with the given index exists
    pub fn track_simulation(&mut self, i: usize) -> Result<(), String> {
        match &mut self.simulations {
            Some(sim) => sim.track_simulation(i),
            None => Err("Can not track simulation if no simulations are running".to_string()),
        }
    }
}
