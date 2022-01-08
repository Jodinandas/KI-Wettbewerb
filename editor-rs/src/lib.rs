use std::sync::MutexGuard;

use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::{EguiPlugin, EguiContext};
use bevy_prototype_lyon::prelude::*;
use simulator::datastructs::IntMut;
use simulator::debug::build_grid_sim;
use simulator::nodes::{NodeBuilder, NodeBuilderTrait, InOut};
use simulator::{self, SimManager};
use themes::*;
use tool_systems::SelectedNode;
use user_interface::repaint_ui;
use wasm_bindgen::prelude::*;
mod input;
mod node_bundles;
mod simulation_display;
mod themes;
mod tool_systems;
mod toolbar;
mod user_interface;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};
use node_bundles::node_render;

use crate::node_bundles::{CrossingBundle, IONodeBundle, StreetBundle};

#[derive(PartialEq)]
pub enum Theme {
    Light,
    Dracula,
}
impl Default for Theme {
    fn default() -> Self {
        Theme::Dracula
    }
}

/*
#[derive(Default)]
pub struct Apps {
    editor: TODO,
    control_room: TODO,
}
*/

#[derive(Clone, PartialEq)]
pub enum UIMode {
    Editor,
    Simulator,
    Preferences,
}
impl Default for UIMode {
    fn default() -> Self {
        UIMode::Editor
    }
}
impl UIMode {
    pub fn toggle(&mut self) {
        *self = match self {
            UIMode::Editor => UIMode::Simulator,
            UIMode::Simulator => UIMode::Preferences,
            UIMode::Preferences => UIMode::Editor,
        }
    }
}

// /// The node that the mouse is currently hovering over
// #[derive(Default)]
// pub struct NodeUnderCursor {
//     pub entity: Option<Entity>,
// }
pub struct UnderCursor;

pub enum AddStreetStage {
    SelectingOutput,
    SelectingInput,
}
impl Default for AddStreetStage {
    fn default() -> Self {
        AddStreetStage::SelectingOutput
    }
}

