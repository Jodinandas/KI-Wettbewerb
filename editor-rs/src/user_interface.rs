use std::collections::HashMap;

use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CollapsingHeader, CtxRef, Ui},
    EguiContext,
};
use simulator::{datastructs::WeakIntMut, nodes::NodeBuilder, SimManager};

use crate::{
    tool_systems::SelectedNode, CurrentTheme, NeedsRecolor, NodeBuilderRef, NodeType, UIMode,
    UIState, UITheme,
};

/// Draws the ui
///
/// Nice reference: [Examples](https://github.com/mvlabat/bevy_egui/blob/main/examples/ui.rs)
pub fn ui_example(
    commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
    mut sim_manager: ResMut<SimManager>,
    mut background: ResMut<ClearColor>,
    mut theme: ResMut<UITheme>,
    mut current_theme: ResMut<CurrentTheme>,
    // mut colors: ResMut<Assets<ColorMaterial>>,
    nodes: QuerySet<(
        Query<Entity, With<NodeType>>,
        Query<(Entity, &NodeBuilderRef), (With<NodeType>, With<SelectedNode>)>,
    )>, //mut crossings: Query<, With<IONodeMarker>>
) {
    let mut repaint_necessary = false;
    let panel = egui::TopBottomPanel::top("menu_top_panel");
    panel.show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            egui::menu::menu(ui, "File", |ui| {
                ui.button("Nothing here yet...");
            });
            ui.separator();
            if ui.button("Preferences").clicked() {
                ui_state.new_mode(UIMode::Preferences);
            }
        });
    });
    match ui_state.mode {
        UIMode::Editor => {
            // Left Side panel, mainly for displaying the item editor
            egui::SidePanel::left("item_editor")
                .default_width(300.0)
                .resizable(false)
                .show(egui_context.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.heading("ItemEditor");
                        egui::warn_if_debug_build(ui);
                    });
                    ui.separator();
                    // if a node is selected, draw its editor
                    //  (each node type has different fields and possibilites
                    //   for modification by the user. Therefor, different ui
                    //   are needed)
                    if let Ok((_entity, selected_node_ref)) = nodes.q1().single() {
                        let selected_node = &selected_node_ref.0;
                        let display_conns = |ui: &mut Ui,
                                             conns: &mut HashMap<
                            simulator::nodes::Direction,
                            WeakIntMut<NodeBuilder>,
                        >| {
                            let mut conns =
                                conns.iter_mut().collect::<Vec<(
                                    &simulator::nodes::Direction,
                                    &mut WeakIntMut<NodeBuilder>,
                                )>>();
                            conns.sort_by_key(|(d, _)| match d {
                                simulator::nodes::Direction::N => 0,
                                simulator::nodes::Direction::E => 1,
                                simulator::nodes::Direction::S => 2,
                                simulator::nodes::Direction::W => 3,
                            });
                            for (dir, c) in conns.iter() {
                                let (ntype, id) = match &*c.upgrade().get() {
                                    NodeBuilder::IONode(n) => ("In/Out Node", n.id),
                                    NodeBuilder::Crossing(n) => ("Crossing", n.id),
                                    NodeBuilder::Street(n) => ("Street", n.id),
                                };
                                ui.label(format!("{:?}\t(id={}): {}", dir, id, ntype));
                                if ntype == "Street" {
                                    match &mut *c.upgrade().get() {
                                        NodeBuilder::Street(street) => {
                                            ui.add(
                                                egui::Slider::new(&mut street.lanes, 1..=10)
                                                    .text("lanes")
                                                    .clamp_to_range(true),
                                            );
                                        }
                                        _ => panic!(""),
                                    };
                                }
                            }
                        };
                        match &mut *selected_node.get() {
                            NodeBuilder::IONode(node) => {
                                ui.horizontal(|ui| {
                                    ui.label("Node type: ");
                                    ui.label("In/Out Node");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Node ID: ");
                                    ui.label(node.id.to_string());
                                });
                                ui.add(
                                    egui::Slider::new(&mut node.spawn_rate, 0.0..=100.0)
                                        .text("spawn rate")
                                        .clamp_to_range(true),
                                );
                                CollapsingHeader::new(format!(
                                    "Connections ({})",
                                    node.connections_out.len()
                                ))
                                .default_open(true)
                                .show(ui, |ui| {
                                    for c in node.connections_out.iter() {
                                        let (ntype, id) = match &*c.upgrade().get() {
                                            NodeBuilder::IONode(n) => ("In/Out Node", n.id),
                                            NodeBuilder::Crossing(n) => ("Crossing", n.id),
                                            NodeBuilder::Street(n) => ("Street", n.id),
                                        };
                                        ui.label(format!("(id={}): {}", id, ntype));
                                        if ntype == "Street" {
                                            match &mut *c.upgrade().get() {
                                                NodeBuilder::Street(street) => {
                                                    ui.add(
                                                        egui::Slider::new(
                                                            &mut street.lanes,
                                                            1..=10,
                                                        )
                                                        .text("lanes")
                                                        .clamp_to_range(true),
                                                    );
                                                }
                                                _ => panic!(""),
                                            };
                                        }
                                    }
                                });
                            }
                            NodeBuilder::Crossing(node) => {
                                ui.horizontal(|ui| {
                                    ui.label("Node type: ");
                                    ui.label("Crossing");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Node ID: ");
                                    ui.label(node.id.to_string());
                                });
                                CollapsingHeader::new(format!(
                                    "Connections IN ({})",
                                    node.connections.input.len()
                                ))
                                .default_open(true)
                                .show(ui, |ui| display_conns(ui, &mut node.connections.input));
                                CollapsingHeader::new(format!(
                                    "Connections OUT ({})",
                                    node.connections.output.len()
                                ))
                                .default_open(true)
                                .show(ui, |ui| display_conns(ui, &mut node.connections.output));
                            }
                            NodeBuilder::Street(node) => {
                                ui.horizontal(|ui| {
                                    ui.label("Node type: ");
                                    ui.label("Street");
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Node ID: ");
                                    ui.label(node.id.to_string());
                                });
                            }
                        }
                    }
                });
            // Toolbar
            egui::SidePanel::right("toolbar")
                .default_width(100.0)
                .resizable(false)
                .show(egui_context.ctx(), |ui| {
                    ui.vertical_centered(|ui| ui_state.toolbar.render_tools(ui));
                    ui.separator();
                    if ui.button("Start Simulation").clicked() {
                        ui_state.mode = UIMode::Simulator;
                        let num_sims = 1;
                        match sim_manager.simulate(num_sims) {
                            Ok(_) => info!("Starting with {} concurrent Simulations", num_sims),
                            Err(e) => error!("Unable to start Simulations. Error: {}", e),
                        }
                        match sim_manager.track_simulation(0) {
                            Ok(_) => info!("Set to track simulation with index 0"),
                            Err(_) => warn!("Can not track simulation with index 0"),
                        }
                    }
                });
        }
        UIMode::Simulator => {}
        UIMode::Preferences => {
            // egui::CentralPanel::default()
            egui::CentralPanel::default().show(egui_context.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Preferences");
                    ui.spacing();
                    if ui.button("(Close)").clicked() {
                        ui_state.to_prev_mode();
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    let mut new_theme = (*current_theme).clone();
                    ui.radio_value(&mut new_theme, CurrentTheme::LIGHT, "Light");
                    ui.radio_value(&mut new_theme, CurrentTheme::DRACULA, "Dracula");
                    if new_theme != *current_theme {
                        *current_theme = new_theme;
                        *theme = UITheme::from_enum(&new_theme);
                        repaint_necessary = true;
                    }
                });
            });
        }
    }
    if repaint_necessary {
        repaint_ui(
            commands,
            Some(egui_context.ctx()),
            &mut background,
            nodes.q0(),
            theme,
        );
    }
}

fn repaint_ui(
    mut commands: Commands,
    egui_ui: Option<&CtxRef>,
    background: &mut ResMut<ClearColor>,
    nodes: &Query<Entity, With<NodeType>>,
    theme: ResMut<UITheme>,
) {
    background.0 = theme.background;
    if let Some(ui) = egui_ui {
        ui.set_visuals(theme.egui_visuals.clone());
    }
    nodes.for_each(|entity| {
        commands.entity(entity).insert(NeedsRecolor);
    });
}
