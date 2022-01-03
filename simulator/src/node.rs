use super::int_mut::{IntMut, WeakIntMut};
use super::movable::RandCar;
use super::node_builder::{CrossingConnections, Direction, InOut};
use super::traversible::Traversible;
use crate::movable::MovableStatus;
use crate::pathfinding::MovableServer;
use crate::traits::{Movable, NodeTrait};
use std::error::Error;
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

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
    fn update_cars(&mut self, t: f64) -> Vec<Car> {
        match self {
            Node::Street(street) => street.update_movables(t),
            Node::IONode(io_node) => {
                // create new car
                io_node.time_since_last_spawn += t;
                let mut new_cars = Vec::<Car>::new();
                // TODO: rework spawn rate
                if io_node.time_since_last_spawn >= io_node.spawn_rate {
                    // TODO: Remove and replace with proper request to
                    //  the movable server
                    // new_cars.push(Car::new())
                    match &mut io_node.movable_server {
                        Some(server) => {
                            let car_result = server.get().generate_movable(io_node.id);
                            if let Ok(car) = car_result {
                                new_cars.push(car);
                            }
                        },
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
            Node::IONode(io_node) => io_node.absorbed_cars += 1,
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
    S3
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
}
impl<Car: Movable> Crossing<Car> {
    /// Returns a new Crossing with no connections and id=0
    pub fn new() -> Crossing {
        Crossing {
            connections: CrossingConnections::new(),
            car_lane: Traversible::<Car>::new(1.0),
            id: 0,
            traffic_light_state: TrafficLightState::S0,
        }
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
    pub fn can_out_node_be_reached(&self, in_node: &IntMut<Node<Car>>, out_node: &IntMut<Node<Car>>) -> bool{
        let input_node_dir = self.connections.get_direction_for_item(InOut::IN, in_node).expect("Crossing seems not to be connected with street (input)");
        let output_node_dir = self.connections.get_direction_for_item(InOut::OUT, out_node).expect("Crossing seems not to be connected with street (output)");
        // funky stuff here
        match self.traffic_light_state{
            TrafficLightState::S0 => {
                if input_node_dir == Direction::N {
                    if output_node_dir == Direction::S || output_node_dir == Direction::W{
                        return true
                    } else {
                        return false
                    }
                } else if input_node_dir == Direction::S {
                    if output_node_dir == Direction::N || output_node_dir == Direction::E{
                        return true
                    } else {
                        return false
                    }
                } else {
                    return false
                }
            },
            TrafficLightState::S1 => {
                if input_node_dir == Direction::W {
                    if output_node_dir == Direction::S || output_node_dir == Direction::E { 
                        return true
                    } else {
                        return false
                    }
                } else if input_node_dir == Direction::E {
                    if output_node_dir == Direction::W || output_node_dir == Direction::N {
                        return true
                    } else {
                        return false
                    }
                } else {
                    return false
                }
            },
            TrafficLightState::S2 => {
                if input_node_dir == Direction::N {
                    if output_node_dir == Direction::E {
                        return true
                    } else {
                        return false
                    }
                } else if input_node_dir == Direction::S {
                    if output_node_dir == Direction::W {
                        return true
                    } else {
                        return false
                    }
                } else {
                    return false
                }
            },
            TrafficLightState::S3 => {
                if input_node_dir == Direction::W {
                    if output_node_dir == Direction::N {
                        return true
                    } else {
                        return false
                    }
                } else if input_node_dir == Direction::E {
                    if output_node_dir == Direction::S {
                        return true
                    } else {
                        return false
                    }
                } else {
                    return false
                }
            },
        }
    }
}
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
    /// time since last spawn in seconds
    pub time_since_last_spawn: f64,
    /// Tracks how many cars have reached their destination in this node
    pub absorbed_cars: usize,
    /// To differentiate different nodes. Should be set to the positions in the
    /// list of all nodes in the simulation
    pub id: usize,
    /// The movable server used to spawn new cars
    pub movable_server: Option<IntMut<MovableServer<Car>>>
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
            id: 0,
            movable_server: None
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
    pub fn update_movables(&mut self, t: f64) -> Vec<Car> {
        self.lanes
            .iter_mut()
            .flat_map(|traversible| (*traversible).update_movables(t))
            .collect()
    }
    /// Adds a movable to the street
    ///
    /// The movable is put on the lane with the leas amount of cars
    /// This should pretty closely resemble how people drive in real life
    /// as you won't drive to the lane that has the most cars in it
    pub fn add_movable(&mut self, movable: Car) {
        // get the index of the lane with the least movables on it
        let trav_most_movables = self
            .lanes
            .iter()
            .enumerate()
            .min_by_key(|(_i, traversible)| traversible.num_movables());
        let i = match trav_most_movables {
            Some((i, _)) => i,
            None => return,
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
