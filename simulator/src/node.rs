use super::int_mut::{IntMut, WeakIntMut};
use super::movable::RandCar;
use super::node_builder::{CrossingConnections, Direction, InOut};
use super::traversible::Traversible;
use crate::movable::MovableStatus;
use crate::pathfinding::MovableServer;
use crate::simulation::calculate_cost;
use crate::traits::{CarReport, Movable, NodeTrait};
use art_int;
use rand::Rng;
use rand::prelude::ThreadRng;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::error::Error;
use std::ptr;

/// A node is any kind of logical object in the Simulation
///  ([Streets](Street), [IONodes](IONode), [Crossings](Crossing))
///
/// # Examples
/// ## How to create a node
/// Nodes are typically created by a [NodeBuilder](super::node_builder::NodeBuilder) objects using
/// the build method.
#[derive(Debug, Clone)]
pub enum Node<Car = RandCar>
where
    Car: Movable,
{
    /// Wrapper
    Street(Street<Car>),
    /// Wrapper
    IONode(IONode<Car>),
    /// Wrapper
    Crossing(Crossing<Car>),
}

impl<Car: Movable> NodeTrait<Car> for Node<Car> {
    fn is_connected(&self, other: &IntMut<Node<Car>>) -> bool {
        self.get_out_connections()
            .iter()
            .find(|n| *n == other)
            .is_some()
    }
    fn update_cars(&mut self, t: f64, mv_server: &mut MovableServer<Car>, rng: &mut ThreadRng) -> Vec<usize> {
        match self {
            Node::Street(street) => street.update_movables(t),
            Node::IONode(io_node) => io_node.update_cars(t, mv_server, rng),
            Node::Crossing(crossing) => {
                crossing.traffic_light_state = crossing.determine_traffic_light_state().expect("Error when determining traffic light state");
                crossing.car_lane.update_movables(t as f32)
            },
        }
    }



    fn get_out_connections(&self) -> Vec<WeakIntMut<Node<Car>>> {
        match self {
            Node::Street(street) => street.get_out_connections(),
            Node::IONode(io_node) => io_node.connections.clone(),
            Node::Crossing(crossing) => crossing.get_out_connections(),
        }
    }

    fn add_car(&mut self, car: Car) {
        match self {
            Node::Street(street) => street.add_movable(car),
            Node::IONode(io_node) => io_node.add_car(car),
            Node::Crossing(crossing) => crossing.car_lane.add(car),
        }
    }

    fn id(&self) -> usize {
        match self {
            Node::Street(inner) => inner.id,
            Node::IONode(inner) => inner.id,
            Node::Crossing(inner) => inner.id,
        }
    }
    fn get_car_status(&mut self) -> Vec<MovableStatus> {
        match self {
            Node::Street(inner) => inner.get_car_status(),
            Node::IONode(inner) => inner.get_car_status(),
            Node::Crossing(inner) => inner.get_car_status(),
        }
    }

    // fn rm_car_by_ref(&mut self, car: &Car) -> Car {
    //     match self {
    //         Node::Street(inner) => {
    //             for l in inner.lanes.iter_mut() {
    //                 if let Ok(car) = l.rm_movable_by_ref(car) {
    //                     return car;
    //                 }
    //             }
    //             panic!("trying to remove car by reference that does not exist")
    //         }
    //         Node::IONode(inner) => {
    //             let index = match inner
    //                 .cached
    //                 .iter()
    //                 .enumerate()
    //                 .find(|(_i, cached_car)| ptr::eq(*cached_car, car))
    //             {
    //                 Some((i, _)) => i,
    //                 None => panic!("Invalid reference passed to rm_movable_by_ref"),
    //             };
    //             inner.cached.remove(index)
    //         }
    //         Node::Crossing(inner) => inner.car_lane.rm_movable_by_ref(car).unwrap(),
    //     }
    // }

    fn remove_car(&mut self, i: usize) -> Car {
        match self {
            Node::Street(street) => street.remove_car(i),
            Node::IONode(io_node) => io_node.cached.remove(&i).unwrap(),
            Node::Crossing(crossing) => crossing.car_lane.remove_movable(i),
        }
    }

    fn get_car_by_index(&mut self, i: usize) -> &Car {
        match self {
            Node::Street(street) => street.get_car_by_index(i),
            Node::IONode(ionode) => ionode.cached.get(&i).unwrap(),
            Node::Crossing(crossing) => crossing.car_lane.get_movable_by_index(i),
        }
    }

