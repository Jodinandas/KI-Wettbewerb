use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(EguiPlugin);
    //app.add_plugins(bevy_webgl2::DefaultPlugins);
    // when building for Web, use WebGL2 rendering
    //#[cfg(target_arch = "wasm32")]
    //app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.add_system(ui_example.system());
    app.insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)));
    app.run();
}
fn ui_example(egui_context: ResMut<EguiContext>) {
    egui::SidePanel::left("item_editor")
        .default_width(300.0)
        .show(egui_context.ctx(), |ui| {
            ui.heading("ItemEditor");
            ui.separator();
        });
}