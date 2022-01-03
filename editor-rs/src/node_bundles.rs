use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::nodes::Direction;
use simulator::{datastructs::IntMut, nodes::NodeBuilder};

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
        let mut shape = node_render::crossing(pos, color);
        // Crossings should be rendered on top of streets
        shape.transform.translation.z = 1.0;
        CrossingBundle {
            shape,
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
        let nbr = NodeBuilderRef(node_builder.clone());
        let mut shape = node_render::io_node(pos, color);
        // IONodes should be rendered on top of streets
        shape.transform.translation.z = 1.0;
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

    use crate::{CONNECTION_CIRCLE_RADIUS, CROSSING_SIZE, IONODE_SIZE, STREET_THICKNESS};

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
            Transform::from_xyz(pos.x, pos.y, 10.),
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
            Transform::from_xyz(pos.x, pos.y, 10.),
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
    Middle,
}

impl OutputCircle {
    pub fn as_dir(&self) -> Direction {
        match self {
            OutputCircle::N => Direction::N,
            OutputCircle::S => Direction::S,
            OutputCircle::W => Direction::W,
            OutputCircle::E => Direction::E,
            OutputCircle::Middle => Direction::N, // doesn't matter
        }
    }
}

/// used to mark the circles used to connect the inputs of crossigns
#[derive(Clone, Copy)]
pub enum InputCircle {
    N,
    S,
    W,
    E,
    Middle,
}
impl InputCircle {
    pub fn as_dir(&self) -> Direction {
        match self {
            InputCircle::N => Direction::N,
            InputCircle::S => Direction::S,
            InputCircle::W => Direction::W,
            InputCircle::E => Direction::E,
            InputCircle::Middle => Direction::N, // doesn't matter
        }
    }
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
    pub shape: ShapeBundle,
}

impl ConnectorCircleIn {
    pub fn new(ctype: InputCircle, color: Color) -> ConnectorCircleIn {
        // depending on the type, move the connector to a specific position on the crossing
        let offset = match ctype {
            InputCircle::N => Vec2::new(-CROSSING_SIZE / 4.0, CROSSING_SIZE / 2.0),
            InputCircle::S => Vec2::new(CROSSING_SIZE / 4.0, -CROSSING_SIZE / 2.0),
            InputCircle::W => Vec2::new(-CROSSING_SIZE / 2.0, -CROSSING_SIZE / 4.0),
            InputCircle::E => Vec2::new(CROSSING_SIZE / 2.0, CROSSING_SIZE / 4.0),
            InputCircle::Middle => Vec2::ZERO,
        };
        let mut shape_bundle = node_render::connector(offset, color);
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
    pub shape: ShapeBundle,
}

impl ConnectorCircleOut {
    pub fn new(ctype: OutputCircle, color: Color) -> ConnectorCircleOut {
        // depending on the type, move the connector to a specific position on the crossing
        let offset = match ctype {
            OutputCircle::N => Vec2::new(CROSSING_SIZE / 4.0, CROSSING_SIZE / 2.0),
            OutputCircle::S => Vec2::new(-CROSSING_SIZE / 4.0, -CROSSING_SIZE / 2.0),
            OutputCircle::W => Vec2::new(-CROSSING_SIZE / 2.0, CROSSING_SIZE / 4.0),
            OutputCircle::E => Vec2::new(CROSSING_SIZE / 2.0, -CROSSING_SIZE / 4.0),
            OutputCircle::Middle => Vec2::ZERO,
        };
        let mut shape_bundle = node_render::connector(offset, color);
        // should always be in the foreground
        shape_bundle.transform.translation.z += 10.0;
        ConnectorCircleOut {
            ctype,
            shape: shape_bundle,
        }
    }
}
