use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::{datastructs::IntMut, nodes::{NodeBuilder, NodeBuilderTrait}};

use crate::{SimulationID, NodeType, StreetLinePosition, NodeBuilderRef};

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
    node_type: NodeType,
    /// a reference to the NodeBuilder
    node_builder_ref: NodeBuilderRef
}

impl CrossingBundle {
    pub fn new(id: usize, node_builder: &IntMut<NodeBuilder>, pos: Vec2, color: Color) -> CrossingBundle{
        let nbr = NodeBuilderRef(node_builder.clone());
        CrossingBundle {
            shape: node_render::crossing(pos, color),
            sim_id: SimulationID(id),
            node_type: NodeType::CROSSING,
            node_builder_ref: nbr
        }
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
    position: StreetLinePosition,
    /// a reference to the NodeBuilder
    node_builder_ref: NodeBuilderRef
}

impl StreetBundle {
    pub fn new(id: usize, node_builder: &IntMut<NodeBuilder>, start: Vec2, end: Vec2, color: Color) -> StreetBundle {
        let nbr = NodeBuilderRef(node_builder.clone());
        StreetBundle {
            shape: node_render::street(start,end, color),
            sim_id: SimulationID(id),
            node_type: NodeType::STREET,
            node_builder_ref: nbr,
            position: StreetLinePosition(start, end),
        }
    }
}


#[derive(Bundle)]
/// This is the way IONodes are saved in the frontend
/// 
/// The bundle contains Information that is relevant to the frontend and a reference
/// to the [simulator::SimulatorBuilder]
pub struct IONodeBundle {
    #[bundle]
    /// The actual shape that is displayed
    shape: ShapeBundle,
    /// Unique id in the SimulatorBuilder
    sim_id: SimulationID,
    /// The Node type (ALWAYS IONODE)
    node_type: NodeType,
    /// a reference to the NodeBuilder
    node_builder_ref: NodeBuilderRef
}

impl IONodeBundle {
    pub fn new(id: usize, node_builder: &IntMut<NodeBuilder>, pos: Vec2, color: Color) -> IONodeBundle{
        println!("Creating ref");
        let nbr = NodeBuilderRef(node_builder.clone());
        println!("Created ref");
        IONodeBundle {
            shape: node_render::io_node(pos, color),
            sim_id: SimulationID(id),
            node_type: NodeType::CROSSING,
            node_builder_ref: nbr
        }
    }
}

pub mod node_render {
    use bevy::{
        math::Vec2,
        prelude::{Color, Transform},
    };
    use bevy_prototype_lyon::{
        entity::ShapeBundle,
        prelude::{DrawMode, FillOptions, GeometryBuilder, ShapeColors, StrokeOptions},
        shapes,
    };

    use crate::{CROSSING_SIZE, IONODE_SIZE, STREET_THICKNESS};

    pub fn crossing(pos: Vec2, color: Color) -> ShapeBundle {
        let rect = shapes::Rectangle {
            width: CROSSING_SIZE,
            height: CROSSING_SIZE,
            ..shapes::Rectangle::default()
        };
        GeometryBuilder::build_as(
            &rect,
            ShapeColors::outlined(color, Color::WHITE),
            DrawMode::Fill(FillOptions::default()), //DrawMode::Outlined {
            //    fill_options: FillOptions::default(),
            //    outline_options: StrokeOptions::default().with_line_width(10.0)
            //}
            Transform::from_xyz(pos.x, pos.y, 0.),
        )
    }

    pub fn io_node(pos: Vec2, color: Color) -> ShapeBundle {
        let test_shape = shapes::Circle {
            radius: IONODE_SIZE,
            ..shapes::Circle::default()
        };
        GeometryBuilder::build_as(
            &test_shape,
            ShapeColors::outlined(color, Color::WHITE),
            DrawMode::Fill(FillOptions::default()), //DrawMode::Outlined {
            //    fill_options: FillOptions::default(),
            //    outline_options: StrokeOptions::default().with_line_width(10.0)
            //}
            Transform::from_xyz(pos.x, pos.y, 0.),
        )
    }
    pub fn street(p1: Vec2, p2: Vec2, color: Color) -> ShapeBundle {
        let line = shapes::Line(p1, p2);
        GeometryBuilder::build_as(
            &line,
            ShapeColors::outlined(color, color),
            //DrawMode::Fill(FillOptions::default()),
            DrawMode::Outlined {
                fill_options: FillOptions::default(),
                outline_options: StrokeOptions::default().with_line_width(STREET_THICKNESS),
            },
            Transform::default(), // Transform::from_xyz(calc_x(i), calc_y(i), 0.0)
        )
    }
}