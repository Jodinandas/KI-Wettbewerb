use bevy::prelude::*;
use bevy::render::mesh::VertexAttributeValues;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use sim_manager::SimManager;
use simulator;
use simulator::datastructs::IntMut;
use simulator::debug::build_grid_sim;
use simulator::nodes::{NodeBuilder, NodeBuilderTrait};
use themes::*;
use toolbar::ToolType;
use wasm_bindgen::prelude::*;
mod user_interface;
mod sim_manager;
mod toolbar;
mod node_bundles;
mod themes;
mod input;
use node_bundles::node_render;
use std::time;

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

#[derive(Default)]
pub struct UIState {
    toolbar: toolbar::Toolbar,
    mode: UIMode,
    prev_mode: Option<UIMode>,
    selected_node: Option<NodeBuilderRef>,
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


#[derive(Debug)]
pub enum NodeType {
    CROSSING,
    IONODE,
    STREET,
}

const GRID_NODE_SPACING: usize = 100;
const GRID_SIDE_LENGTH: usize = 7;
const STREET_THICKNESS: f32 = 5.0;
// const STREET_SPACING: usize = 20;
const CROSSING_SIZE: f32 = 20.0;
const IONODE_SIZE: f32 = 20.0;
const CONNECTION_CIRCLE_RADIUS: f32 = 10.0;
const CONNECTION_CIRCLE_DIST_FROM_MIDDLE: f32 = 10.0;



#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(ShapePlugin)
        .init_resource::<UIState>()
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
        .add_system(mark_under_cursor.system())
        //.add_system(color_under_cursor.system())
        //.add_system(rotation_test.system())
        .add_system(input::keyboard_movement.system())
        .add_system(input::mouse_panning.system())
        .add_system(toolbarsystem.system())
        .run();
}

pub struct Camera;

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(Camera);
}

/// This system marks a node under the cursor with the [UnderCursor] component
///  this makes it easy for tools etc. to perform actions on nodes, as the
///  one under the cursor can be queried with the [UnderCursor] component
fn mark_under_cursor (
        mut commands: Commands,
        windows: Res<Windows>,
        queries: QuerySet<(
            // previously selected nodes that need to be unselected
            Query<(Entity), (With<NodeType>, With<UnderCursor>)>,
            // candidates for selection
            Query<(
                Entity,
                &Transform,
                &NodeType,
                &SimulationID,
                &NodeBuilderRef,
            )>,
            // the camera 
            Query<&Transform, With<Camera>>
        )>,
        mut uistate: ResMut<UIState>,
        theme: Res<UITheme>,
) {
    // unselect previously selected
    queries.q0().for_each( | entity | {
        commands.entity(entity).remove::<UnderCursor>();
    });
    let now= time::Instant::now();
    let window = windows.get_primary().unwrap();
    let mouse_pos = window.cursor_position();
    if let Some(pos) = mouse_pos {
        let shape = input::get_shape_under_mouse(pos, windows, &queries.q1(), &mut uistate, queries.q2());
        if let Some(shape) = shape {
            // mark it 
            commands
                .entity(shape.entity)
                .insert(UnderCursor);
        }
    }
}

pub fn color_under_cursor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    query: Query<(&Handle<Mesh>, &NodeType), With<UnderCursor>>,
) {
    query.for_each( | (mesh_handle, node_type) | {
        repaint_node(mesh_handle, Color::rgb(0.0, 0.0, 0.0), &mut meshes);
    } );
}

/// Because it is not possible (at least to our knowledge) to query the
///  start and end position of the line as a Shape bundle, we store the
///  line positions seperatly
pub struct StreetLinePosition(Vec2, Vec2);

/// Holds an IntMut (interior mutability) for a nodebuilder
#[derive(Debug, Clone)]
pub struct NodeBuilderRef(IntMut<NodeBuilder>);

