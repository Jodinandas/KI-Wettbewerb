use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use simulator::simple::simulation::Simulator;
use wasm_bindgen::prelude::*;
use bevy_prototype_lyon::{prelude::*, shapes::{Polygon, RegularPolygon}};
use simulator;

#[derive(PartialEq)]
pub enum Theme {
    Light,
    Dark
}
impl Default for Theme {
    fn default() -> Self {
        Theme::Light
    }
}

#[derive(Default)]
pub struct UIState {
    theme: Theme
}

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
    .insert_resource(simulator::debug::build_grid_sim(10).build())
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
    .add_system(ui_example.system())
    .add_system(rotation_test.system())
    .run();
}

/// Draws the ui
/// 
/// Nice reference: [Examples](https://github.com/mvlabat/bevy_egui/blob/main/examples/ui.rs)
fn ui_example(egui_context: ResMut<EguiContext>, mut ui_state: ResMut<UIState>,) {
    egui::TopBottomPanel::top("menu_top_panel")
        .show(egui_context.ctx(), |ui| {
                ui.horizontal(|ui| {
                    let new_visuals = ui.style_mut().visuals.clone().light_dark_small_toggle_button(ui);
                    if let Some(visuals) = new_visuals {
                        ui.ctx().set_visuals(visuals);
                    };
                    ui.separator();
                    egui::menu::menu(ui, "File", | ui | {
                        ui.label("Nothing here yet...");
                    });
            });
        });
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
}

fn generate_simulation() {}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    let step = 100.0;
    let (x, y) = (7, 7);
    let (offsetx, offsety) = (500.0, 500.0);
    
    for i in 0..x{
        for j in 0..y {
            let test_shape = shapes::RegularPolygon {
                    sides: 7,
                    feature: shapes::RegularPolygonFeature::Radius(50.0),
                    ..shapes::RegularPolygon::default()
                };
                let mut geometry = GeometryBuilder::build_as(
                    &test_shape, 
                    ShapeColors::outlined(Color::rgb(100., 50., 0.), Color::WHITE), 
                    DrawMode::Fill(FillOptions::default())
                    //DrawMode::Outlined {
                    //    fill_options: FillOptions::default(),
                    //    outline_options: StrokeOptions::default().with_line_width(10.0)
                    //}
                    , Transform::from_xyz(i as f32 * step - offsetx, j as f32 * step - offsety, 0.0)
                );
                geometry.transform.rotate(Quat::from_rotation_z(5.0 * i as f32 * j as f32));
                commands.spawn_bundle(
                    geometry
                ).insert(ExamplePolygon);
            }
    }
}

struct ExamplePolygon;

fn rotation_test(polygon_query: Query<&mut Transform, With<ExamplePolygon>>, time: Res<Time>) {
    polygon_query.for_each_mut( | mut t | {
        t.rotate(Quat::from_rotation_z(time.delta_seconds() * 0.5))
    }
    );
}

pub struct NodeComponent;
pub struct SimulationIndex(usize);

pub trait Render {
    fn render(&mut self, node_query: Query<(&SimulationIndex, &Transform), With<NodeComponent>>, sim: Res<Simulator>);
}

impl Render for Simulator {
    fn render(&mut self, node_query: Query<(&SimulationIndex, &Transform), With<NodeComponent>>, sim: Res<Simulator>) {
        for (node_i, transform) in node_query.iter() {
            
        }
    }
}