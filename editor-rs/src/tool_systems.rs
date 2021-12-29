use bevy::{
    ecs::schedule::ShouldRun,
    input::Input,
    math::Vec2,
    prelude::{
        Commands, DespawnRecursiveExt, Entity, MouseButton, Query, QuerySet, Res, ResMut,
        Transform, With, Without, BuildChildren,
    },
    window::Windows,
};
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::nodes::{CrossingBuilder, IONodeBuilder, NodeBuilder, NodeBuilderTrait};

use crate::{
    get_primary_window_size, input,
    node_bundles::{CrossingBundle, IONodeBundle, OutputCircle, ConnectorCircleOut},
};
use crate::{
    node_bundles::node_render, sim_manager::SimManager, themes::UITheme, toolbar::ToolType, Camera,
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

pub fn screen_to_world_space(cam: &Transform, windows: &Res<Windows>) -> Vec2 {
    // camera scaling factor
    let scaling = cam.scale.x;
    let midpoint_screenspace = (get_primary_window_size(windows) / 2.0)
        - (Vec2::new(cam.translation.x, cam.translation.y)) / scaling;
    midpoint_screenspace
}

pub fn mouse_to_world_space(cam: &Transform, mouse_pos: Vec2, windows: &Res<Windows>) -> Vec2 {
    let midpoint_screenspace = (get_primary_window_size(windows) / 2.0)
        - (Vec2::new(cam.translation.x, cam.translation.y));
    (mouse_pos - midpoint_screenspace) * cam.scale.x
}

/// A marker for crossings currently displaying connectors
pub struct HasConnectors;

/// generates connectors over crossings, so they can be connected using the
/// Add Street tool
pub fn generate_connectors(
    mut commands: Commands,
    theme: Res<UITheme>,
    node_under_cursor: Query<(Entity, &Transform, &NodeType), (With<UnderCursor>, Without<HasConnectors>)>
) {
    if let Ok((entity, transform, ntype)) = node_under_cursor.single() {
        if *ntype != NodeType::CROSSING { 
            return
        }
        let connectors: Vec<Entity> =
        [OutputCircle::N, OutputCircle::S, OutputCircle::W, OutputCircle::E].iter().map( | direction | 
            commands.spawn_bundle(ConnectorCircleOut::new(*direction, theme.connector_out)).id()).collect();
        commands.entity(entity).push_children(&connectors);
    }
}

pub fn add_street_system() {

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
    println!("Added Crossing wit id= {}", id);
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
    println!("Added IONode with id= {}", id);
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
                    println!(
                        "Deleting Node wit id= {} (Entity: {:?})",
                        sim_index.0, entity
                    );
                    commands.entity(entity).despawn();
                    println!("Deleted Node");
                }
            }
        }
    }
}

/// This systems deletes a node if the cursor is over it and the mouse is clicked
///
/// currently, this system has a weird bug, which causes the system to crash
///
/// to reduce developement time, [delete_node_system_simple] is used instead
pub fn delete_node_system(
    mut sim_manager: ResMut<SimManager>,
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    shape_query: QuerySet<(
        Query<(Entity, &Transform, &NodeType, &SimulationID), With<NodeType>>,
        Query<&Transform, With<Camera>>,
    )>,
    mut commands: Commands,
) {
    let mouse_click = match input::handle_mouse_clicks(&mouse_input, &windows) {
        Some(click) => click,
        None => return,
    };

    let clicked_object = input::get_shape_under_mouse(
        mouse_click,
        windows,
        shape_query.q0().iter().map(|(e, t, n, _id)| (e, t, n)),
        &shape_query.q1(),
    );

    // select nearest object
    // get position of mouse click on screen
    if let Some((entity, transform, node_type)) = clicked_object {
        let sim_id = match shape_query.q0().get(entity) {
            Ok((_e, _t, _n, id)) => id,
            Err(_) => return,
        };

        if let Ok(sim_builder) = sim_manager.modify_sim_builder() {
            let removed_nodes = sim_builder
                .remove_node_and_connected_by_id(sim_id.0)
                .expect("Unable to remove node");
            let indices_to_remove: Vec<usize> = removed_nodes
                .iter()
                .map(|node| node.get().get_id())
                .collect();
            for (entity, _, _, sim_index) in shape_query.q0().iter() {
                if indices_to_remove.contains(&sim_index.0) {
                    println!(
                        "Deleting Node wit id= {} (Entity: {:?})",
                        sim_index.0, entity
                    );
                    commands.entity(entity).despawn();
                    println!("Deleted Node");
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
        println!("Getting prev selected");
        commands
            .entity(prev_selected)
            .remove::<SelectedNode>()
            .insert(NeedsRecolor);
        println!("Got prev selected");
    }
    println!("Getting new selected");
    commands
        .entity(entity)
        .insert(SelectedNode)
        .insert(NeedsRecolor);
    println!("Got new selected");
}
