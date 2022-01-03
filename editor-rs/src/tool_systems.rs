use bevy::{
    ecs::schedule::ShouldRun,
    input::Input,
    math::Vec2,
    prelude::{
        BuildChildren, Children, Commands, DespawnRecursiveExt, Entity, GlobalTransform,
        MouseButton, Parent, Query, QuerySet, Res, ResMut, Transform, With, Without,
    },
    window::Windows,
};
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::{nodes::{
    CrossingBuilder, Direction, IONodeBuilder, InOut, NodeBuilder, NodeBuilderTrait,
}, SimManager};
#[allow(unused_imports)]
use log::{trace, debug, info, warn, error};

use crate::{
    get_primary_window_size,
    input::{self, handle_mouse_clicks},
    node_bundles::{
        ConnectorCircleIn, ConnectorCircleOut, CrossingBundle, IONodeBundle, InputCircle,
        OutputCircle, StreetBundle,
    },
    AddStreetStage, StreetLinePosition, CONNECTOR_DISPLAY_RADIUS,
};
use crate::{
    node_bundles::node_render, themes::UITheme, toolbar::ToolType, Camera,
    NeedsRecolor, NodeBuilderRef, NodeType, SimulationID, UIState, UnderCursor,
};

pub fn run_if_delete_node(ttype: Res<UIState>) -> ShouldRun {
    let ttype = match ttype.toolbar.get_selected() {
        Some(t) => t.get_type(),
        None => return ShouldRun::No,
    };
    match ttype {
        ToolType::DeleteNode => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
pub fn run_if_select(ttype: Res<UIState>) -> ShouldRun {
    let ttype = match ttype.toolbar.get_selected() {
        Some(t) => t.get_type(),
        None => return ShouldRun::No,
    };
    match ttype {
        ToolType::Select => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}
pub fn run_if_add_street(ttype: Res<UIState>) -> ShouldRun {
    let ttype = match ttype.toolbar.get_selected() {
        Some(t) => t.get_type(),
        None => return ShouldRun::No,
    };
    match ttype {
        ToolType::AddStreet => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

pub fn run_if_add_crossing(ttype: Res<UIState>) -> ShouldRun {
    let ttype = match ttype.toolbar.get_selected() {
        Some(t) => t.get_type(),
        None => return ShouldRun::No,
    };
    match ttype {
        ToolType::AddCrossing => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

pub fn run_if_add_ionode(ttype: Res<UIState>) -> ShouldRun {
    let ttype = match ttype.toolbar.get_selected() {
        Some(t) => t.get_type(),
        None => return ShouldRun::No,
    };
    match ttype {
        ToolType::AddIONode => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

pub fn mouse_to_world_space(cam: &Transform, mouse_pos: Vec2, windows: &Res<Windows>) -> Vec2 {
    let midpoint_screenspace = (get_primary_window_size(windows) / 2.0)
        - Vec2::new(cam.translation.x, cam.translation.y) / cam.scale.x;
    (mouse_pos - midpoint_screenspace) * cam.scale.x
}

/// A marker for crossings currently displaying connectors
pub struct HasConnectors;

/// generates connectors over crossings, so they can be connected using the
/// Add Street tool
pub fn generate_connectors(
    mut commands: Commands,
    theme: Res<UITheme>,
    stage: Res<AddStreetStage>,
    street: Query<&NewStreetInfo, With<PlacingStreet>>,
    node_under_cursor: Query<
        (Entity, &NodeBuilderRef, &NodeType),
        (With<UnderCursor>, Without<HasConnectors>),
    >,
) {
    if let Ok((entity, nbr, ntype)) = node_under_cursor.single() {
        if *ntype != NodeType::CROSSING {
            return;
        }
        let mut connectors: Vec<Entity> = Vec::new();
        let nbr = &nbr.0.get();
        match *stage {
            AddStreetStage::SelectingOutput => {
                match &**nbr {
                    NodeBuilder::Street(_) => return,
                    NodeBuilder::IONode(_node) => {
                        // draw a connector in the middle of the ionode
                        let id = commands
                            .spawn_bundle(ConnectorCircleOut::new(OutputCircle::Middle, theme.connector_out))
                            .id();
                        connectors.push(id);
                    },
                    NodeBuilder::Crossing(crossing_builder) => {
                        let dirs = [
                            OutputCircle::N,
                            OutputCircle::S,
                            OutputCircle::W,
                            OutputCircle::E,
                        ];
                        for cdir in dirs.iter() {
                            if !crossing_builder.has_connection(InOut::OUT, cdir.as_dir()) {
                                let id = commands
                                    .spawn_bundle(ConnectorCircleOut::new(*cdir, theme.connector_out))
                                    .id();
                                connectors.push(id);
                            }
                        }
                    }
               }
 
            },
            AddStreetStage::SelectingInput => {
                let street_info = street
                    .single()
                    .expect("Unable to get street even though stage is set to SelectingInput");
                if nbr.get_id() == street_info.start_id.0 {
                    return 
                }
                match &**nbr {
                    NodeBuilder::Street(_) => return,
                    NodeBuilder::IONode(_node) => {
                        // draw a connector in the middle of the ionode
                        let id = commands
                            .spawn_bundle(ConnectorCircleIn::new(InputCircle::Middle, theme.connector_in))
                            .id();
                        connectors.push(id);
                    },
                    NodeBuilder::Crossing(crossing_builder) => {
                        let dirs = [
                            InputCircle::N,
                            InputCircle::S,
                            InputCircle::W,
                            InputCircle::E
                        ];
                        for cdir in dirs.iter() {
                            if !crossing_builder.has_connection(InOut::IN, cdir.as_dir()) {
                                let id = commands
                                    .spawn_bundle(ConnectorCircleIn::new(*cdir, theme.connector_in))
                                    .id();
                                connectors.push(id);
                            }
                        }
                    }
               }
 
            },
        }
        commands
            .entity(entity)
            .push_children(&connectors)
            .insert(HasConnectors);
    }
}

/// Stores info needed when constructing a new street
pub struct NewStreetInfo {
    pub start_id: SimulationID,
    pub out_conn_type: OutputCircle,
}

/// is responsible for adding a new street if the out connector was clicked
/// and for managing the stage of the tool
pub fn connector_clicked(
    mut stage: ResMut<AddStreetStage>,
    out_circles: QuerySet<(
        Query<(&Parent, &GlobalTransform, &OutputCircle), With<UnderCursor>>,
        Query<Entity, With<OutputCircle>>,
    )>,
    in_circles: QuerySet<(
        Query<(&Parent, &GlobalTransform, &InputCircle), With<UnderCursor>>,
        Query<Entity, With<InputCircle>>,
    )>,
    street: Query<(Entity, &NewStreetInfo, &StreetLinePosition), With<PlacingStreet>>,
    parent_nodes: Query<&SimulationID>,
    mut sim_manager: ResMut<SimManager>,
    windows: Res<Windows>,
    theme: Res<UITheme>,
    mut ui_state: ResMut<UIState>,
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    camera: Query<&Transform, With<Camera>>,
) {
    let mut mouse_pos = match handle_mouse_clicks(&mouse_input, &windows) {
        Some(p) => p,
        None => return,
    };

    if let Ok(cam) = camera.single() {
        mouse_pos = mouse_to_world_space(&cam, mouse_pos, &windows);
    }
    match *stage {
        AddStreetStage::SelectingOutput => {
            if let Ok((parent_node, pos, ctype)) = out_circles.q0().single() {
                let start = Vec2::new(pos.translation.x, pos.translation.y);
                let new_street = node_render::street(start, mouse_pos, theme.placing_street);
                let id = parent_nodes
                    .get(parent_node.0)
                    .expect("There is no parent for connector!");
                info!("Starting to create new Street at position {:?}", start);
                commands
                    .spawn()
                    .insert_bundle(new_street)
                    .insert(PlacingStreet)
                    .insert(NewStreetInfo {
                        start_id: id.clone(),
                        out_conn_type: *ctype,
                    })
                    .insert(StreetLinePosition(start, mouse_pos));
                *stage = AddStreetStage::SelectingInput;
                // lock toolbar to prevent the user from switching to another tool while
                // still connecting crossings
                ui_state.toolbar.locked = true;
                // delete the connectors
                out_circles.q1().iter().for_each(| c | {
                    commands.entity(c).despawn();
                });
                commands.entity(parent_node.0).remove::<HasConnectors>();
            }
        }
        AddStreetStage::SelectingInput => {
            if let Ok((parent, _pos, ctype)) = in_circles.q0().single() {
                let (entity, street_info, street_pos) = street
                    .single()
                    .expect("Unable to get street even though input connector was clicked");
                let end_id = parent_nodes
                    .get(parent.0)
                    .expect("There is no parent for connector!");
                let builder = match sim_manager.modify_sim_builder() {
                    Ok(b) => b,
                    Err(_) => return,
                };
                let new_street = match builder.connect_with_street(
                    (street_info.start_id.0, street_info.out_conn_type.as_dir()),
                    (end_id.0, ctype.as_dir()),
                    1,
                ) {
                    Ok(s) => s,
                    Err(e) => panic!("{}", e),
                };
                let new_street_id = new_street.get().get_id();
                let street_bundle = StreetBundle::new(
                    new_street_id,
                    new_street,
                    street_pos.0,
                    street_pos.1,
                    theme.street,
                );
                info!("new Street with position {} {}", street_pos.0, street_pos.1);
                commands
                    .entity(entity).despawn();
                commands.spawn_bundle(street_bundle);
                *stage = AddStreetStage::SelectingOutput;
                // delete the connectors
                in_circles.q1().iter().for_each(| c | {
                    commands.entity(c).despawn();
                });
                ui_state.toolbar.locked = false;
                commands.entity(parent.0).remove::<HasConnectors>();
            }
        }
    }
}

/// removes connectors if the mouse leaves a set distance
pub fn remove_connectors_out_of_bounds(
    mut commands: Commands,
    conn_query: Query<(Entity, &Children, &Transform), With<HasConnectors>>,
    windows: Res<Windows>,
    camera: Query<&Transform, With<Camera>>,
) {
    let window = windows.get_primary().unwrap();
    let mut mouse_pos = match window.cursor_position() {
        Some(p) => p,
        None => return,
    };

    if let Ok(cam) = camera.single() {
        mouse_pos = mouse_to_world_space(&cam, mouse_pos, &windows);
    }
    let max_dist_sqr = CONNECTOR_DISPLAY_RADIUS * CONNECTOR_DISPLAY_RADIUS;
    conn_query
        .iter()
        .for_each(|(entity, connectors, transform)| {
            let conn_pos = Vec2::new(transform.translation.x, transform.translation.y);
            if (conn_pos - mouse_pos).length_squared() > max_dist_sqr {
                // remove connectors
                connectors.iter().for_each(|c| {
                    commands.entity(*c).despawn();
                });
                commands.entity(entity).remove::<HasConnectors>();
            }
        });
}

/// marks a street that is currently being placed
pub struct PlacingStreet;

/// renders the street that is produced when an output connecter of a crossing is clicked
pub fn render_new_street(
    mut street_query: Query<(Entity, &mut StreetLinePosition), With<PlacingStreet>>,
    mut commands: Commands,
    windows: Res<Windows>,
    camera: Query<&Transform, With<Camera>>,
    theme: Res<UITheme>,
) {
    let window = windows.get_primary().unwrap();
    let mut mouse_pos = match window.cursor_position() {
        Some(p) => p,
        None => return,
    };

    if let Ok(cam) = camera.single() {
        mouse_pos = mouse_to_world_space(&cam, mouse_pos, &windows);
    }
    if let Ok((entity, mut line_position)) = street_query.single_mut() {
        *line_position.1 = *mouse_pos;
        let new_shape_bundle =
            node_render::street(line_position.0, line_position.1, theme.placing_street);
        commands
            .entity(entity)
            .remove_bundle::<ShapeBundle>()
            .insert_bundle(new_shape_bundle);
    }
}

pub fn add_crossing_system(
    mut commands: Commands,
    mut sim_manager: ResMut<SimManager>,
    mouse_input: Res<Input<MouseButton>>,
    theme: ResMut<UITheme>,
    windows: Res<Windows>,
    camera: Query<&Transform, With<Camera>>,
) {
    let mut mouse_click = match input::handle_mouse_clicks(&mouse_input, &windows) {
        Some(click) => click,
        None => return,
    };
    //
    if let Ok(cam) = camera.single() {
        mouse_click = mouse_to_world_space(&cam, mouse_click, &windows);
    }

    let simulation_builder = match sim_manager.modify_sim_builder() {
        Ok(builder) => builder,
        Err(_) => {
            eprintln!("Can't modify street builder to add crossing");
            return;
        }
    };
    let nbr = simulation_builder.add_node(NodeBuilder::Crossing(CrossingBuilder::new()));
    let id = nbr.get().get_id();
    info!("Added Crossing wit id= {}", id);
    commands.spawn_bundle(CrossingBundle::new(id, nbr, mouse_click, theme.crossing));
}

pub fn add_io_node_system(
    mut commands: Commands,
    mut sim_manager: ResMut<SimManager>,
    mouse_input: Res<Input<MouseButton>>,
    theme: ResMut<UITheme>,
    windows: Res<Windows>,
    camera: Query<&Transform, With<Camera>>,
) {
    let mut mouse_click = match input::handle_mouse_clicks(&mouse_input, &windows) {
        Some(click) => click,
        None => return,
    };
    //
    if let Ok(cam) = camera.single() {
        mouse_click = mouse_to_world_space(&cam, mouse_click, &windows);
    }

    let simulation_builder = match sim_manager.modify_sim_builder() {
        Ok(builder) => builder,
        Err(_) => {
            eprintln!("Can't modify street builder to add crossing");
            return;
        }
    };
    let nbr = simulation_builder.add_node(NodeBuilder::IONode(IONodeBuilder::new()));
    let id = nbr.get().get_id();
    info!("Added IONode with id= {}", id);
    commands.spawn_bundle(IONodeBundle::new(id, nbr, mouse_click, theme.io_node));
}

/// Marker for the currently connected node
pub struct SelectedNode;

pub fn delete_node_system_simple(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut sim_manager: ResMut<SimManager>,
    nodes: QuerySet<(
        Query<(Entity, &SimulationID), (With<NodeType>, With<UnderCursor>)>,
        Query<(Entity, &SimulationID), (With<NodeType>, Without<UnderCursor>)>,
    )>,
    mut commands: Commands,
) {
    let mut _mouse_click = match input::handle_mouse_clicks(&mouse_input, &windows) {
        Some(click) => click,
        None => return,
    };

    if let Ok((entity, sim_id)) = nodes.q0().single() {
        if let Ok(sim_builder) = sim_manager.modify_sim_builder() {
            commands.entity(entity).despawn();
            let removed_nodes = sim_builder
                .remove_node_and_connected_by_id(sim_id.0)
                .expect("Unable to remove node");
            let indices_to_remove: Vec<usize> = removed_nodes
                .iter()
                .map(|node| node.get().get_id())
                .collect();
            for (entity, sim_index) in nodes.q1().iter() {
                if indices_to_remove.contains(&sim_index.0) {
                    info!(
                        "Deleting Node wit id= {} (Entity: {:?})",
                        sim_index.0, entity
                    );
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}


pub fn select_node(
    mut commands: Commands,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    shapes: QuerySet<(
        Query<(Entity, &Transform, &NodeType)>,
        Query<Entity, With<SelectedNode>>,
    )>,
    camera: Query<&Transform, With<Camera>>,
) {
    let mouse_click = match input::handle_mouse_clicks(&mouse_input, &windows) {
        Some(click) => click,
        None => return,
    };
    let (entity, _, _) =
        match input::get_shape_under_mouse(mouse_click, windows, shapes.q0().iter(), &camera) {
            Some(s) => s,
            None => return,
        };
    if let Ok(prev_selected) = shapes.q1().single() {
        info!("Unselected previouse node (entity={:?})", prev_selected);
        commands
            .entity(prev_selected)
            .remove::<SelectedNode>()
            .insert(NeedsRecolor);
    }
    info!("Selecting node (entity={:?})", entity);
    commands
        .entity(entity)
        .insert(SelectedNode)
        .insert(NeedsRecolor);
}