pub fn repaint_node(
    mesh_handle: &Handle<Mesh>,
    color: Color,
    meshes: &mut ResMut<Assets<Mesh>>,
)  {
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
    println!("Build Grid");

    let calc_x = |ie| (ie / side_len * spacing) as f32;
    let calc_y = |ie| (ie % side_len * spacing) as f32; // - 4. * (spacing as f32);

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
                    commands
                        .spawn_bundle(CrossingBundle::new(i, n_builder, Vec2::new(x, y), theme.crossing));
                }

                NodeBuilder::IONode(_io_node) => {
                    let x = calc_x(i);
                    let y = calc_y(i);
                    commands
                        .spawn_bundle(IONodeBundle::new(i, n_builder, Vec2::new(x, y), theme.io_node));
                }
                NodeBuilder::Street(street) => {
                    // println!("   type=Street");
                    if let Some(conn_in) = &street.conn_in {
                        if let Some(conn_out) = &street.conn_out {
                            let index_in = conn_in.upgrade().get().get_id();
                            let index_out = conn_out.upgrade().get().get_id();
                            let pos_j = Vec2::new(calc_x(index_in), calc_y(index_in));
                            let pos_i = Vec2::new(calc_x(index_out), calc_y(index_out));
                            commands
                                .spawn_bundle(StreetBundle::new(i, n_builder, pos_i, pos_j, theme.street));
                        }
                    }
                    return;
                }
            }
        });
    println!("built Grid");
}

fn toolbarsystem(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut sim_manager: ResMut<SimManager>,
    shapes: Query<(
        Entity,
        &Transform,
        &NodeType,
        &SimulationID,
        &NodeBuilderRef,
    )>,
    mut uistate: ResMut<UIState>,
    theme: ResMut<UITheme>,
    mut commands: Commands,
    camera: Query<&Transform, With<Camera>>,
) {
    if let Some(tool) = uistate.toolbar.get_selected() {
        match tool.get_type() {
            ToolType::Pan => {
                // nothing. this conde is located in mouse_panning.
            },
            ToolType::DeleteNode => { 
                // select nearest object
                // get position of mouse click on screen
                let click_pos = input::handle_mouse_clicks(&mouse_input, &windows);
                // only recognize click if not in gui
                if let Some(pos) = click_pos {
                    let (closest_shape, prev_selection) = input::get_nearest_shapes(pos, windows, &shapes, &mut uistate, &camera);
                    if let Some(closest_shape) = closest_shape {
                        uistate.selected_node = None;
                        if let Ok(sim_builder) = sim_manager.modify_sim_builder() {
                            
                            let removed_nodes = sim_builder.remove_node_and_connected_by_id(closest_shape.sim_id.0).expect("Unable to remove node");
                            let indices_to_remove: Vec<usize> = removed_nodes.iter().map(| node | node.get().get_id()).collect();
                            for (entity, _, _, sim_index, _) in shapes.iter() {
                                if indices_to_remove.contains(&sim_index.0) {
                                    
                                    commands.entity(entity).despawn_recursive();
                                }
                            }
                        }
                    }
                }
            },
            ToolType::Select => {
                // select nearest object
                // get position of mouse click on screen
                let click_pos = input::handle_mouse_clicks(&mouse_input, &windows);
                // only recognize click if not in gui
                if let Some(pos) = click_pos {
                    let (closest_shape, prev_selection) = input::get_nearest_shapes(pos, windows, &shapes, &mut uistate, &camera);
                    if let Some(closest_shape) = closest_shape {
                        if let Some(prev_selection) =
                            prev_selection
                        {
                            let pos = prev_selection.transform.translation;
                            let new_shape_bundle = match prev_selection.node_type {
                                NodeType::CROSSING => {
                                    node_render::crossing(Vec2::new(pos.x, pos.y), theme.crossing)
                                }
                                NodeType::IONODE => {
                                    node_render::io_node(Vec2::new(pos.x, pos.y), theme.io_node)
                                }
                                NodeType::STREET => {
                                    todo!("Street selection is not implemented yet!")
                                }
                            };
                            commands
                                .entity(prev_selection.entity)
                                .remove_bundle::<ShapeBundle>()
                                .insert_bundle(new_shape_bundle);
                        }
                        uistate.selected_node = Some(closest_shape.node_builder_ref.clone());
                        let pos = closest_shape.transform.translation;
                        let new_shape_bundle = match closest_shape.node_type {
                            NodeType::CROSSING => {
                                node_render::crossing(Vec2::new(pos.x, pos.y), theme.highlight)
                            }
                            NodeType::IONODE => {
                                node_render::io_node(Vec2::new(pos.x, pos.y), theme.highlight)
                            }
                            NodeType::STREET => {
                                todo!("Street selection is not implemented yet!")
                            }
                        };
                        commands
                            .entity(closest_shape.entity)
                            .remove_bundle::<ShapeBundle>()
                            .insert_bundle(new_shape_bundle);
                    }
                    };
            }
            ToolType::None => (),
            ToolType::AddStreet => (),
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

pub struct NodeComponent;

#[derive(Debug, Clone, Copy)]
pub struct SimulationID(usize);
