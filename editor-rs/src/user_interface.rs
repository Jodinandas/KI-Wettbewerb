use std::{collections::HashMap, ops::RangeInclusive, env, fs::File};
use std::io::{Write, Read};
use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CollapsingHeader, CtxRef, Ui, Color32},
    EguiContext,
};
use simulator::{datastructs::WeakIntMut, nodes::NodeBuilder, SimManager, SimulatorBuilder};

use crate::{StreetLinePosition, SimulationID, node_bundles};
use crate::{
    tool_systems::SelectedNode, CurrentTheme, NeedsRecolor, NodeBuilderRef, NodeType, UIMode,
    UIState, themes::UITheme,
};

use art_int::Network;
use serde::{self, Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct FunnyNNBuilderCombi {
    pub builder: SimulatorBuilder,
    pub nn: Option<Vec<Network>>,
    pub builder_graphics: HashMap<usize, Vec<[f32; 2]>>
}



pub fn update_sim_reports(
    mut sim_manager: ResMut<SimManager>
) {
    sim_manager.update_reports();
}

/// Draws the ui
///
/// Nice reference: [Examples](https://github.com/mvlabat/bevy_egui/blob/main/examples/ui.rs)
#[tracing::instrument(skip_all)]
pub fn draw_user_interface(
    mut commands: Commands,
    egui_context: ResMut<EguiContext>,
    mut ui_state: ResMut<UIState>,
    mut sim_manager: ResMut<SimManager>,
    mut background: ResMut<ClearColor>,
    mut theme: ResMut<UITheme>,
    mut current_theme: ResMut<CurrentTheme>,
    // mut colors: ResMut<Assets<ColorMaterial>>,
    nodes: QuerySet<(
        Query<(Entity, &Transform, Option<&StreetLinePosition>, &SimulationID), With<NodeType>>,
        Query<(Entity, &NodeBuilderRef), (With<NodeType>, With<SelectedNode>)>,
    )>, //mut crossings: Query<, With<IONodeMarker>>
) {
    let mut repaint_necessary = false;
    let panel = egui::TopBottomPanel::top("menu_top_panel");
    panel.show(egui_context.ctx(), |ui| {
        ui.horizontal(|ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {

                    if !sim_manager.is_simulating() {
                        if ui.button("Save").clicked() {
                            let report = sim_manager.simulation_report.as_ref().map(| report | report.get_best_nn());
                            match sim_manager.modify_sim_builder() {
                                Ok(builder) => {
                                    let sim_wrapper = FunnyNNBuilderCombi {
                                        builder: builder.clone(),
                                        nn: report,
                                        builder_graphics: nodes.q0().iter().map(| (_, transform, street_line_pos, sim_id) | {
                                            let id = sim_id.0;
                                            match street_line_pos {
                                                Some(pos) => {
                                                    let start: [f32; 2] = pos.0.into();
                                                    let end: [f32; 2] = pos.1.into();
                                                    (id, vec![start, end])
                                                },
                                                None => {
                                                    let pos = [transform.translation.x, transform.translation.y];
                                                    (id, vec![pos])
                                                },
                                            }
                                        }).collect()
                                    };
                                    let json = serde_json::to_string_pretty(&sim_wrapper);
                                    match json {
                                        Ok(s) => {
                                            // Create a temporary file.
                                            let temp_directory = env::current_dir().unwrap();
                                            let full_path = temp_directory.as_path();
                                            let temp_file = temp_directory.join("StreetSimulation.json");
                                            let mut file = File::create(temp_file).unwrap();
                                            write!(&mut file, "{}", s).unwrap();
                                            info!("Saved simulation and street network to {}", full_path.display());
                                        },
                                        Err(_) => todo!(),
                                    }
                                },
                                Err(_) => todo!(),
                            }
                        }
                        if ui.button("Load").clicked() {
                            let directory = env::current_dir().unwrap();
                            let path = directory.join("StreetSimulation.json");
                            let mut file = File::open(path).unwrap();
                            let mut json = String::new();
                            file.read_to_string(&mut json).unwrap();
                            let sim_wrapper = serde_json::from_str::<FunnyNNBuilderCombi>(&json);
                            match sim_manager.modify_sim_builder() {
                                Ok(builder) => {
                                    match sim_wrapper {
                                        Ok(sim_info) => {
                                            let new_builder = sim_info.builder;
                                            *builder = new_builder; 
                                            // despawn old nodes
                                            nodes.q0().iter().for_each(| (entity, _, _, _) | {
                                                commands.entity(entity).despawn_recursive();
                                            });
                                            let ui_info = sim_info.builder_graphics;
                                            ui_info.iter().for_each(| (id, position) | {
                                                let node = builder.get_node(*id).unwrap();
                                                match &*node.get() {
                                                    NodeBuilder::IONode(_) => {
                                                        let bundle = node_bundles::IONodeBundle::new(*id, &node, position[0].into(), theme.io_node);
                                                        commands.spawn_bundle(bundle);
                                                    },
                                                    NodeBuilder::Crossing(_) => {
                                                        let bundle = node_bundles::CrossingBundle::new(*id, &node, position[0].into(), theme.crossing);
                                                        commands.spawn_bundle(bundle);
                                                    },
                                                    NodeBuilder::Street(_) => {
                                                        let bundle = node_bundles::StreetBundle::new(*id, &node, position[0].into(), position[1].into(), theme.street);
                                                        commands.spawn_bundle(bundle);
                                                    },
                                                }

                                            });
                                            let nn = sim_info.nn;
                                            info!("Loaded Simulation Builder");
                                        },
                                        Err(err) => {
                                            error!("Unable to load from file. Error: {}", err);
                                        },
                                    }
                                },
                                Err(err) => {
                                    error!("Cannot load file because SimBuilder can not be modified: {}", err)
                                },
                            }
                        }
                    }
                });
            ui.separator();
            ui.horizontal( | ui | {
                if ui.button("Street Editor").clicked() {
                    ui_state.new_mode(UIMode::Editor);
                } else if ui.button("Simulation").clicked() {
                    ui_state.new_mode(UIMode::Simulator);
                } else if ui.button("Preferences").clicked() {
                    ui_state.new_mode(UIMode::Preferences);
                }
            });
            });
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
                        ui.colored_label(theme.text_color,"ItemEditor");
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
                                ui.colored_label(theme.text_color,format!("{:?}\t(id={}): {}", dir, id, ntype));
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
                                    ui.colored_label(theme.text_color, "Node type: ");
                                    ui.colored_label(theme.text_color,"In/Out Node");
                                });
                                ui.horizontal(|ui| {
                                    ui.colored_label(theme.text_color,"Node ID: ");
                                    ui.colored_label(theme.text_color,node.id.to_string());
                                });
                                ui.add(
                                    egui::Slider::new(&mut node.spawn_rate, 0.0..=1.0)
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
                                        ui.colored_label(theme.text_color, format!("(id={}): {}", id, ntype));
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
                                    ui.colored_label(theme.text_color,"Node type: ");
                                    ui.colored_label(theme.text_color,"Crossing");
                                });
                                ui.horizontal(|ui| {
                                    ui.colored_label(theme.text_color,"Node ID: ");
                                    ui.colored_label(theme.text_color,node.id.to_string());
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
                                    ui.colored_label(theme.text_color,"Node type: ");
                                    ui.colored_label(theme.text_color,"Street");
                                });
                                ui.horizontal(|ui| {
                                    ui.colored_label(theme.text_color,"Node ID: ");
                                    ui.colored_label(theme.text_color,node.id.to_string());
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
                    // ui.separator();
                    // if ui.button("Start Simulation").clicked() {
                    //     ui_state.mode = UIMode::Simulator;
                    //     let num_sims = 15;
                    //     match sim_manager.simulate(num_sims, 10) {
                    //         Ok(_) => info!("Starting with {} concurrent Simulations", num_sims),
                    //         Err(e) => error!("Unable to start Simulations. Error: {}", e),
                    //     }
                    //     match sim_manager.track_simulation(0) {
                    //         Ok(_) => info!("Set to track simulation with index 0"),
                    //         Err(_) => warn!("Can not track simulation with index 0"),
                    //     }
                    // }
                });
        }
        UIMode::Simulator => {
            // Left Side panel, mainly for displaying the item editor
            egui::SidePanel::left("Simulation Settings")
                .default_width(300.0)
                .resizable(false)
                .show(egui_context.ctx(), |ui| {
                ui.heading("Simulation Settings");
                match sim_manager.is_simulating() {
                    false => {
                        ui.vertical(| ui | {
                            let builder = sim_manager.modify_sim_builder().expect("Can not modify SimBuilder even though no simulation is running");
                            ui.add(
                                egui::Slider::new(
                                    &mut builder.delay,
                                    0..=1000
                                )
                                .text("Simulation delay in ms")
                                .clamp_to_range(true)
                            );
                            ui.label("(Useful for inspecting the car movement)");
                            ui.separator();
                            ui.add(
                                egui::Slider::new(
                                    &mut builder.dt,
                                    0.01..=10.0
                                )
                                .text("Time step in seconds")
                                .clamp_to_range(true)
                            );
                            ui.separator();
                            ui.add(
                                egui::Slider::new(
                                    &mut sim_manager.generations,
                                    1..=10000
                                )
                                .text("Number of generations to simulate")
                                .clamp_to_range(true)
                            );
                            ui.separator();
                            ui.add(
                                egui::Slider::new(
                                    &mut sim_manager.population,
                                    1..=10000
                                )
                                .text("Population size")
                                .clamp_to_range(true)
                            );
                            ui.separator();
                            ui.add(
                                egui::Slider::new(
                                    &mut sim_manager.stop_iterations,
                                    1..=10000
                                )
                                .text("Iterations to stop")
                                .clamp_to_range(true)
                            );
                            ui.separator();
                            ui.add(
                                egui::Slider::new(
                                    &mut sim_manager.mutation_chance,
                                    0.0..=1.0
                                )
                                .text("Mutation Chance")
                                .clamp_to_range(true)
                            );
                            ui.add(
                                egui::Slider::new(
                                    &mut sim_manager.mutation_coeff,
                                    0.0..=1.0
                                )
                                .text("Mutation Coefficent")
                                .clamp_to_range(true)
                            );
                            ui.separator();
                            ui.heading("Commands");
                            ui.horizontal_wrapped(|  ui | {
                                if ui.button("Start Simulation").clicked() {
                                    match sim_manager.simulate() {
                                        Err(err) => error!("Error when trying to start simulation: {}", err),
                                        Ok(_) => {
                                            match sim_manager.track_simulation(2) {
                                                Ok(_) => info!("Tracking Simulation index=0"),
                                                Err(_) => warn!("Unable to track Simulation with index=0"),
                                            };
                                            info!("Started simulation")
                                        }
                                    }
                                }
                            });
                        });
                    },
                    true => {
                        ui.add(egui::Label::new("Locked").text_color(Color32::from_rgb((theme.highlight.r() * 255.0) as u8, (theme.highlight.g() * 255.0) as u8, (theme.highlight.b() * 255.0) as u8)).strong());
                        ui.separator();
                        ui.heading("Commands");
                        ui.vertical_centered(| ui | {
                            if ui.button("Stop Simulation").clicked() {
                                sim_manager.terminate_sims();
                            }
                            // if ui.button("Pause Simulation").clicked() {
                            // }
                        });
                    },
                }

            });
            egui::TopBottomPanel::bottom("Generation Report").default_height(100.0).resizable(true).show(egui_context.ctx(), | ui | {
                ui.heading("Generation Report");
                if let Some(sims) = &sim_manager.simulations {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (i, report) in sims.generation_reports.iter().enumerate() {
                            ui.horizontal(| ui | {
                                ui.label(format!("Generation #{}", i) );
                                ui.separator();
                                ui.label(format!("Cost: {}", report.cost) );
                                ui.label(format!("CO2: {} tonnes", report.tonnes_co2) );
                            });
                        }
                    });
                }
            });
            // Toolbar
            egui::SidePanel::right("Simulation Overview")
                .default_width(100.0)
                .resizable(false)
                .show(egui_context.ctx(), |ui| {
                    ui.horizontal(| ui | {
                        ui.heading("Simulation Overview");
                    });
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        if let Ok(stati) = sim_manager.get_sim_status() {
                            stati.iter_mut().enumerate().for_each( | (i, sim_info) | {
                                if ui.button(format!("Simulation {}", i)).clicked()  {
                                    sim_info.displaying = !sim_info.displaying;
                                }
                                if sim_info.displaying {
                                    egui::Window::new(format!("Information for Simulation {}", i)).show( egui_context.ctx(), | ui | {
                                                
                                    });
                                }
                            });
                        }
                    });
                });

        }
        UIMode::Preferences => {
            // egui::CentralPanel::default()
            egui::CentralPanel::default().show(egui_context.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(theme.text_color,"Preferences");
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

pub fn repaint_ui(
    mut commands: Commands,
    egui_ui: Option<&CtxRef>,
    background: &mut ResMut<ClearColor>,
    nodes: &Query<(Entity, &Transform, Option<&StreetLinePosition>, &SimulationID), With<NodeType>>,
    theme: ResMut<UITheme>,
) {
    background.0 = theme.background;
    if let Some(ui) = egui_ui {
        ui.set_visuals(theme.egui_visuals.clone());
    }
    nodes.for_each(| (entity, _, _, _)| {
        commands.entity(entity).insert(NeedsRecolor);
    });
}
