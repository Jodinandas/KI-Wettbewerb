use bevy::prelude::*;
use bevy_egui::egui::Visuals;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use sim_manager::SimManager;
use simulator;
use simulator::datastructs::IntMut;
use simulator::debug::build_grid_sim;
use simulator::nodes::{NodeBuilder, NodeBuilderTrait};
use toolbar::ToolType;
use wasm_bindgen::prelude::*;
mod user_interface;
use bevy::input::mouse::{MouseMotion, MouseWheel};
mod sim_manager;
mod toolbar;
mod node_bundles;
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

#[derive(Default)]
struct EguiDimension {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}

#[derive(Default)]
pub struct UIState {
    toolbar: toolbar::Toolbar,
    mode: UIMode,
    prev_mode: Option<UIMode>,
    selected_node: Option<NodeBuilderRef>,
    sidepanel_dimensions: EguiDimension,
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

/// This struct stores information about the visual style of the application
///
/// the colors of the simulator can be defined with fields like background
///
/// to change the visuals of the rest of the frontend, use the `egui_visuals` field
#[derive(Default)]
pub struct UITheme {
    background: Color,
    io_node: Color,
    street: Color,
    crossing: Color,
    highlight: Color,
    egui_visuals: Visuals,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CurrentTheme {
    LIGHT,
    DARK,
}

impl UITheme {
    pub fn light() -> UITheme {
        UITheme {
            background: Color::rgb(255.0, 255.0, 255.0),
            io_node: Color::rgb(0.0, 200.0, 0.0),
            street: Color::rgb(100., 50., 0.),
            crossing: Color::rgb(0.0, 200.0, 0.0),
            highlight: Color::rgb(255.0, 0.0, 0.0),
            egui_visuals: Visuals::light(),
        }
    }
    pub fn dark() -> UITheme {
        UITheme {
            background: Color::rgb(0.0, 0.0, 0.0),
            io_node: Color::rgb(200.0, 200.0, 0.0),
            street: Color::rgb(255., 255., 255.),
            crossing: Color::rgb(200.0, 200.0, 0.0),
            highlight: Color::rgb(255.0, 0.0, 0.0),
            egui_visuals: Visuals::dark(),
        }
    }
    pub fn from_enum(theme: &CurrentTheme) -> UITheme {
        match theme {
            CurrentTheme::LIGHT => UITheme::light(),
            CurrentTheme::DARK => UITheme::dark(),
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
const GRID_SIDE_LENGTH: usize = 70;
const STREET_THICKNESS: f32 = 5.0;
// const STREET_SPACING: usize = 20;
const CROSSING_SIZE: f32 = 20.0;
const IONODE_SIZE: f32 = 20.0;

const PAN_SPEED: f32 = 10.0;
const MIN_X: f32 = 300.0;
const MAX_X: f32 = 100.0;

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
        //.add_system(rotation_test.system())
        .add_system(keyboard_movement.system())
        .add_system(mouse_panning.system())
        .add_system(toolbarsystem.system())
        .run();
}


fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(Camera);
}

/// Because it is not possible (at least to our knowledge) to query the
///  start and end position of the line as a Shape bundle, we store the
///  line positions seperatly
pub struct StreetLinePosition(Vec2, Vec2);

/// Holds an IntMut (interior mutability) for a nodebuilder
#[derive(Debug, Clone)]
pub struct NodeBuilderRef(IntMut<NodeBuilder>);

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

struct Camera;
// pans canvas
fn keyboard_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let speed: f32 = PAN_SPEED;
    for mut transform in camera.iter_mut() {
        let s: Vec3 = transform.scale;
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += speed * s.x;
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= speed * s.x;
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += speed * s.y;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= speed * s.y;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            transform.scale += Vec3::from((0.1 * s.x, 0.1 * s.y, 0.0));
        }
        if keyboard_input.pressed(KeyCode::E) {
            transform.scale -= Vec3::from((0.1 * s.x, 0.1 * s.y, 0.0));
        }
    }
}
fn toolbarsystem(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut sim_manager: ResMut<SimManager>,
    mut shapes: Query<(
        Entity,
        &Transform,
        &NodeType,
        &SimulationID,
        &NodeBuilderRef,
    )>,
    mut uistate: ResMut<UIState>,
    mut theme: ResMut<UITheme>,
    mut commands: Commands,
    mut camera: Query<&Transform, With<Camera>>,
) {
    if let Some(tool) = uistate.toolbar.get_selected() {
        match tool.get_type() {
            ToolType::Pan => {
                // nothing. this conde is located in mouse_panning.
            },
            ToolType::DeleteNode => { 
                // select nearest object
                // get position of mouse click on screen
                let click_pos = handle_mouse_clicks(&mouse_input, &windows);
                // only recognize click if not in gui
                if let Some(pos) = click_pos {
                    let (closest_shape, prev_selection) = intersect_shapes_with_click(pos, windows, &shapes, &mut uistate, camera);
                    if let Some(closest_shape) = closest_shape {
                        uistate.selected_node = None;
                        if let Ok(sim_builder) = sim_manager.modify_sim_builder() {
                            
                            let removed_nodes = sim_builder.remove_node_and_connected_by_id(closest_shape.sim_index.0).expect("Unable to remove node");
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
                let click_pos = handle_mouse_clicks(&mouse_input, &windows);
                // only recognize click if not in gui
                if let Some(pos) = click_pos {
                    let (closest_shape, prev_selection) = intersect_shapes_with_click(pos, windows, &shapes, &mut uistate, camera);
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

fn intersect_shapes_with_click<'a>(click_pos: Vec2,
    windows: Res<Windows>,
    shapes: &'a Query<(
        Entity,
        &Transform,
        &NodeType,
        &SimulationID,
        &NodeBuilderRef,
    )>,
    uistate: &mut ResMut<UIState>,
    mut camera: Query<&Transform, With<Camera>>,
) -> (Option<ShapeClicked<'a>>, Option<ShapeClicked<'a>>) {
    let mut closest_shape: Option<ShapeClicked<'a>> = None;
    let mut prev_selection: Option<ShapeClicked> = None;
    // println!("{:?}", click_pos);
    if let Some(camera_transform) = camera.iter().next() {
        // camera scaling factor
        let scaling = camera_transform.scale.x;
        // get position of 0,0 of world coordinate system in screen coordinates
        let midpoint_screenspace = (get_primary_window_size(&windows) / 2.0)
            - (Vec2::new(
                camera_transform.translation.x,
                camera_transform.translation.y,
            )) / scaling;
        let shape_dists = shapes.iter().map(
            |(entity, transform, node_type, sim_index, node_builder_ref)| {
                // get shape position in scren coordinates
                let position = midpoint_screenspace
                    + (Vec2::new(
                        transform.translation.x,
                        transform.translation.y,
                    )) / scaling;
                // calculate distance, squared to improve performance so does not need to be rooted
                let dist = (position - click_pos).length_squared();
                let shape_clicked = ShapeClicked{
                    dist,
                    entity,
                    transform,
                    node_type,
                    sim_index,
                    node_builder_ref,
                };
                // check if this entity is the one currently selected
                //  (we need to change the color later on when unselecting it)
                if let Some(selected_nb) = &uistate.selected_node {
                    if selected_nb.0 == shape_clicked.node_builder_ref.0 {
                        prev_selection =
                            Some(shape_clicked.clone());
                    }
                }
                shape_clicked
            },
        );
        shape_dists.for_each(| shape_information| {
            if let Some(old_nearest) = &closest_shape
            {
                if shape_information.dist < old_nearest.dist {
                    closest_shape = Some(shape_information);
                }
            } else {
                closest_shape = Some(shape_information);
            }
        });
    }
    (closest_shape, prev_selection)
}

// used by the code that check if a shape was clicked
#[derive(Clone)]
struct ShapeClicked<'a> {
    pub dist: f32,
    pub entity: Entity,
    pub transform: &'a Transform,
    pub node_type: &'a NodeType,
    pub sim_index: &'a SimulationID,
    pub node_builder_ref: &'a NodeBuilderRef
}

// for selection
fn handle_mouse_clicks(
    mouse_input: &Res<Input<MouseButton>>,
    windows: &Res<Windows>,
) -> Option<Vec2> {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = win.cursor_position() {
            if (pos.x > MIN_X) && (get_primary_window_size(&windows).x > (MAX_X + pos.x)) {
                return win.cursor_position();
            }
        }
    }
    None
}
fn movement_within_bounds(
    mouse_input: &Res<Input<MouseButton>>,
    windows: &Res<Windows>,
    mouse_button: &MouseButton,
) -> bool {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.pressed(*mouse_button) {
        if let Some(pos) = win.cursor_position() {
            if (pos.x > MIN_X) && (get_primary_window_size(&windows).x > (MAX_X + pos.x)) {
                return true;
            }
        }
        return false;
    }
    // when scrolling, should not terminate
    true
}
fn mouse_panning(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    uistate: Res<UIState>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    // change input mapping for orbit and panning here
    let pan_button = MouseButton::Left;
    if movement_within_bounds(&input_mouse, &windows, &pan_button) {
        let mut pan = Vec2::ZERO;
        let mut rotation_move = Vec2::ZERO;
        let mut scroll = 0.0;
        if input_mouse.pressed(pan_button) {
            // Pan only if we're not rotating at the moment
            for ev in ev_motion.iter() {
                pan += ev.delta;
            }
        }
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }

        for mut transform in camera.iter_mut() {
            if pan.length_squared() > 0.0 && uistate.toolbar.get_tooltype() == ToolType::Pan {
                let window = get_primary_window_size(&windows);
                pan = pan * Vec2::new(transform.scale.x, transform.scale.y);
                transform.translation += Vec3::new(-pan.x, pan.y, 0.0);
            } else if scroll.abs() > 0.0 {
                let scr = f32::powf(1.1, scroll);
                transform.scale *= Vec3::new(scr, scr, 1.0);
            }
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