    fn reset_cars(&mut self) {
        match self {
            Node::Street(s) => s.lanes.iter_mut().for_each(| l | l.reset()),
            Node::IONode(node) => {node.cached = HashMap::new(); node.recorded_cars = Vec::new(); node.num_cars_spawned = 0; node.total_cost = 0.0;},
            Node::Crossing(node) => {node.car_lane.reset()},
        }
    }

    fn get_overnext_node_ids(&self) -> HashMap<usize, u32> {
        match self {
            Node::Street(street) => street.lanes.iter().flat_map(| l | l.get_overnext_node_ids()).collect(),
            Node::IONode(node) => HashMap::new(),
            Node::Crossing(cross) => cross.car_lane.get_overnext_node_ids(),
        }
    }

    fn get_target_id_of_car_at_end(&self) -> Option<usize> {
        match self {
            Node::Street(street) => street.lanes[0].get_target_id_of_movable_at_end(),
            Node::IONode(node) => None,
            Node::Crossing(crossing) => crossing.car_lane.get_target_id_of_movable_at_end(),
        }
    }
}

/// The state of a traffic light (ampelstatus)
#[derive(Debug, Clone)]
pub enum TrafficLightState {
    /// State 0
    S0,
    /// State 1
    S1,
    /// State 2
    S2,
    /// State 3
    S3,
}

