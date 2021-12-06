use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::*,
    shapes::{Polygon, RegularPolygon},
};
use simulator::simple::node;
use simulator::simple::node_builder::{NodeBuilder, NodeBuilderTrait};
use simulator::{
    debug::build_grid_sim,
    simple::{simulation::Simulator, simulation_builder::SimulatorBuilder},
};
use std::borrow::BorrowMut;
use std::ptr;
use std::sync::{Arc, Weak};
use toolbar::ToolType;
use wasm_bindgen::prelude::*;
mod toolbar;
use simulator;

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

enum UIMode {
    Editor,
    Simulator,
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
            UIMode::Simulator => UIMode::Editor,
        }
    }
}

#[derive(Default)]
pub struct UIState {
    toolbar: toolbar::Toolbar,
    mode: UIMode,
}

#[derive(Default)]
struct UITheme {
    background: Color,
    io_node: Color,
    street: Color,
    crossing: Color,
}

#[derive(PartialEq)]
enum CurrentTheme {
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
        }
    }
    pub fn dark() -> UITheme {
        UITheme {
            background: Color::rgb(0.0, 0.0, 0.0),
            io_node: Color::rgb(200.0, 200.0, 0.0),
            street: Color::rgb(100., 50., 200.),
            crossing: Color::rgb(200.0, 200.0, 0.0),
        }
    }
    pub fn from_enum(theme: &CurrentTheme) -> UITheme {
        match theme {
            CurrentTheme::LIGHT => UITheme::light(),
            CurrentTheme::DARK => UITheme::dark(),
        }
    }

}
enum NodeType {
    CROSSING,
    IONODE,
    STREET
}

const GRID_NODE_SPACING: usize = 200;
const GRID_SIDE_LENGTH: usize = 70;
const STREET_THICKNESS: f32 = 5.0;
const STREET_SPACING: usize = 20;
const CROSSING_SIZE: f32 = 70.0;
const IONODE_SIZE: f32 = 40.0;

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
        .add_startup_system(setup.system())
        .add_startup_system(spawn_simulation_builder.system())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(UITheme::dark()) // Theme
        .insert_resource(CurrentTheme::DARK) // Theme
        .insert_resource(bevy::input::InputSystem)
        .add_system(ui_example.system())
        //.add_system(rotation_test.system())
        .add_system(panning.system())
        .run();
}

/// Draws the ui
///
/// Nice reference: [Examples](https://github.com/mvlabat/bevy_egui/blob/main/examples/ui.rs)
fn ui_example(
    egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
    mut background: ResMut<ClearColor>,
    mut theme: ResMut<UITheme>,
    mut current_theme: ResMut<CurrentTheme>,
    nodes: Query<(&mut ShapeColors, &NodeType)>
    //mut crossings: Query<, With<IONodeMarker>>
) {
    egui::TopBottomPanel::top("menu_top_panel").show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            let new_visuals = ui
                .style_mut()
                .visuals
                .clone()
                .light_dark_small_toggle_button(ui);
            if let Some(visuals) = new_visuals {
                let repaint_necessary = (visuals.dark_mode
                    && (*current_theme != CurrentTheme::DARK))
                    || (!visuals.dark_mode && (*current_theme != CurrentTheme::LIGHT));
                if repaint_necessary {
                    match visuals.dark_mode {
                        true => *current_theme = CurrentTheme::DARK,
                        false => *current_theme = CurrentTheme::LIGHT,
                    }
                    ui.ctx().set_visuals(visuals);
                    *theme = UITheme::from_enum(&*current_theme);
                    background.0 = theme.background;
                    nodes.for_each_mut(
                        | (mut shape_color, node_type) | {
                            let color = match node_type {
                                NodeType::CROSSING => theme.crossing,
                                NodeType::IONODE => theme.io_node,
                                NodeType::STREET => theme.street,
                            };
                            shape_color.main = color;
                        }
                    )
                }
            };
            ui.separator();
            egui::menu::menu(ui, "File", |ui| {
                ui.label("Nothing here yet...");
            });
            if ui.button("Toggle Mode").clicked() {
                ui_state.mode.toggle();
            };
        });
    });
    match ui_state.mode {
        UIMode::Editor => {
            // Left Side panel, mainly for displaying the item editor
            egui::SidePanel::left("item_editor")
                .default_width(300.0)
                .show(egui_context.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("ItemEditor");
                        egui::warn_if_debug_build(ui);
                    });
                    ui.separator();
                });
            // Toolbar
            egui::SidePanel::right("toolbar")
                .default_width(50.0)
                .show(egui_context.ctx(), |ui| {
                    ui.vertical_centered(|ui| ui_state.toolbar.render_tools(ui));
                    ui.separator();
                });
        }
        UIMode::Simulator => {}
    }
}

