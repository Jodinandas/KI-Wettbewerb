use bevy::{
    ecs::schedule::ShouldRun,
    input::Input,
    math::Vec2,
    prelude::{
        Commands, DespawnRecursiveExt, Entity, MouseButton, Query, QuerySet, Res, ResMut,
        Transform, With,
    },
    window::Windows,
};
use bevy_prototype_lyon::entity::ShapeBundle;
use simulator::nodes::NodeBuilderTrait;

use crate::input;
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
pub fn run_if_pan(ttype: Res<UIState>) -> ShouldRun {
    let ttype = match ttype.toolbar.get_selected() {
        Some(t) => t.get_type(),
        None => return ShouldRun::No,
    };
    match ttype {
        ToolType::Pan => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

/// Marker for the currently connected node
pub struct SelectedNode;

pub fn delete_node_system(
    mut sim_manager: ResMut<SimManager>,
    mouse_input: Res<Input<MouseButton>>,
    shape_queries: QuerySet<(
        Query<(&SimulationID,), With<UnderCursor>>,
        Query<(Entity, &SimulationID), With<NodeType>>,
    )>,
    mut commands: Commands,
) {
    if !mouse_input.just_pressed(MouseButton::Left) {
        return;
    }
    // select nearest object
    // get position of mouse click on screen
    if let Ok((sim_id,)) = shape_queries.q0().single() {
        if let Ok(sim_builder) = sim_manager.modify_sim_builder() {
            let removed_nodes = sim_builder
                .remove_node_and_connected_by_id(sim_id.0)
                .expect("Unable to remove node");
            let indices_to_remove: Vec<usize> = removed_nodes
                .iter()
                .map(|node| node.get().get_id())
                .collect();
            for (entity, sim_index) in shape_queries.q1().iter() {
                if indices_to_remove.contains(&sim_index.0) {
                    commands.entity(entity).despawn_recursive();
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
        match input::get_shape_under_mouse(mouse_click, windows, shapes.q0(), &camera) {
            Some(s) => s,
            None => return,
        };
    if let Ok(prev_selected) = shapes.q1().single() {
        commands
            .entity(prev_selected)
            .remove::<SelectedNode>()
            .insert(NeedsRecolor);
    }
    commands
        .entity(entity)
        .insert(SelectedNode)
        .insert(NeedsRecolor);
}

pub fn color_selected() {}

fn toolbarsystem(
    mouse_input: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut sim_manager: ResMut<SimManager>,
    shape_queries: QuerySet<(
        Query<
            (
                Entity,
                &SimulationID,
                &Transform,
                &NodeBuilderRef,
                &NodeType,
            ),
            With<UnderCursor>,
        >,
        Query<(Entity, &SimulationID), With<NodeType>>,
    )>,
    mut uistate: ResMut<UIState>,
    theme: ResMut<UITheme>,
    mut commands: Commands,
    camera: Query<&Transform, With<Camera>>,
) {
    if let Some(tool) = uistate.toolbar.get_selected() {
        match tool.get_type() {
            ToolType::Pan => {
                // nothing. this conde is located in mouse_panning.
            }
            ToolType::DeleteNode => {}
            ToolType::Select => {
                // select nearest object
                // get position of mouse click on screen
                let click_pos = input::handle_mouse_clicks(&mouse_input, &windows);
                // only recognize click if not in gui
                if let Some(pos) = click_pos {
                    let (entity, sim_id, transform, nbr, node_type) =
                        match shape_queries.q0().single() {
                            Ok(result) => result,
                            Err(_) => return,
                        };
                    //if let Some(prev_selection) =
                    //    prev_selection
                    //{
                    //    let pos = prev_selection.transform.translation;
                    //    let new_shape_bundle = match prev_selection.node_type {
                    //        NodeType::CROSSING => {
                    //            node_render::crossing(Vec2::new(pos.x, pos.y), theme.crossing)
                    //        }
                    //        NodeType::IONODE => {
                    //            node_render::io_node(Vec2::new(pos.x, pos.y), theme.io_node)
                    //        }
                    //        NodeType::STREET => {
                    //            todo!("Street selection is not implemented yet!")
                    //        }
                    //    };
                    //    commands
                    //        .entity(prev_selection.entity)
                    //        .remove_bundle::<ShapeBundle>()
                    //        .insert_bundle(new_shape_bundle);
                    //}
                    //uistate.selected_node = Some(nbr.clone());
                    let pos = transform.translation;
                    let new_shape_bundle = match node_type {
                        NodeType::CROSSING => {
                            node_render::crossing(Vec2::new(pos.x, pos.y), theme.highlight)
                        }
                        NodeType::IONODE => {
                            node_render::io_node(Vec2::new(pos.x, pos.y), theme.highlight)
                        }
                        NodeType::STREET => {
                            todo!("Street selection is not implemented yet!")
                        }
                    };
                    commands
                        .entity(entity)
                        .remove_bundle::<ShapeBundle>()
                        .insert_bundle(new_shape_bundle);
                };
            }
            ToolType::None => (),
            ToolType::AddStreet => (),
        }
    }
}
