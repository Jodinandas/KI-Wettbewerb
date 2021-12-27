use simulator::datastructs::{IntMut, MovableStatus};
use simulator::path::{MovableServer, PathAwareCar};
use simulator::SimulatorBuilder;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::sync::{mpsc, Mutex};
use std::thread::{self, JoinHandle};

/// saves a handle to the thread performing the simulation
/// and provides ways of communication
struct Simulating {
    /// the handle of the simulation thread
    sim: JoinHandle<()>,
    /// Car updates are received from this part of the channel if the simulator is
    /// set to report updates with `report_updates`
    ///
    /// Unfortunatly, this field has to be wrapped  in a Mutex so it implements the
    /// [Sync] trait. (Which is required by bevy)
    pub car_updates: Mutex<mpsc::Receiver<HashMap<usize, Vec<MovableStatus>>>>,
    /// if this bool is set to true, the Simulator will terminate
    pub terminate: IntMut<bool>,
    /// this bool is set by the simulator and reports if the simulation has ended
    /// this variable is not public to ensure it is only modified by the simulator
    terminated: IntMut<bool>,
    /// if set to true, updates will be sent to `car_updates`
    pub report_updates: IntMut<bool>,
}

impl Simulating {
    /// starts a new simulation
    pub fn new(sim_builder: &mut SimulatorBuilder, time_steps: f32) -> Simulating {
        let mut new_sim = sim_builder.build();
        // the channel that information about the car updates will be passed through
        let (tx, rx) = mpsc::channel();
        let terminate = IntMut::new(false);
        let terminated = IntMut::new(false);
        let report_updates = IntMut::new(false);
        let report_updates_moved = report_updates.clone();
        let terminate_moved = terminate.clone();
        let terminated_moved = terminated.clone();
        // -------------------------- this is where the magic happens --------------
        let handle = thread::spawn(move || {
            while !*terminate_moved.get() {
                new_sim.sim_iter(time_steps.into());
                // report car position updates
                if *report_updates_moved.get() {
                    tx.send(new_sim.get_car_status()).expect("Unable to send car status updates, even though report_updates is set to true");
                }
            }
            *terminated_moved.get() = true;
        });
        Simulating {
            sim: handle,
            car_updates: Mutex::new(rx),
            terminate,
            terminated,
            report_updates,
        }
    }
    /// True, if the simulation has terminated
    pub fn has_terminated(&self) -> bool {
        *self.terminated.get()
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
    simulations: Vec<Simulating>,
    /// the index of the simulation which currently tracks car updates
    tracking_index: Option<usize>,
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
        let sim_builder = SimulatorBuilder::new();
        SimManager {
            movable_server: IntMut::new(MovableServer::new()),
            sim_builder: sim_builder,
            simulations: Vec::new(),
            tracking_index: None,
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
    ///
    pub fn simulate(&mut self, num_sims: usize) -> Result<(), Box<dyn Error>> {
        // are any simulations still running?
        let any_sims = self.simulations.iter().any(|s| !s.has_terminated());
        if any_sims {
            return Err(Box::new(SimulationRunningError {
                msg: "Can not start new simulations while old ones are still running.",
            }));
        }
        for _i in 0..num_sims {}
        Ok(())
    }

    /// returns a status update, if it is found in the channel, else
    /// None is returned. None is also returned, if no Simulation is tracked
    ///
    /// TODO: Add some way of handling the case where the Simulation computes
    ///  status updates faster than the UI can display it (This could cause the
    ///  Receiver to fill up.)
    pub fn get_status_updates(&self) -> Option<HashMap<usize, Vec<MovableStatus>>> {
        match self.tracking_index {
            Some(i) => Some(
                self.simulations[i]
                    .car_updates
                    .lock()
                    .unwrap()
                    .try_recv()
                    .expect("Unable to get car updates!"),
            ),
            None => return None,
        }
    }

    /// tracks the car_updates of the simulation with the given index#
    /// raises an error, if no simulation with the given index exists
    pub fn track_simulation(&mut self, i: usize) -> Result<(), String> {
        match i < self.simulations.len() {
            true => {
                // if another simulation was tracked, stop the status updates
                match self.tracking_index {
                    Some(old_i) => {
                        *self.simulations[old_i].report_updates.get() = false;
                    }
                    None => {}
                }
                self.tracking_index = Some(i);
                *self.simulations[i].report_updates.get() = true;
            }
            false => return Err("Simulation with the given index does not exist".into()),
        }
        Ok(())
    }
}