/// A simple crossing
#[derive(Debug, Clone)]
pub struct Crossing<Car = RandCar>
where
    Car: Movable,
{
    /// The other nodes the Crossing is connected to
    ///
    /// A crossing is a rectangle and each of the 4 sides
    /// can have one input and one output connection
    pub connections: CrossingConnections<Node<Car>>,
    /// cars are stored in this field
    pub car_lane: Traversible<Car>,
    /// a number to differentiate different nodes
    pub id: usize,
    /// the state of the traffic light (ampelphase)
    pub traffic_light_state: TrafficLightState,
    /// time since last cars could drive over the crossing in each direction
    ///
    /// `[N, E, S, W]`
    ///  
    /// for further explanation, look at the method `calculate_nn_inputs`
    pub time_since_input_passable: [f32; 4],
    /// the NN used to determine the traffic light state at each iteration
    pub nn: Option<art_int::Network>,
}
impl<Car: Movable> Crossing<Car> {
    /// Returns a new Crossing with no connections and id=0
    pub fn new() -> Crossing {
        Crossing {
            connections: CrossingConnections::new(),
            car_lane: Traversible::<Car>::new(1.0),
            id: 0,
            traffic_light_state: TrafficLightState::S0,
            time_since_input_passable: [0.0; 4],
            nn: None,
        }
    }
    /// calculates the inputs for the neural network controlling the traffic light state
    /// # What are the inputs?
    ///
    /// // 1. For each direction, how many cars are waiting to go over the crossing?
    /// 3. What direction do the cars want to go to?
    ///
    /// If there is no street, the time and number of cars is set to 0.0
    pub fn calculate_nn_inputs(&self) -> [f32; 16] {
        let mut cars_at_end = [0.0f32; 16];

        let mut i = 0;
        let map_output_id_to_dir_index: HashMap<usize, Direction> = self.connections.output.iter().map(| (dir, conn) | {
            (conn.upgrade().get().id(), *dir)
        }).collect();
        for dir in [Direction::N, Direction::E, Direction::S, Direction::W] {
            if let Some(conn) = self.connections.input.get(&dir) {
                let node_id = conn.upgrade().get().get_target_id_of_car_at_end();
                if let Some(id) = node_id {
                    let dir_out = map_output_id_to_dir_index[&id];
                    let offset = match dir_out {
                        Direction::N => 0,
                        Direction::E => 1,
                        Direction::S => 2,
                        Direction::W => 3,
                    };
                    cars_at_end[i + offset] = 1.0;
                }
                // for (id, count) in node_ids {
                //     cars_at_end[i + offset] = count as f32;
                // }
            } 
            i += 4;
        }
        cars_at_end
    }
    /// Is used to set the NN given by the genetic algorithm
    pub fn set_neural_network(&mut self, nn: art_int::Network) {
        // make sure the input has the right size
        assert_eq!(nn.layers[0].neurons[0].weights.len(), 16);
        self.nn = Some(nn);
    }
    /// computes the traffic light state using the neural network
    pub fn determine_traffic_light_state(&self) -> Result<TrafficLightState, &'static str> {
        let nn_input = self.calculate_nn_inputs();
        // the output should be a value between 0 and 1 where 0.25 is state 0, 0.5 is state 1 and so on
        let nn_output = match &self.nn {
            Some(nn) => {
                let out_vec = nn.propagate(nn_input.into());
                //let out = out_vec.get(0);
                //out.map(|op| *op).ok_or("NN has no output!")?
                out_vec
            }
            None => return Err("cannot determine traffic state without NeuralNetwork"),
        };
        let i = nn_output.iter().enumerate().max_by(| (_, a), (_, b) | a.partial_cmp(b).unwrap_or(Ordering::Equal)).unwrap().0;
        Ok(
            match i {
                0 => TrafficLightState::S0,
                1 => TrafficLightState::S1,
                2 => TrafficLightState::S2,
                3 => TrafficLightState::S3,
                _ => {warn!("NN returned strange index ({})", i); return Err("Weird index")},
            }
        )
    }

    /// removes the neural network and returns it
    pub fn remove_neural_network(&mut self) -> Result<art_int::Network, &'static str> {
        let nn = self.nn.take();
        nn.ok_or("No neural network to remove!")
    }

    /// Returns a list of only OUTPUT connecitons
    ///
    /// This function is deprecated and will be removed soon
    pub fn get_out_connections(&self) -> Vec<WeakIntMut<Node<Car>>> {
        self.connections
            .output
            .values()
            .map(|c| c.clone())
            .collect()
    }
    /// Tries to add a connections at the specified position and raises
    /// an error if this is not possible
    pub fn connect(
        &mut self,
        dir: Direction,
        conn_type: InOut,
        other: &IntMut<Node<Car>>,
    ) -> Result<&mut Self, Box<dyn Error>> {
        self.connections.add(dir, conn_type, other)?;
        Ok(self)
    }
    /// gets movable status
    pub fn get_car_status(&self) -> Vec<MovableStatus> {
        self.car_lane.get_movable_status()
    }
    /// determines whether out node on crossing can be reached by current state of the traffic light
    ///# State 0
    ///```text
    ///       N
    ///     /| ^
    ///    / | |
    ///W <-  | |  -> E
    ///      | | /
    ///      v |/
    ///       S
    ///```
    ///
    ///# State 1
    ///```text
    ///       N
    ///       ^       
    ///        \
    ///  <–––––––––––
    ///W –––––––––––> E
    ///       \
    ///       v     
    ///       S
    ///```
    ///
    ///# State 2
    ///```text
    ///       N       
    ///        \
    ///W <–––   –––> E
    ///      \
    ///       S
    ///```
    ///       
    ///# State 3
    ///```text
    ///       N
    ///       ^
    ///      /
    ///W ––––   –––– E
    ///        /
    ///       v
    ///       S
    /// ```
    pub fn can_out_node_be_reached(
        &self,
        in_node: &IntMut<Node<Car>>,
        out_node: &IntMut<Node<Car>>,
    ) -> bool {
        let input_node_dir = self
            .connections
            .get_direction_for_item(InOut::IN, in_node)
            .expect("Crossing doesn't seem to be connected with street (input)");
        let output_node_dir = self
            .connections
            .get_direction_for_item(InOut::OUT, out_node)
            .expect("Crossing doesn't seem to be connected with street (output)");
        // funky stuff here
        match self.traffic_light_state {
            TrafficLightState::S0 => {
                if input_node_dir == Direction::N {
                    if output_node_dir == Direction::S || output_node_dir == Direction::W {
                        return true;
                    } else {
                        return false;
                    }
                } else if input_node_dir == Direction::S {
                    if output_node_dir == Direction::N || output_node_dir == Direction::E {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            TrafficLightState::S1 => {
                if input_node_dir == Direction::W {
                    if output_node_dir == Direction::S || output_node_dir == Direction::E {
                        return true;
                    } else {
                        return false;
                    }
                } else if input_node_dir == Direction::E {
                    if output_node_dir == Direction::W || output_node_dir == Direction::N {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            TrafficLightState::S2 => {
                if input_node_dir == Direction::N {
                    if output_node_dir == Direction::E {
                        return true;
                    } else {
                        return false;
                    }
                } else if input_node_dir == Direction::S {
                    if output_node_dir == Direction::W {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            TrafficLightState::S3 => {
                if input_node_dir == Direction::W {
                    if output_node_dir == Direction::N {
                        return true;
                    } else {
                        return false;
                    }
                } else if input_node_dir == Direction::E {
                    if output_node_dir == Direction::S {
                        return true;
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
    }
}

/// information important for calculating the Cost
#[derive(Clone, Debug)]
pub struct CostCalcParameters;

/// A Node that represents either the start of the simulation or the end of it
///
/// One of its responsibilities is to add cars and passengers to the simulation
#[derive(Debug, Clone)]
pub struct IONode<Car = RandCar>
where
    Car: Movable,
{
    /// All the nodes where cars should be redirected
    pub connections: Vec<WeakIntMut<Node<Car>>>,
    /// new Cars/Second
    pub spawn_rate: f64,
    /// parameters for calculating the cost
    pub cost_calc_params: CostCalcParameters,
    /// Tracks how many cars have reached their destination in this node
    pub absorbed_cars: usize,
    /// To differentiate different nodes. Should be set to the positions in the
    /// list of all nodes in the simulation
    pub id: usize,
    /// car cache to be able to return references in the update_cars function
    pub cached: HashMap<usize, Car>,
    /// total cost all cars produced
    pub total_cost: f32,
    /// if set to true, the node will record cars that have reached it and only
    /// delete them if get_car_status is called
    pub record: bool,
    pub num_cars_spawned: usize,
    /// the cars that have been recorded
    pub recorded_cars: Vec<Car>
}
impl<Car> IONode<Car>
where
    Car: Movable,
{
    /// Returns a new IONode
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            spawn_rate: 0.01,
            absorbed_cars: 0,
            total_cost: 0.0,
            id: 0,
            cached: HashMap::new(),
            cost_calc_params: CostCalcParameters {},
            record: false,
            recorded_cars: Vec::new(),
            num_cars_spawned: 0
        }
    }

    /// adds a connections
    pub fn connect(&mut self, n: &IntMut<Node<Car>>) {
        self.connections.push(n.downgrade())
    }
    /// get car status (position and lane index)
    pub fn get_car_status(&mut self) -> Vec<MovableStatus> {
        self.recorded_cars.drain(..).map(| car | {
            MovableStatus {
                position: 0.0,
                lane_index: 0,
                movable_id: car.get_id(),
                delete: true,
            }
        }).collect()
    }

    /// adds car
    pub fn add_car(&mut self, car: Car) {
        self.absorbed_cars += 1;
        self.total_cost += calculate_cost(car.get_report(), &self.cost_calc_params);
        if self.record {
            self.recorded_cars.push(car);
        }
    }

    /// sets self.record
    pub fn set_car_recording(&mut self, record: bool) {
        self.record = record;
    }

    /// is responsible for spawning new cars if a time is reached
    pub fn update_cars(&mut self, dt: f64, mv_server: &mut MovableServer<Car>, rng: &mut ThreadRng) -> Vec<usize> {
        // create new car
        let mut new_cars = Vec::<usize>::new();
        // TODO: rework spawn rate
        if rng.gen_bool(self.spawn_rate*dt) {
            // TODO: Remove and replace with proper request to
            //  the movable server
            // new_cars.push(Car::new())
            let car_result = mv_server.generate_movable(self.id);
            match car_result {
                Ok(car) => {
                    self.cached.insert(self.num_cars_spawned, car);
                    new_cars.push(self.num_cars_spawned);
                    self.num_cars_spawned += 1;
                },
                Err(err) => {
                    warn!("Unable to generate new car: {}", err);
                },
            }
        }
        new_cars
    }
}

/// A `Street` is mostly used to connect `IONode`s or `Crossing`s
///
/// # Fields
/// - `lanes` stores how many lanes the `Street` has
#[derive(Debug, Clone)]
pub struct Street<Car = RandCar>
where
    Car: Movable,
{
    /// The connection leading to the node at the end of the road
    pub conn_out: Option<WeakIntMut<Node<Car>>>,
    /// The node where the road starts at
    pub conn_in: Option<WeakIntMut<Node<Car>>>,
    /// This field handles the actual logic of moving cars etc.
    /// it contains a list of all the lanes
    pub lanes: Vec<Traversible<Car>>,
    /// The index in the simulation
    pub id: usize,
}

impl<Car: Movable> Street<Car> {
    /// Create new street
    pub fn new(_end: &IntMut<Node>) -> Street {
        Street {
            conn_out: None,
            conn_in: None,
            lanes: vec![Traversible::<Car>::new(100.0)],
            id: 0,
        }
    }
    /// Connects a node at the specifed position. If a node is already
    /// connected at the position, it is simply overwritten
    /// FIXME: Raise an error if there is already a connection, or unconnect
    ///     the node the street is connected to as well
    pub fn connect(&mut self, conn_type: InOut, other: &IntMut<Node<Car>>) -> &mut Self {
        let new_item = Some(other.downgrade());
        match conn_type {
            InOut::IN => self.conn_in = new_item,
            InOut::OUT => self.conn_out = new_item,
        }
        self
    }
    /// Returns the out connection in a Vec of length 1 (or 0 if there is none)
    pub fn get_out_connections<'a>(&'a self) -> Vec<WeakIntMut<Node<Car>>> {
        let mut out = Vec::new();
        if let Some(conn) = &self.conn_out {
            out.push(conn.clone());
        }
        out
    }
    /// Advances the movables on all lanes
    ///
    /// # How is the index calculated?
    /// Imagine this set of lanes
    /// ```text
    /// Lane 0: 0  1  2  3  4  5    | num of mov. : 5
    /// Lane 1: 6  7  8             | num of mov. : 3
    /// Lane 2: 9  10 11 12         | num of mov. : 4
    /// ```
    /// Let's say movable `10` has reached the end
    ///
    /// How can we calculate the lane and the index on that lane
    /// from just this number?
    ///
    /// * Step 1: 10 - 5 = 5
    /// * Step 2: 5 - 3 = 2
    /// * Step 3: 2 - 4 < 0, so the offset is the number of movables on the previous two lanes
    ///  and the movable is on this lane (lane 2). The index in the lane is 2
    pub fn update_movables(&mut self, t: f64) -> Vec<usize> {
        let mut offset = 0;
        let mut movables = Vec::new();
        for traversible in self.lanes.iter_mut() {
            for m in traversible.update_movables(t as f32) {
                movables.push(m + offset)
            }
            offset += traversible.num_movables();
        }
        movables
    }
    /// returns the number of cars waiting at the end
    pub fn get_num_cars_at_end(&self) -> u32 {
        self.lanes
            .iter()
            .map(|lane| lane.num_movables_waiting())
            .sum()
    }
    /// removes a movable using the index.
    ///
    /// to see how the index is calculated, go to the documentation of `update_movable`
    pub fn remove_car(&mut self, index: usize) -> Car {
        let mut element_index = index;
        for lane in self.lanes.iter_mut() {
            let num_m = lane.num_movables();
            if (element_index as isize - num_m as isize) < 0 {
                return lane.remove_movable(element_index);
            }
            element_index -= num_m;
        }
        panic!("Invalid Index!")
    }
    /// returns a reference to the Car with index i
    fn get_car_by_index(&mut self, i: usize) -> &Car {
        let mut element_index = i as isize;
        for lane in self.lanes.iter() {
            let num_m = lane.num_movables() as isize;
            if element_index - num_m < 0 {
                return lane.get_movable_by_index(i);
            }
            element_index -= num_m;
        }
        panic!("Invalid Index!")
    }

    /// Adds a movable to the street
    pub fn add_movable(&mut self, movable: Car) {
        info!("Adding movable to dstreet");
        // get the index of the lane with the least movables on it
        // let trav_most_movables = self
        //     .lanes
        //     .iter()
        //     .enumerate()
        //     .min_by_key(|(_i, traversible)| traversible.num_movables());
        // let i = match trav_most_movables {
        //     Some((i, _)) => i,
        //     None => {
        //         warn!("Can not determine lane with minimum number of cars.");
        //         return;
        //     }
        // };
        self.lanes[0].add(movable)
    }
    /// gets car status
    pub fn get_car_status(&self) -> Vec<MovableStatus> {
        let mut car_status = Vec::new();
        for (i, s) in self.lanes.iter().enumerate() {
            let stati = s.get_movable_status();
            for mut status in stati {
                status.lane_index = i as u8;
                car_status.push(status);
            }
        }
        car_status
    }
}
