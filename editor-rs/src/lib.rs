use bevy::prelude::*;
use bevy_egui::egui::Visuals;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::prelude::*;
use simulator::datastructs::IntMut;
use simulator::nodes::{NodeBuilder, NodeBuilderTrait};
use simulator::debug::build_grid_sim;
use toolbar::ToolType;
use wasm_bindgen::prelude::*;
use simulator;
mod user_interface;
use bevy::input::mouse::{MouseWheel,MouseMotion};
mod toolbar;
mod sim_manager;


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
    Preferences
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
pub struct UIState {
    toolbar: toolbar::Toolbar,
    mode: UIMode,
    prev_mode: Option<UIMode>,
    selected_node: Option<NodeBuilderRef>
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
    egui_visuals: Visuals 
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
            egui_visuals: Visuals::light()
        }
    }
    pub fn dark() -> UITheme {
        UITheme {
            background: Color::rgb(0.0, 0.0, 0.0),
            io_node: Color::rgb(200.0, 200.0, 0.0),
            street: Color::rgb(255., 255., 255.),
            crossing: Color::rgb(200.0, 200.0, 0.0),
            highlight: Color::rgb(255.0, 0.0, 0.0),
            egui_visuals: Visuals::dark()
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
const GRID_SIDE_LENGTH: usize = 50;
const STREET_THICKNESS: f32 = 5.0;
// const STREET_SPACING: usize = 20;
const CROSSING_SIZE: f32 = 20.0;
const IONODE_SIZE: f32 = 20.0;

const PAN_SPEED: f32 = 10.0;

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
        .add_startup_system(spawn_simulation_builder.system())
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


mod node_render {
    use bevy::{prelude::{Color, Transform}, math::Vec2};
    use bevy_prototype_lyon::{entity::ShapeBundle, shapes, prelude::{GeometryBuilder, ShapeColors, DrawMode, FillOptions, StrokeOptions}};

    use crate::{CROSSING_SIZE, IONODE_SIZE, STREET_THICKNESS};

    pub fn crossing(x: f32, y: f32, color: Color) -> ShapeBundle {
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
            Transform::from_xyz(x, y, 0.),
        )
    }

    pub fn io_node(x: f32, y: f32, color: Color) -> ShapeBundle {
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
            Transform::from_xyz(x, y, 0.),
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
                outline_options: StrokeOptions::default()
                    .with_line_width(STREET_THICKNESS),
            },
            Transform::default(), // Transform::from_xyz(calc_x(i), calc_y(i), 0.0)
        )
    }
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(Camera);
}

/// Because it is not possible (at least to out knowledge) to query the
///  start and end position of the line as a Shape bundle, we store the
///  line positions seperatly 
pub struct StreetLinePosition(Vec2, Vec2);

/// Holds an IntMut (interior mutability) for a nodebuilder
#[derive(Debug, Clone)]
struct NodeBuilderRef(IntMut<NodeBuilder>);

/// This function spawns the simultation builder instance
/// that is later used to create simulations
///
/// It also generates the graphics for it
fn spawn_simulation_builder(mut commands: Commands, theme: Res<UITheme>) {
    // for testing purposes
    let side_len = GRID_SIDE_LENGTH;
    let spacing = GRID_NODE_SPACING;
    let new_builder = build_grid_sim(side_len as u32);
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
                    let geometry = node_render::crossing(x, y, theme.crossing);
                    commands
                        .spawn()
                        .insert_bundle(geometry)
                        .insert(SimulationIndex(i))
                        // insert direct reference to the NodeBuilder
                        .insert(NodeBuilderRef(n_builder.clone()))
                        .insert(NodeType::CROSSING);
                }

                NodeBuilder::IONode(_io_node) => {
                    let x = calc_x(i);
                    let y = calc_y(i);
                    let geometry = node_render::io_node(x, y, theme.crossing);
                    commands
                        .spawn_bundle(geometry)
                        .insert(SimulationIndex(i))
                        // insert direct reference to the NodeBuilder
                        .insert(NodeBuilderRef(n_builder.clone()))
                        .insert(NodeType::IONODE);
                }
                NodeBuilder::Street(street) => {
                    // println!("   type=Street");
                    if let Some(conn_in) = &street.conn_in {
                        if let Some(conn_out) = &street.conn_out {
                            let index_in = conn_in.upgrade().get().get_id();
                            let index_out = conn_out.upgrade().get().get_id();
                            let pos_j = Vec2::new(calc_x(index_in), calc_y(index_in));
                            let pos_i = Vec2::new(calc_x(index_out), calc_y(index_out));
                            let geometry = node_render::street(pos_i, pos_j, theme.street);
                            commands
                                .spawn_bundle(geometry)
                                .insert(SimulationIndex(i))
                                .insert(NodeType::STREET)
                                // insert direct reference to the NodeBuilder
                                .insert(NodeBuilderRef(n_builder.clone()))
                                .insert(StreetLinePosition(pos_i, pos_j));
                        }
                    }
                    return;
                }
            }
        });
    commands.insert_resource(new_builder);
    println!("built Grid");
}

