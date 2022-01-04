use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::*;
use simulator::datastructs::IntMut;
use simulator::debug::build_grid_sim;
use simulator::nodes::{NodeBuilder, NodeBuilderTrait};
use simulator::{self, SimManager};
use themes::*;
use tool_systems::SelectedNode;
use wasm_bindgen::prelude::*;
mod input;
mod node_bundles;
mod simulation_display;
mod themes;
mod tool_systems;
mod toolbar;
mod user_interface;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use node_bundles::node_render;

use crate::node_bundles::{CrossingBundle, IONodeBundle, StreetBundle};

#[derive(PartialEq)]
pub enum Theme {
    Light,
    Dark,
}
impl Default for Theme {
    fn default() -> Self {
        Theme::Light
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
const CONNECTION_CIRCLE_DIST_FROM_MIDDLE: f32 = 10.0;

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
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
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(UITheme::dark()) // Theme
        .insert_resource(CurrentTheme::DARK) // Theme
        .insert_resource(bevy::input::InputSystem)
        .add_system(user_interface::ui_example.system())
        .add_system_to_stage(CoreStage::PreUpdate, mark_under_cursor.system())
        // .add_system(color_under_cursor.system())
        //.add_system(rotation_test.system())
        .add_system(input::keyboard_movement.system())
        .add_system(input::mouse_panning.system())
        .add_system(recolor_nodes.system())
        .add_system(debug_status_updates.system())
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
        .add_system(simulation_display::display_cars.system())
        // .add_system_set(
        //     SystemSet::new()
        //         .with_run_criteria(simulation_display::run_if_simulating.system())
        //         .with_system(simulation_display::display_cars.system()),
        // )
        .run();
}

fn debug_status_updates(sim_manager: Res<SimManager>) {
    let report = sim_manager.get_status_updates();
    if let Some(r) = report {
        let update: String = r
            .values()
            .map(|s| s.iter().map(|s| s.position.to_string()))
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
    *new_builder = build_grid_sim(side_len as u32);
    info!("spawning node grid");

    let calc_y = |ie| ((side_len - ie / side_len) * spacing) as f32;
    let calc_x = |ie| (ie % side_len * spacing) as f32; // - 4. * (spacing as f32);

    new_builder
        .nodes
        .iter()
        .enumerate()
        .for_each(|(i, n_builder)| {
            match &*(*n_builder).get() {
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
                            let index_in = conn_in.upgrade().get().get_id();
                            let index_out = conn_out.upgrade().get().get_id();
                            let pos_j = Vec2::new(calc_x(index_in), calc_y(index_in));
                            let pos_i = Vec2::new(calc_x(index_out), calc_y(index_out));
                            commands.spawn_bundle(StreetBundle::new(
                                i,
                                n_builder,
                                pos_i,
                                pos_j,
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
