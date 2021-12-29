use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::{
    datastructs::IntMut,
    nodes::{NodeBuilder, NodeBuilderTrait},
};

use crate::{NodeBuilderRef, NodeType, SimulationID, StreetLinePosition, CROSSING_SIZE};

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
    node_builder_ref: NodeBuilderRef,
}

impl CrossingBundle {
    pub fn new(
        id: usize,
        node_builder: &IntMut<NodeBuilder>,
        pos: Vec2,
        color: Color,
    ) -> CrossingBundle {
        let nbr = NodeBuilderRef(node_builder.clone());
        CrossingBundle {
            shape: node_render::crossing(pos, color),
            sim_id: SimulationID(id),
            node_type: NodeType::CROSSING,
            node_builder_ref: nbr,
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
    node_builder_ref: NodeBuilderRef,
}

impl StreetBundle {
    pub fn new(
        id: usize,
        node_builder: &IntMut<NodeBuilder>,
        start: Vec2,
        end: Vec2,
        color: Color,
    ) -> StreetBundle {
        let nbr = NodeBuilderRef(node_builder.clone());
        StreetBundle {
            shape: node_render::street(start, end, color),
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
    node_builder_ref: NodeBuilderRef,
}

impl IONodeBundle {
    pub fn new(
        id: usize,
        node_builder: &IntMut<NodeBuilder>,
        pos: Vec2,
        color: Color,
    ) -> IONodeBundle {
        println!("Creating ref");
        let nbr = NodeBuilderRef(node_builder.clone());
        println!("Created ref");
        IONodeBundle {
            shape: node_render::io_node(pos, color),
            sim_id: SimulationID(id),
            node_type: NodeType::CROSSING,
            node_builder_ref: nbr,
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

    use crate::{CROSSING_SIZE, IONODE_SIZE, STREET_THICKNESS, CONNECTION_CIRCLE_RADIUS};

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
    pub fn connector(pos: Vec2, color: Color) -> ShapeBundle {
        let circle = shapes::Circle {
            radius: CONNECTION_CIRCLE_RADIUS,
            ..shapes::Circle::default()
        };
        GeometryBuilder::build_as(
            &circle,
            ShapeColors::outlined(color, Color::WHITE),
            DrawMode::Fill(FillOptions::default()), //DrawMode::Outlined {
            //    fill_options: FillOptions::default(),
            //    outline_options: StrokeOptions::default().with_line_width(10.0)
            //}
            Transform::from_xyz(pos.x, pos.y, 0.),
        )
    }
}


/// used to mark the circles used to connect the outputs of crossings
#[derive(Clone, Copy)]
pub enum OutputCircle {
    N,
    S,
    W,
    E,
}

/// used to mark the circles used to connect the inputs of crossigns
#[derive(Clone, Copy)]
pub enum InputCircle {
    N,
    S,
    W,
    E,
}

/// These circles are displayed when hovering over a crossing when
/// having selected the AddStreet Tool. They are displayed as a way
/// to connect specific outputs  /inputs of crossings
/// 
/// This Bundle should be used when spawning an entity as a child of 
/// a crossing
#[derive(Bundle)]
pub struct ConnectorCircleIn {
    pub ctype: InputCircle,
    #[bundle]
    pub shape: ShapeBundle
}

impl ConnectorCircleIn {
    pub fn new(ctype: InputCircle, parent_pos: &Transform, color: Color) -> ConnectorCircleIn {
        let mut raw_pos = Vec2::new(parent_pos.translation.x, parent_pos.translation.y);
        // depending on the type, move the connector to a specific position on the crossing        
        match ctype {
            InputCircle::N => {
                raw_pos.y += CROSSING_SIZE / 2.0;
                raw_pos.x -= CROSSING_SIZE / 4.0;
            },
            InputCircle::S => {
                raw_pos.y -= CROSSING_SIZE / 2.0;
                raw_pos.x += CROSSING_SIZE / 4.0;
            },
            InputCircle::W => {
                raw_pos.x -= CROSSING_SIZE / 2.0;
                raw_pos.y -= CROSSING_SIZE / 4.0;
            },
            InputCircle::E => {
                raw_pos.x += CROSSING_SIZE / 2.0;
                raw_pos.y += CROSSING_SIZE / 4.0;
            } 
        }
        let mut shape_bundle = node_render::connector(raw_pos, color);
        // should always be in the foreground
        shape_bundle.transform.translation.z += 10.0;
        ConnectorCircleIn {
            ctype,
            shape: shape_bundle,
        }
    }
}

/// These circles are displayed when hovering over a crossing when
/// having selected the AddStreet Tool. They are displayed as a way
/// to connect specific outputs  /inputs of crossings
/// 
/// This Bundle should be used when spawning an entity as a child of 
/// a crossing
/// 
/// ## Why have separate Bundles for In- and OutConnectors?
///  While this may cause come redundant code, it is way easer to
///  query for a specific Component ([ConnectorCircleIn]/[ConnectorCircleOut])
///  than to query for an enum with both options as variants. With the variants,
///  one would have to query for both in and out circles and then match them.
#[derive(Bundle)]
pub struct ConnectorCircleOut {
    pub ctype: OutputCircle,
    #[bundle]
    pub shape: ShapeBundle
}

impl ConnectorCircleOut {
    pub fn new(ctype: OutputCircle, parent_pos: &Transform, color: Color) -> ConnectorCircleOut {
        let mut raw_pos = Vec2::new(parent_pos.translation.x, parent_pos.translation.y);
        // depending on the type, move the connector to a specific position on the crossing        
        match ctype {
            OutputCircle::N => {
                raw_pos.y += CROSSING_SIZE / 2.0;
                raw_pos.x += CROSSING_SIZE / 4.0;
            },
            OutputCircle::S => {
                raw_pos.y -= CROSSING_SIZE / 2.0;
                raw_pos.x -= CROSSING_SIZE / 4.0;
            },
            OutputCircle::W => {
                raw_pos.x -= CROSSING_SIZE / 2.0;
                raw_pos.y += CROSSING_SIZE / 4.0;
            },
            OutputCircle::E => {
                raw_pos.x += CROSSING_SIZE / 2.0;
                raw_pos.y -= CROSSING_SIZE / 4.0;
            } 
        }
        let mut shape_bundle = node_render::connector(raw_pos, color);
        // should always be in the foreground
        shape_bundle.transform.translation.z += 10.0;
        ConnectorCircleOut {
            ctype,
            shape: shape_bundle,
        }
    }
}