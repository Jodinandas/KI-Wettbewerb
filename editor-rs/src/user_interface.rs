use bevy::prelude::*;
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::nodes::NodeBuilder;
use bevy_egui::{egui::{self, Ui, CtxRef}, EguiContext};

use crate::{UIState, UITheme, CurrentTheme, StreetLinePosition, NodeType, UIMode, node_render};

/// Draws the ui
///
/// Nice reference: [Examples](https://github.com/mvlabat/bevy_egui/blob/main/examples/ui.rs)
pub fn ui_example(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
    mut background: ResMut<ClearColor>,
    mut theme: ResMut<UITheme>,
    mut current_theme: ResMut<CurrentTheme>,
    // mut colors: ResMut<Assets<ColorMaterial>>, 
    nodes: Query<(Entity, &Transform, Option<&StreetLinePosition>, &NodeType)>, //mut crossings: Query<, With<IONodeMarker>>
) {
    let mut repaint_necessary = false;
    egui::TopBottomPanel::top("menu_top_panel").show(egui_context.ctx(), |ui| {
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
                    if let Some(selected_node_wrapper) = &mut ui_state.selected_node {
                        let selected_node = &mut selected_node_wrapper.0;
                        match &mut *selected_node.get() {
                            NodeBuilder::IONode(node) => {
                                ui.horizontal( | ui | {
                                    ui.label("Node type: ");
                                    ui.label("In/Out Node");
                                });
                                ui.horizontal( | ui | {
                                    ui.label("Node ID: ");
                                    ui.label(node.id.to_string());
                                });
                                ui.add(egui::Slider::new(&mut node.spawn_rate, 0.0..=100.0).text("spawn rate").clamp_to_range(true));
                                ui.collapsing(format!("Connections ({})", node.connections.len()), | ui | {
                                    for c in node.connections.iter() {
                                        let (ntype, id) =  match &*c.upgrade().get() {
                                            NodeBuilder::IONode(n) => ("In/Out Node", n.id),
                                            NodeBuilder::Crossing(n) => ("Crossing", n.id),
                                            NodeBuilder::Street(n) => ("Street", n.id),
                                        };
                                        ui.label(format!("(id={}): {}", id, ntype));
                                    }
                                });
                            },
                            NodeBuilder::Crossing(node) => {
                                ui.horizontal( | ui | {
                                    ui.label("Node type: ");
                                    ui.label("Crossing");
                                });
                                ui.horizontal( | ui | {
                                    ui.label("Node ID: ");
                                    ui.label(node.id.to_string());
                                });
                                ui.collapsing(format!("Connections IN ({})", node.connections.input.len()), | ui | {
                                    for (dir, c) in node.connections.input.iter() {
                                        let (ntype, id) =  match &*c.upgrade().get() {
                                            NodeBuilder::IONode(n) => ("In/Out Node", n.id),
                                            NodeBuilder::Crossing(n) => ("Crossing", n.id),
                                            NodeBuilder::Street(n) => ("Street", n.id),
                                        };
                                        ui.label(format!("{:?}\t(id={}): {}", dir, id, ntype));
                                    }
                                });
                                ui.collapsing(format!("Connections OUT ({})", node.connections.output.len()), | ui | {
                                    for (dir, c) in node.connections.output.iter() {
                                        let (ntype, id) =  match &*c.upgrade().get() {
                                            NodeBuilder::IONode(n) => ("In/Out Node", n.id),
                                            NodeBuilder::Crossing(n) => ("Crossing", n.id),
                                            NodeBuilder::Street(n) => ("Street", n.id),
                                        };
                                        ui.label(format!("{:?}\t(id={}): {}", dir, id, ntype));
                                    }
                                });
                            },
                            NodeBuilder::Street(node) => {
                                ui.horizontal( | ui | {
                                    ui.label("Node type: ");
                                    ui.label("Street");
                                });
                                ui.horizontal( | ui | {
                                    ui.label("Node ID: ");
                                    ui.label(node.id.to_string());
                                });
                            },
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
                });
        }
        UIMode::Simulator => {},
        UIMode::Preferences => {
            // egui::CentralPanel::default()
            egui::CentralPanel::default().show(egui_context.ctx(), |ui| {
                ui.horizontal(| ui | {
                    ui.heading("Preferences");
                    ui.spacing();
                    if ui.button("(Close)").clicked() {
                        ui_state.to_prev_mode();
                    }
                });
                ui.separator();
                ui.vertical( | ui  | {
                    let mut new_theme = (*current_theme).clone();
                    ui.radio_value(&mut new_theme, CurrentTheme::LIGHT, "Light");
                    ui.radio_value(&mut new_theme, CurrentTheme::DARK, "Dark");
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
        repaint_ui(Some(egui_context.ctx()), &mut commands, &mut background, nodes, current_theme, theme);
    }
}

fn repaint_ui(egui_ui: Option<&CtxRef>, commands: &mut Commands, background: &mut ResMut<ClearColor>, nodes: Query<(Entity, &Transform, Option<&StreetLinePosition>, &NodeType)>, current_theme: ResMut<CurrentTheme>, theme: ResMut<UITheme>) {
        background.0 = theme.background;
        if let Some(ui) = egui_ui{
                ui.set_visuals(theme.egui_visuals.clone());
        }
        nodes.for_each_mut(|(entity, mut transform, street_line_position, node_type)| {
            match node_type {
                NodeType::CROSSING => {
                    let pos = transform.translation;
                    let new_shape_bundle = node_render::crossing(pos.x, pos.y, theme.crossing);
                    commands.entity(entity).remove_bundle::<ShapeBundle>().insert_bundle(new_shape_bundle);
                }
                NodeType::IONODE => {
                    let pos = transform.translation;
                    let new_shape_bundle = node_render::io_node(pos.x, pos.y, theme.io_node);
                    commands.entity(entity).remove_bundle::<ShapeBundle>().insert_bundle(new_shape_bundle);
                },
                NodeType::STREET => {
                    if let Some(line) = street_line_position {
                        let (p1, p2) = (line.0, line.1);
                        let new_shape_bundle = node_render::street(p1, p2, theme.street);
                        commands.entity(entity).remove_bundle::<ShapeBundle>().insert_bundle(new_shape_bundle);
                    }
                },
            };
        });
}