use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;

use crate::{SimulationID, NodeType, StreetLinePosition};

#[derive(Bundle)]
/// This is the way Crossings are saved in the frontend
/// 
/// The bundle contains Information that is relevant to the frontend and a reference
/// to the [simulator::SimulatorBuilder]
pub struct CrossingBundle {
    #[bundle]
    /// The actual shape that is displayed
    shape: ShapeBundle,
    /// Unique id in the SimulatorBuilder
    sim_id: SimulationID,
    /// The Node type (ALWAYS CROSSING)
    node_type: NodeType
}

pub struct SelectionCircleNorthIn {

}

#[derive(Bundle)]
pub struct ConnectionSelection {
    #[bundle]
    north: ShapeBundle,
    #[bundle]
    south: ShapeBundle,
    #[bundle]
    west: ShapeBundle,
    #[bundle]
    east: ShapeBundle,

}

impl CrossingBundle {
    pub fn new(pos: Vec2, color: Color) {

    }
}

#[derive(Bundle)]
/// This is the way Crossings are saved in the frontend
/// 
/// The bundle contains Information that is relevant to the frontend and a reference
/// to the [simulator::SimulatorBuilder]
pub struct StreetBundle {
    #[bundle]
    /// The actual shape that is displayed
    shape: ShapeBundle,
    /// Unique id in the SimulatorBuilder
    sim_id: SimulationID,
    /// The Node type (ALWAYS STREET)
    node_type: NodeType,
    /// Where the Street starts and ends.
    /// 
    /// Unfortunatly, this has to be saved seperatly, as the line
    /// start and end (to my knowledge) positions can't be read from
    /// the ShapeBundle
    position: StreetLinePosition
}