struct Camera;
// pans canvas
fn keyboard_movement(keyboard_input: Res<Input<KeyCode>>, mut camera: Query<&mut Transform, With<Camera>>) {
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
fn toolbarsystem(mouse_input: Res<Input<MouseButton>>, windows: Res<Windows>, mut shapes: Query<(Entity, &Transform, &NodeType, &SimulationIndex, &NodeBuilderRef)>,
      mut uistate: ResMut<UIState>,
      mut theme: ResMut<UITheme>,
      mut commands: Commands,
      mut camera: Query<&Transform, With<Camera>>
    ){

    if let Some(tool) = uistate.toolbar.get_selected() {
        match tool.get_type() {
            ToolType::Pan => {
                // nothing. this conde is located in mouse_panning.
                }
            ToolType::Select => {
                // select nearest object
                // get position of mouse click on screen
                let click_pos = handle_mouse_clicks(&mouse_input, &windows);
                let mut closest_shape: Option<(f32, Entity, &Transform, SimulationIndex, &NodeBuilderRef, &NodeType)> = None;
                let mut prev_selection: Option<(Entity, &NodeType, &Transform)> = None;
                if let Some(click_pos) = click_pos{
                    // println!("{:?}", click_pos);
                    if let Some(camera_transform) = camera.iter().next(){
                        // camera scaling factor
                        let scaling = camera_transform.scale.x;
                        // get position of 0,0 of world coordinate system in screen coordinates
                        let midpoint_screenspace = (get_primary_window_size(&windows)/2.0)-(Vec2::new(camera_transform.translation.x, camera_transform.translation.y))/scaling;
                        let shape_dists = shapes.iter().map( | (entity, transform, shapevariant, index, nodebuilderref) | {
                            // check if this entity is the one currently selected
                            //  (we need to change the color later on when unselecting it)
                            if let Some(selected_nb) = &uistate.selected_node {
                                if selected_nb.0 == nodebuilderref.0 {
                                    prev_selection = Some((entity, shapevariant, transform));
                                }
                            }
                            // get shape position in scren coordinates
                            let position = midpoint_screenspace + (Vec2::new(transform.translation.x, transform.translation.y))/scaling;
                            // calculate distance, squared to improve performance so does not need to be rooted
                            let dist = (position - click_pos).length_squared();
                            (dist, entity, transform, index, nodebuilderref, shapevariant)
                        });
                        shape_dists.for_each(
                            | (d, entity, transform, i, nbr, sv) | {
                                if let Some((d_prev, entity_prev, trans_prev, _i_prev, nbr_prev, _sv_prev)) = closest_shape {
                                    if d < d_prev {
                                        closest_shape = Some((d, entity, transform, *i, nbr, sv));
                                    }
                                } else {
                                    closest_shape = Some((d, entity, transform, *i, nbr, sv));
                                }
                            }
                        );
                    }
                    if let Some((_d, entity, transform, _, nbr, sv)) = closest_shape {
                        if let Some((prev_selected_node_entity, node_type, prev_trans)) = prev_selection {
                            let pos = prev_trans.translation;
                            let new_shape_bundle = match node_type {
                                NodeType::CROSSING => node_render::crossing(pos.x, pos.y, theme.crossing),
                                NodeType::IONODE => node_render::io_node(pos.x, pos.y, theme.io_node),
                                NodeType::STREET => todo!("Street selection is not implemented yet!"),
                            };
                            commands.entity(prev_selected_node_entity).remove_bundle::<ShapeBundle>().insert_bundle(new_shape_bundle);
                        }
                        uistate.selected_node = Some(nbr.clone());
                        let pos = transform.translation;
                        let new_shape_bundle = match sv {
                            NodeType::CROSSING => node_render::crossing(pos.x, pos.y, theme.highlight),
                            NodeType::IONODE => node_render::io_node(pos.x, pos.y, theme.highlight),
                            NodeType::STREET => todo!("Street selection is not implemented yet!"),
                        };
                        commands.entity(entity).remove_bundle::<ShapeBundle>().insert_bundle(new_shape_bundle);
                    } 
                };
            }
            ToolType::None => (),
            ToolType::AddStreet => (),
        }
    }
}
// for selection
fn handle_mouse_clicks(mouse_input: &Res<Input<MouseButton>>, windows: &Res<Windows>) -> Option<Vec2>{
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        win.cursor_position()
    }
    else {
        None
    }
}
fn mouse_panning(windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    uistate : Res<UIState>,
    mut camera: Query<&mut Transform, With<Camera>>)
    {
    // change input mapping for orbit and panning here
    let pan_button = MouseButton::Left;

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

    for mut transform in camera.iter_mut(){
        if pan.length_squared() > 0.0 && uistate.toolbar.get_tooltype() == ToolType::Pan{
            let window = get_primary_window_size(&windows);
            pan = pan*Vec2::new(transform.scale.x, transform.scale.y)/(1.3*std::f32::consts::PI);
            transform.translation += Vec3::new(-pan.x, pan.y, 0.0);            
        }
        else if scroll.abs() > 0.0 {
            let scr = f32::powf(1.1, scroll);
            transform.scale *= Vec3::new(scr, scr, 1.0);
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
pub struct SimulationIndex(usize);