#[derive(Default)]
pub struct UIState {
    toolbar: toolbar::Toolbar,
    mode: UIMode,
    prev_mode: Option<UIMode>,
}
impl UIState {
    /// if there was a previous mode, switch to it
    pub fn to_prev_mode(&mut self) {
        if let Some(prev) = &self.prev_mode {
            let temp = self.mode.clone();
            self.mode = prev.clone();
            self.prev_mode = Some(temp);
        }
    }
    pub fn new_mode(&mut self, mode: UIMode) {
        if mode != self.mode {
            self.prev_mode = Some(self.mode.clone());
            self.mode = mode;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    CROSSING,
    IONODE,
    STREET,
}

const GRID_NODE_SPACING: usize = 100;
const GRID_SIDE_LENGTH: usize = 3;
const STREET_THICKNESS: f32 = 5.0;
// const STREET_SPACING: usize = 20;
const CROSSING_SIZE: f32 = 20.0;
const IONODE_SIZE: f32 = 20.0;
const CONNECTION_CIRCLE_RADIUS: f32 = 5.0;
const CONNECTOR_DISPLAY_RADIUS: f32 = 30.0;
const CONNECTION_CIRCLE_DIST_FROM_MIDDLE: f32 = CROSSING_SIZE/2.0 + 10.0;
/// the first value is where the street is placed in the direction of the connection
/// the second value is how much the street is shifted to the side
const STREET_OFFSET: [f32; 2] = [CROSSING_SIZE/2.0, CROSSING_SIZE/4.0];
const CAR_Z: f32 = 20.0;
const CAR_SIZE: f32 = 1.5;

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.add_plugins_with(DefaultPlugins, | group | { group.disable::<bevy::log::LogPlugin>() } )
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .init_resource::<UIState>()
        .init_resource::<AddStreetStage>()
        //app.add_plugins(bevy_webgl2::DefaultPlugins);
        // when building for Web, use WebGL2 rendering
        //#[cfg(target_arch = "wasm32")]
        //app.add_plugin(bevy_webgl2::WebGL2Plugin);
        .insert_resource(SimManager::new())
        .add_startup_system(spawn_node_grid.system())
        .add_startup_system(spawn_camera.system())
        .insert_resource(UITheme::dracula()) // Theme
        .insert_resource(CurrentTheme::DRACULA) // Theme
        .insert_resource(ClearColor(UITheme::dracula().background))
        .insert_resource(bevy::input::InputSystem)
        .insert_resource(first_frame{ b: true })
        .add_system(user_interface::draw_user_interface.system())
        .add_system_to_stage(CoreStage::PreUpdate, mark_under_cursor.system())
        // .add_system(color_under_cursor.system())
        //.add_system(rotation_test.system())
        .add_system(input::keyboard_movement.system())
        .add_system(input::mouse_panning.system())
        .add_system(recolor_nodes.system())
        .add_system(debug_status_updates.system())
        .add_system(toggle_theme_on_startup.system())
        // .add_system(toolbarsystem.system())
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(tool_systems::run_if_delete_node.system())
                .with_system(tool_systems::delete_node_system_simple.system()),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_run_criteria(tool_systems::run_if_add_street.system())
                .with_system(tool_systems::remove_connectors_out_of_bounds.system())
                .with_system(tool_systems::connector_clicked.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(tool_systems::run_if_select.system())
                .with_system(tool_systems::select_node.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(tool_systems::run_if_add_street.system())
                .with_system(tool_systems::generate_connectors.system())
                .with_system(tool_systems::render_new_street.system())
                .with_system(input::mark_connector_under_cursor.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(tool_systems::run_if_add_crossing.system())
                .with_system(tool_systems::add_crossing_system.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(tool_systems::run_if_add_ionode.system())
                .with_system(tool_systems::add_io_node_system.system()),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(simulation_display::run_if_simulating.system())
                .with_system(simulation_display::display_cars.system()),
        )
        .run();
}


struct first_frame{
    b: bool
}

fn toggle_theme_on_startup(commands: Commands, egui_context: ResMut<EguiContext>, mut background: ResMut<ClearColor>, nodes: Query<Entity, With<NodeType>>,
    mut theme: ResMut<UITheme>, mut current_theme: ResMut<CurrentTheme>, mut ff: ResMut<first_frame>) {
    if ff.b {
    let mut new_theme = CurrentTheme::LIGHT;
    if new_theme != *current_theme {
        *current_theme = new_theme;
        *theme = UITheme::from_enum(&new_theme);
    }
    new_theme = CurrentTheme::DRACULA;
    if new_theme != *current_theme {
        *current_theme = new_theme;
        *theme = UITheme::from_enum(&new_theme);
    }
    // repaint_ui(
    //     commands,
    //     Some(egui_context.ctx()),
    //     &mut background,
    //     &nodes,
    //     theme,
    //     );
    background.0 = theme.background;
    egui_context.ctx().set_visuals(theme.egui_visuals.clone());
    ff.b = false;
    }
}

fn debug_status_updates(sim_manager: Res<SimManager>) {
    let report = sim_manager.get_status_updates();
    if let Some(r) = report {
        let update: String = r
            .values()
            .map(|s| s.iter().map(|s| s.position.to_string() + ", "))
            .flatten()
            .collect();
        debug!("Car Status Update: {}", update);
    }
}

pub struct Camera;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(Camera);
}

pub struct NeedsRecolor;

/// recolors all nodes with the marker Component [NeedsRecolor]
pub fn recolor_nodes(
    mut commands: Commands,
    to_recolor: Query<
        (Entity, &Handle<Mesh>, &NodeType, Option<&SelectedNode>),
        With<NeedsRecolor>,
    >,
    theme: Res<UITheme>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    to_recolor.for_each(|(entity, mesh_handle, ntype, selected)| {
        // repaint the node
        let color = match selected.is_some() {
            true => theme.highlight,
            false => match ntype {
                NodeType::CROSSING => theme.crossing,
                NodeType::IONODE => theme.io_node,
                NodeType::STREET => theme.street,
            },
        };
        repaint_node(mesh_handle, color, &mut meshes);
        // remove the repaint marker
        commands.entity(entity).remove::<NeedsRecolor>();
    });
}

/// This system marks a node under the cursor with the [UnderCursor] component
///  this makes it easy for tools etc. to perform actions on nodes, as the
///  one under the cursor can be queried with the [UnderCursor] component
fn mark_under_cursor(
    mut commands: Commands,
    windows: Res<Windows>,
    queries: QuerySet<(
        // previously marked nodes that need to be unmarked
        Query<Entity, (With<NodeType>, With<UnderCursor>)>,
        // candidates for selection
        Query<(Entity, &Transform, &NodeType)>,
        // the camera
        Query<&Transform, With<Camera>>,
    )>,
) {
    // unselect previously selected
    queries.q0().for_each(|entity| {
        commands.entity(entity).remove::<UnderCursor>();
    });
    let window = windows.get_primary().unwrap();
    let mouse_pos = window.cursor_position();
    if let Some(pos) = mouse_pos {
        let shape =
            input::get_shape_under_mouse(pos, windows, &mut queries.q1().iter(), queries.q2());
        if let Some((entity, _trans, _type)) = shape {
            // mark it
            commands.entity(entity).insert(UnderCursor);
        }
    }
}

pub fn color_under_cursor(
    mut commands: Commands,
    query: Query<Entity, (With<UnderCursor>, With<NodeType>)>,
) {
    query.for_each(|entity| {
        commands
            .entity(entity)
            .insert(NeedsRecolor)
            .insert(SelectedNode);
        info!("coloring all nodes under cursor");
    });
}

/// Because it is not possible (at least to our knowledge) to query the
///  start and end position of the line as a Shape bundle, we store the
///  line positions seperatly
pub struct StreetLinePosition(Vec2, Vec2);

/// Holds an IntMut (interior mutability) for a nodebuilder
#[derive(Debug, Clone)]
pub struct NodeBuilderRef(IntMut<NodeBuilder>);

pub fn repaint_node(mesh_handle: &Handle<Mesh>, color: Color, meshes: &mut ResMut<Assets<Mesh>>) {
    let mesh = meshes.get_mut(mesh_handle).unwrap();
    let colors = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR).unwrap();
    let values = match colors {
        VertexAttributeValues::Float4(colors) => colors
            .iter()
            .map(|[_r, _g, _b, _a]| color.into())
            .collect::<Vec<[f32; 4]>>(),
        _ => vec![],
    };
    mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, values);
}

pub fn calculate_offset_from_crossing_in(street: &IntMut<NodeBuilder>, c_in: &MutexGuard<NodeBuilder>, c_out: &MutexGuard<NodeBuilder>) -> Vec2 {
    let mut offset = Vec2::ZERO;
    match &**c_in{
        NodeBuilder::IONode(io_node) => {
            // The offset can not be determined from an IONode, as the direction is unclear.
            //  Therefor, use the output as reference
            if let NodeBuilder::Crossing(crossing) = &**c_out{
                let dir = crossing.get_direction_for_item(InOut::IN, street).expect("Crossing that is set as output doesn't have street as input");
                match dir {
                    simulator::nodes::Direction::N => {
                        offset.x -= STREET_OFFSET[1];
                        offset.y += STREET_OFFSET[0];
                    },
                    simulator::nodes::Direction::S => {
                        offset.x += STREET_OFFSET[1];
                        offset.y -= STREET_OFFSET[0];
                    },
                    simulator::nodes::Direction::E => {
                        offset.x += STREET_OFFSET[0];
                        offset.y += STREET_OFFSET[1];
                    },
                    simulator::nodes::Direction::W => {
                        offset.x -= STREET_OFFSET[0];
                        offset.y -= STREET_OFFSET[1];
                    },
                }
            }
        },
        NodeBuilder::Crossing(crossing) => {
            let dir = crossing.get_direction_for_item(InOut::OUT, street).expect("Crossing that is set as input doesn't have street as output");
            match dir {
                simulator::nodes::Direction::N => {
                    offset.x += STREET_OFFSET[1];
                    offset.y += STREET_OFFSET[0];
                },
                simulator::nodes::Direction::S => {
                    offset.x -= STREET_OFFSET[1];
                    offset.y -= STREET_OFFSET[0];
                },
                simulator::nodes::Direction::E => {
                    offset.x += STREET_OFFSET[0];
                    offset.y -= STREET_OFFSET[1];
                },
                simulator::nodes::Direction::W => {
                    offset.x -= STREET_OFFSET[0];
                    offset.y += STREET_OFFSET[1];
                },
            }
        },
        NodeBuilder::Street(_) => panic!("Street connected to street!"),
    }
    offset
}

/// This function is for debugging purposes
/// It spawns a grid of nodes
///
/// It also generates the graphics for it
fn spawn_node_grid(
    mut commands: Commands,
    theme: Res<UITheme>,
    mut sim_manager: ResMut<SimManager>,
) {
    // for testing purposes
    let side_len = GRID_SIDE_LENGTH;
    let spacing = GRID_NODE_SPACING;
    let new_builder = sim_manager
        .modify_sim_builder()
        .expect("Simulation is running while trying to construct grid");
    *new_builder = build_grid_sim(side_len as u32, GRID_NODE_SPACING as f32);
    new_builder.with_delay(0).with_dt(2.0);
    info!("spawning node grid");

    let calc_y = |ie| ((side_len - ie / side_len) * spacing) as f32;
    let calc_x = |ie| (ie % side_len * spacing) as f32; // - 4. * (spacing as f32);

    new_builder
        .nodes
        .iter()
        .enumerate()
        .for_each(|(i, n_builder)| {
            match &mut *(*n_builder).get() {
                // generates the entities displaying the
                NodeBuilder::Crossing(_crossing) => {
                    let x = calc_x(i);
                    let y = calc_y(i);
                    commands.spawn_bundle(CrossingBundle::new(
                        i,
                        n_builder,
                        Vec2::new(x, y),
                        theme.crossing,
                    ));
                }

                NodeBuilder::IONode(_io_node) => {
                    let x = calc_x(i);
                    let y = calc_y(i);
                    commands.spawn_bundle(IONodeBundle::new(
                        i,
                        n_builder,
                        Vec2::new(x, y),
                        theme.io_node,
                    ));
                }
                NodeBuilder::Street(street) => {
                    if let Some(conn_in) = &street.conn_in {
                        if let Some(conn_out) = &street.conn_out {
                            let guarded_conn_in = conn_in.upgrade();
                            let guarded_conn_in = guarded_conn_in.get();
                            let guarded_conn_out = conn_out.upgrade();
                            let guarded_conn_out = guarded_conn_out.get();
                            let index_in = guarded_conn_in.get_id();
                            let index_out = guarded_conn_out.get_id();
                            let offset = calculate_offset_from_crossing_in(n_builder, &guarded_conn_in, &guarded_conn_out);
                            
                            let pos_j = Vec2::new(calc_x(index_in), calc_y(index_in)) + offset;
                            let pos_i = Vec2::new(calc_x(index_out), calc_y(index_out)) + offset;
                            // set the length in the backend
                            let len = (pos_j - pos_i).length();
                            street.lane_length = len;
                            commands.spawn_bundle(StreetBundle::new(
                                i,
                                n_builder,
                                pos_j,
                                pos_i,
                                theme.street,
                            ));
                        }
                    }
                    return;
                }
            }
        });
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

pub struct NodeComponent;

#[derive(Debug, Clone, Copy)]
pub struct SimulationID(usize);