fn generate_simulation() {}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // camera
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(Camera);
    let step = 100.0;
    let (x, y) = (7, 7);
    let (offsetx, offsety) = (500.0, 500.0);

    return;

    for i in 0..x {
        for j in 0..y {
            let test_shape = shapes::RegularPolygon {
                sides: 7,
                feature: shapes::RegularPolygonFeature::Radius(50.0),
                ..shapes::RegularPolygon::default()
            };
            let mut geometry = GeometryBuilder::build_as(
                &test_shape,
                ShapeColors::outlined(Color::rgb(100., 50., 0.), Color::WHITE),
                DrawMode::Fill(FillOptions::default()), //DrawMode::Outlined {
                //    fill_options: FillOptions::default(),
                //    outline_options: StrokeOptions::default().with_line_width(10.0)
                //}
                Transform::from_xyz(i as f32 * step - offsetx, j as f32 * step - offsety, 0.0),
            );
            geometry
                .transform
                .rotate(Quat::from_rotation_z(5.0 * i as f32 * j as f32));
            commands.spawn_bundle(geometry).insert(ExamplePolygon);
        }
    }
}

/// This function spawns the simultation builder instance
/// that is later used to create simulations
///
/// It also generates the graphics for it
fn spawn_simulation_builder(mut commands: Commands, asset_server: Res<AssetServer>) {
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
            // println!("Generated Graphics for {}", i);
            //const PATH: [usize; 11] = [2, 32, 7, 46, 12, 60, 17, 63, 18, 66, 19];
            //const PATH: [usize; 7] = [1, 28, 6, 31, 7, 30, 2];
            //if PATH.contains(&i) {
            //    return;
            //}
            match &*(**n_builder).lock().unwrap() {
                NodeBuilder::Crossing(crossing) => {
                    // println!("   type=Crossing");
                    //let test_shape = shapes::RegularPolygon {
                    //    sides: 4,
                    //    feature: shapes::RegularPolygonFeature::Radius(50.0),
                    //    ..shapes::RegularPolygon::default()
                    //};
                    let test_shape = shapes::Rectangle {
                        width: CROSSING_SIZE,
                        height: CROSSING_SIZE,
                        ..shapes::Rectangle::default()
                    };
                    let geometry = GeometryBuilder::build_as(
                        &test_shape,
                        ShapeColors::outlined(Color::rgb(100., 50., 0.), Color::WHITE),
                        DrawMode::Fill(FillOptions::default()), //DrawMode::Outlined {
                        //    fill_options: FillOptions::default(),
                        //    outline_options: StrokeOptions::default().with_line_width(10.0)
                        //}
                        Transform::from_xyz(calc_x(i), calc_y(i), 0.),
                    );
                    commands.spawn_bundle(geometry).insert(SimulationIndex(i))
                        .insert(NodeType::CROSSING);
                }

                NodeBuilder::IONode(io_node) => {
                    // println!("   type=IONode");
                    //let test_shape = shapes::RegularPolygon {
                    //    sides: 7,
                    //    feature: shapes::RegularPolygonFeature::Radius(50.0),
                    //    ..shapes::RegularPolygon::default()
                    //};
                    let test_shape = shapes::Circle {
                        radius: IONODE_SIZE,
                        ..shapes::Circle::default()
                    };
                    let geometry = GeometryBuilder::build_as(
                        &test_shape,
                        ShapeColors::outlined(Color::rgb(0., 100., 0.), Color::WHITE),
                        DrawMode::Fill(FillOptions::default()), //DrawMode::Outlined {
                        //    fill_options: FillOptions::default(),
                        //    outline_options: StrokeOptions::default().with_line_width(10.0)
                        //}
                        Transform::from_xyz(calc_x(i), calc_y(i), 0.),
                    );
                    commands.spawn_bundle(geometry).insert(SimulationIndex(i))
                        .insert(NodeType::IONODE);
                }
                NodeBuilder::Street(street) => {
                    // println!("   type=Street");
                    if let Some(conn_in) = &street.conn_in {
                        if let Some(conn_out) = &street.conn_out {
                            let index_in = (*conn_in.upgrade().unwrap()).lock().unwrap().get_id();
                            let index_out = (*conn_out.upgrade().unwrap()).lock().unwrap().get_id();
                            let pos_j = Vec2::new(calc_x(index_in), calc_y(index_in));
                            let pos_i = Vec2::new(calc_x(index_out), calc_y(index_out));
                            //println!("I({}): {:?}, J({}): {:?}", i, pos_i, j, pos_j);
                            let test_shape = shapes::Line(pos_i, pos_j);
                            let geometry = GeometryBuilder::build_as(
                                &test_shape,
                                ShapeColors::outlined(Color::rgb(0., 100., 0.), Color::WHITE),
                                //DrawMode::Fill(FillOptions::default()),
                                DrawMode::Outlined {
                                    fill_options: FillOptions::default(),
                                    outline_options: StrokeOptions::default()
                                        .with_line_width(STREET_THICKNESS),
                                },
                                Transform::default(), // Transform::from_xyz(calc_x(i), calc_y(i), 0.0)
                            );
                            commands.spawn_bundle(geometry).insert(SimulationIndex(i))
                                .insert(NodeType::STREET);
                        }
                    }
                    return;
                }
            }
        });
    commands.insert_resource(new_builder);
}

