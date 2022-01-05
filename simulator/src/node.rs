use super::int_mut::{IntMut, WeakIntMut};
use super::movable::RandCar;
use super::node_builder::{CrossingConnections, Direction, InOut};
use super::traversible::Traversible;
use crate::movable::MovableStatus;
use crate::pathfinding::MovableServer;
use crate::simulation::calculate_cost;
use crate::traits::{Movable, NodeTrait, CarReport};
use std::convert::TryFrom;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::error::Error;
use std::ptr;
use art_int;

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
    fn update_cars(&mut self, t: f64) -> Vec<usize> {
        match self {
            Node::Street(street) => street.update_movables(t),
            Node::IONode(io_node) => {
                // create new car
                io_node.time_since_last_spawn += t;
                let mut new_cars = Vec::<usize>::new();
                // TODO: rework spawn rate
                if io_node.time_since_last_spawn >= io_node.spawn_rate {
                    // TODO: Remove and replace with proper request to
                    //  the movable server
                    // new_cars.push(Car::new())
                    match &mut io_node.movable_server {
                        Some(server) => {
                            let car_result = server.get().generate_movable(io_node.id);
                            info!("Spawned new movable");
                            if let Ok(mut car) = car_result {
                                io_node.cached.push(car);
                                new_cars.push(io_node.cached.len()-1);
                            }
                            io_node.time_since_last_spawn = 0.0;
                        }
                        None => warn!("Trying to simulate Node with uninitialised MovableServer"),
                    }
                }
                new_cars
            }
            Node::Crossing(crossing) => crossing.car_lane.update_movables(t),
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
    fn get_car_status(&self) -> Vec<MovableStatus> {
        match self {
            Node::Street(inner) => inner.get_car_status(),
            Node::IONode(inner) => inner.get_car_status(),
            Node::Crossing(inner) => inner.get_car_status(),
        }
    }

    fn rm_car_by_ref(&mut self, car: &Car) -> Car {
        match self {
            Node::Street(inner) => {
                for l in inner.lanes.iter_mut() {
                    if let Ok(car) = l.rm_movable_by_ref(car) {
                        return car;
                    }
                }
                panic!("trying to remove car by reference that does not exist")
            },
            Node::IONode(inner) => {
                let index = match inner.cached.iter().enumerate().find(
                    | (i, cached_car) | ptr::eq(*cached_car, car)
                ) {
                    Some((i, _)) => i,
                    None => panic!("Invalid reference passed to rm_movable_by_ref")
                };
                inner.cached.remove(index)
            },
            Node::Crossing(inner) => inner.car_lane.rm_movable_by_ref(car).unwrap(),
        }
        
    }

    fn remove_car(&mut self, i: usize) -> Car {
        match self {
            Node::Street(street) => street.remove_car(i),
            Node::IONode(io_node) => io_node.cached.remove(i),
            Node::Crossing(crossing) => crossing.car_lane.remove_movable(i),
        }
    }

    fn get_car_by_index(&mut self, i: usize) -> &Car {
        match self {
            Node::Street(street) => street.get_car_by_index(i),
            Node::IONode(ionode) => &ionode.cached[i],
            Node::Crossing(crossing) => crossing.car_lane.get_movable_by_index(i),
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
    pub nn: Option<art_int::Network>
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
            nn: None 
        }
    }
    /// calculates the inputs for the neural network controlling the traffic light state
    /// # What are the inputs?
    /// 
    /// 1. For each direction, how many cars are waiting to go over the crossing?
    /// 2. For each direction, when was the last time, cars were able to go over the crossing?
    /// 
    /// If there is no street, the time and number of cars is set to 0.0
    pub fn calculate_nn_inputs(&self) -> [f32; 8] {
        let mut cars_at_end = [0.0f32; 4];
        self.connections.input.iter().for_each(| (dir, conn) | {
            let index = match dir {
                Direction::N => 0,
                Direction::E => 1,
                Direction::S => 2,
                Direction::W => 3,
            };
            let cars = match &*conn.upgrade().get() {
                Node::Street(street) => street.get_num_cars_at_end(),
                _ => {error!("Crossing connected with crossing or IONode"); return}
            };
            cars_at_end[index] = cars as f32;
        });
        [
            cars_at_end[0],
            cars_at_end[1],
            cars_at_end[2],
            cars_at_end[3],
            self.time_since_input_passable[0],
            self.time_since_input_passable[1],
            self.time_since_input_passable[2],
            self.time_since_input_passable[3],
        ]
    }
    /// Is used to set the NN given by the genetic algorithm
    pub fn set_neural_network(&mut self, nn: art_int::Network) {
        // make sure the input has the right size
        assert_eq!(nn.layers[0].neurons[0].weights.len(), 8);
        self.nn = Some(nn);
    }
    /// computes the traffic light state using the neural network
    pub fn determine_traffic_light_state(&self) -> Result<TrafficLightState, &'static str> {
        let nn_input = self.calculate_nn_inputs();
        // the output should be a value between 0 and 1 where 0.25 is state 0, 0.5 is state 1 and so on
        let nn_output = match &self.nn {
            Some(nn) => {
                let out_vec = nn.propagate(nn_input.into());
                let out = out_vec.get(0);
                out.map(| op | *op).ok_or("NN has no output!")?
            },
            None => return Err("cannot determine traffic state without NeuralNetwork"),
        };
        Ok (
            if nn_output < 0.25 {
                TrafficLightState::S0
            } else if nn_output < 0.5 {
                TrafficLightState::S1
            } else if nn_output < 0.75 {
                TrafficLightState::S2
            } else {
                TrafficLightState::S3
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
            .expect("Crossing seems not to be connected with street (input)");
        let output_node_dir = self
            .connections
            .get_direction_for_item(InOut::OUT, out_node)
            .expect("Crossing seems not to be connected with street (output)");
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
pub struct IONode<Car = RandCar,>
where
    Car: Movable,
{
    /// All the nodes where cars should be redirected
    pub connections: Vec<WeakIntMut<Node<Car>>>,
    /// new Cars/Second
    pub spawn_rate: f64,
    /// parameters for calculating the cost
    pub cost_calc_params: CostCalcParameters,
    /// time since last spawn in seconds
    pub time_since_last_spawn: f64,
    /// Tracks how many cars have reached their destination in this node
    pub absorbed_cars: usize,
    /// To differentiate different nodes. Should be set to the positions in the
    /// list of all nodes in the simulation
    pub id: usize,
    /// car cache to be able to return references in the update_cars function
    pub cached: Vec<Car>,
    /// total cost all cars produced
    pub total_cost:  f32,
    /// The movable server used to spawn new cars
    pub movable_server: Option<IntMut<MovableServer<Car>>>,
}
impl<Car> IONode<Car>
where
    Car: Movable,
{
    /// Returns a new IONode
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            spawn_rate: 1.0,
            time_since_last_spawn: 0.0,
            absorbed_cars: 0,
            total_cost: 0.0,
            id: 0,
            cached: Vec::new(),
            movable_server: None,
            cost_calc_params: CostCalcParameters {},
        }
    }
    /// Used when constructing a node from a [NodeBuilder](crate::nodes::NodeBuilder)
    ///  The Movable server is added so that the node can spawn new cars
    ///
    ///  This functionality is not in the `new` function to keep the function signature
    ///  the same across all node types
    pub fn register_movable_server(&mut self, mv_server: &IntMut<MovableServer<Car>>) {
        self.movable_server = Some(mv_server.clone())
    }

    /// adds a connections
    pub fn connect(&mut self, n: &IntMut<Node<Car>>) {
        self.connections.push(n.downgrade())
    }
    /// get car status (position and lane index)
    pub fn get_car_status(&self) -> Vec<MovableStatus> {
        Vec::new()
    }

    /// adds car
    pub fn add_car(&mut self, car: Car) {
        self.absorbed_cars += 1;
        self.total_cost += calculate_cost(car.get_report(), &self.cost_calc_params);
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
            for m in traversible.update_movables(t) {
                movables.push(m + offset)
            }
            offset += traversible.num_movables();
        }
        movables
    }
    /// returns the number of cars waiting at the end
    pub fn get_num_cars_at_end(&self) -> u32 {
        self.lanes.iter().map(| lane | lane.num_movables_waiting()).sum()
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
                return lane.get_movable_by_index(i) 
            }
            element_index -= num_m;
        }
        panic!("Invalid Index!")
    }

    /// Adds a movable to the street
    ///
    /// The movable is put on the lane with the leas amount of cars
    /// This should pretty closely resemble how people drive in real life
    /// as you won't drive to the lane that has the most cars in it
    pub fn add_movable(&mut self, movable: Car) {
        info!("Adding movable to dstreet");
        // get the index of the lane with the least movables on it
        let trav_most_movables = self
            .lanes
            .iter()
            .enumerate()
            .min_by_key(|(_i, traversible)| traversible.num_movables());
        let i = match trav_most_movables {
            Some((i, _)) => i,
            None => {warn!("Can not determine lane with minimum number of cars."); return},
        };
        self.lanes[i].add(movable)
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
