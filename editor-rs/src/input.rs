use bevy::{prelude::{Res, Entity, Query, Transform, ResMut, With, MouseButton, EventReader}, window::Windows, math::{Vec2, Vec3}, input::{Input, mouse::{MouseMotion, MouseWheel}}};

use crate::{NodeType, SimulationID, NodeBuilderRef, UIState, get_primary_window_size, toolbar::ToolType, Camera};

const MIN_X: f32 = 300.0;
const MAX_X: f32 = 100.0;



pub fn intersect_shapes_with_click<'a>(click_pos: Vec2,
    windows: Res<Windows>,
    shapes: &'a Query<(
        Entity,
        &Transform,
        &NodeType,
        &SimulationID,
        &NodeBuilderRef,
    )>,
    uistate: &mut ResMut<UIState>,
    camera: &Query<&Transform, With<Camera>>,
) -> (Option<ShapeClicked<'a>>, Option<ShapeClicked<'a>>) {
    let mut closest_shape: Option<ShapeClicked<'a>> = None;
    let mut prev_selection: Option<ShapeClicked> = None;
    // println!("{:?}", click_pos);
    if let Some(camera_transform) = camera.iter().next() {
        // camera scaling factor
        let scaling = camera_transform.scale.x;
        // get position of 0,0 of world coordinate system in screen coordinates
        let midpoint_screenspace = (get_primary_window_size(&windows) / 2.0)
            - (Vec2::new(
                camera_transform.translation.x,
                camera_transform.translation.y,
            )) / scaling;
        let shape_dists = shapes.iter().map(
            |(entity, transform, node_type, sim_index, node_builder_ref)| {
                // get shape position in scren coordinates
                let position = midpoint_screenspace
                    + (Vec2::new(
                        transform.translation.x,
                        transform.translation.y,
                    )) / scaling;
                // calculate distance, squared to improve performance so does not need to be rooted
                let dist = (position - click_pos).length_squared();
                let shape_clicked = ShapeClicked{
                    dist,
                    entity,
                    transform,
                    node_type,
                    sim_index,
                    node_builder_ref,
                };
                // check if this entity is the one currently selected
                //  (we need to change the color later on when unselecting it)
                if let Some(selected_nb) = &uistate.selected_node {
                    if selected_nb.0 == shape_clicked.node_builder_ref.0 {
                        prev_selection =
                            Some(shape_clicked.clone());
                    }
                }
                shape_clicked
            },
        );
        shape_dists.for_each(| shape_information| {
            if let Some(old_nearest) = &closest_shape
            {
                if shape_information.dist < old_nearest.dist {
                    closest_shape = Some(shape_information);
                }
            } else {
                closest_shape = Some(shape_information);
            }
        });
    }
    (closest_shape, prev_selection)
}

// used by the code that check if a shape was clicked
#[derive(Clone)]
pub struct ShapeClicked<'a> {
    pub dist: f32,
    pub entity: Entity,
    pub transform: &'a Transform,
    pub node_type: &'a NodeType,
    pub sim_index: &'a SimulationID,
    pub node_builder_ref: &'a NodeBuilderRef
}

// for selection
pub fn handle_mouse_clicks(
    mouse_input: &Res<Input<MouseButton>>,
    windows: &Res<Windows>,
) -> Option<Vec2> {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(pos) = win.cursor_position() {
            if (pos.x > MIN_X) && (get_primary_window_size(&windows).x > (MAX_X + pos.x)) {
                return win.cursor_position();
            }
        }
    }
    None
}
pub fn movement_within_bounds(
    mouse_input: &Res<Input<MouseButton>>,
    windows: &Res<Windows>,
    mouse_button: &MouseButton,
) -> bool {
    let win = windows.get_primary().expect("no primary window");
    if mouse_input.pressed(*mouse_button) {
        if let Some(pos) = win.cursor_position() {
            if (pos.x > MIN_X) && (get_primary_window_size(&windows).x > (MAX_X + pos.x)) {
                return true;
            }
        }
        return false;
    }
    // when scrolling, should not terminate
    true
}
pub fn mouse_panning(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    uistate: Res<UIState>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    // change input mapping for orbit and panning here
    let pan_button = MouseButton::Left;
    if movement_within_bounds(&input_mouse, &windows, &pan_button) {
        let mut pan = Vec2::ZERO;
        let mut scroll = 0.0;
        if input_mouse.pressed(pan_button) {
            // Pan only if we're not rotating at the moment
            for ev in ev_motion.iter() {
                pan += ev.delta;
            }
        }
        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }

        for mut transform in camera.iter_mut() {
            if pan.length_squared() > 0.0 && uistate.toolbar.get_tooltype() == ToolType::Pan {
                pan = pan * Vec2::new(transform.scale.x, transform.scale.y);
                transform.translation += Vec3::new(-pan.x, pan.y, 0.0);
            } else if scroll.abs() > 0.0 {
                let scr = f32::powf(1.1, scroll);
                transform.scale *= Vec3::new(scr, scr, 1.0);
            }
        }
    }
}