struct Camera;
// pans canvas
fn panning(keyboard_input: Res<Input<KeyCode>>, mut camera: Query<&mut Transform, With<Camera>>) {
    let speed: f32 = PAN_SPEED;
    for mut transform in camera.iter_mut() {
        let s: Vec3 = transform.scale;
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            transform.translation.x -= speed * s.x;
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            transform.translation.x += speed * s.x;
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            transform.translation.y -= speed * s.y;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            transform.translation.y += speed * s.y;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            transform.scale += Vec3::from((0.1 * s.x, 0.1 * s.y, 0.1 * s.z));
        }
        if keyboard_input.pressed(KeyCode::E) {
            transform.scale -= Vec3::from((0.1 * s.x, 0.1 * s.y, 0.1 * s.z));
        }
    }
}
struct ExamplePolygon;

/*
fn toolbarsystem(mouse_input: Res<Input<mouse::MouseButton>>,
      mouse_movement: Res<Input<>>,
      mut camera: Query<&mut Transform, With<Camera>>,
      mut uistate: ResMut<UIState>){

    if let Some(tool) = uistate.toolbar.get_selected() {
        match tool.get_type() {
            ToolType::Pan => {
                if mouse_input.pressed(MouseButton::Left){
                    let dpos = mouse_movement;
                }
            },
            _ => {}
        }
    }
}
*/

fn rotation_test(polygon_query: Query<&mut Transform, With<ExamplePolygon>>, time: Res<Time>) {
    polygon_query.for_each_mut(|mut t| t.rotate(Quat::from_rotation_z(time.delta_seconds() * 0.5)));
}

pub struct NodeComponent;
pub struct SimulationIndex(usize);

pub trait Render {
    fn render(
        &mut self,
        node_query: Query<(&SimulationIndex, &Transform), With<NodeComponent>>,
        sim: Res<SimulatorBuilder>,
    );
}

impl Render for SimulatorBuilder {
    fn render(
        &mut self,
        node_query: Query<(&SimulationIndex, &Transform), With<NodeComponent>>,
        sim: Res<SimulatorBuilder>,
    ) {
        for (node_i, transform) in node_query.iter() {
            let node = self.get_node(node_i.0);
            match (**node).lock().unwrap().generate_graphics_info() {
                node::graphics::Info::Crossing(_) => {}
                node::graphics::Info::IONode(_) => {}
                node::graphics::Info::Street(street) => {
                    // Implement something here
                }
            }
        }
    }
}
