use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use simulator::simple::simulation::Simulator;
use wasm_bindgen::prelude::*;
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
    .init_resource::<UIState>()
    //app.add_plugins(bevy_webgl2::DefaultPlugins);
    // when building for Web, use WebGL2 rendering
    //#[cfg(target_arch = "wasm32")]
    //app.add_plugin(bevy_webgl2::WebGL2Plugin);
    .add_startup_system(generate_simulation.system())
    .insert_resource(simulator::debug::build_grid_sim(10).build())
    .add_system(ui_example.system())
    .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
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