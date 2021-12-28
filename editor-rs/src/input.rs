use bevy::{
    input::{
        mouse::{MouseMotion, MouseWheel},
        Input,
    },
    math::{Vec2, Vec3},
    prelude::{Entity, EventReader, KeyCode, MouseButton, Query, Res, ResMut, Transform, With},
    window::Windows,
};

use crate::{
    get_primary_window_size, toolbar::ToolType, Camera, NodeBuilderRef, NodeType, SimulationID,
    UIState, CONNECTION_CIRCLE_RADIUS, CROSSING_SIZE, IONODE_SIZE,
};

const MIN_X: f32 = 300.0;
const MAX_X: f32 = 100.0;
const PAN_SPEED: f32 = 10.0;

/// used to mark the circles used to connect the outputs of crossings
#[derive(Clone, Copy)]
pub enum OutputCircle {
    N,
    S,
    W,
    E,
}

/// used to mark the circles used to connect the inputs of crossigns
#[derive(Clone, Copy)]
pub enum InputCircle {
    N,
    S,
    W,
    E,
}

/// returns the Output Circle that the mouse is currently over
///
/// This is used to be able to connect different sides of a crossing with
/// another. (The Circle you clicked on represents one side of the crossing)
pub fn is_mouse_over_out_circle(
    mouse_pos: Vec2,
    windows: Res<Windows>,
    circles: Query<(Entity, &Transform, &OutputCircle)>,
    camera: &Query<&Transform, With<Camera>>,
) -> Option<(Entity, Transform, OutputCircle)> {
    let camera_transform = match camera.single() {
        Ok(cam) => cam,
        Err(_) => return None,
    };
    // camera scaling factor
    let scaling = camera_transform.scale.x;
    // get position of 0,0 of world coordinate system in screen coordinates
    let midpoint_screenspace = (get_primary_window_size(&windows) / 2.0)
        - (Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        )) / scaling;
    let min_dist_circle_sqr = CONNECTION_CIRCLE_RADIUS * CONNECTION_CIRCLE_RADIUS;
    circles
        .iter()
        .filter(|(_entity, transform, out_circle)| {
            // get shape position in screen coordinates
            let position = midpoint_screenspace
                + (Vec2::new(transform.translation.x, transform.translation.y)) / scaling;
            // calculate distance, squared to improve performance so does not need to be rooted
            let dist = (position - mouse_pos).length_squared();
            dist <= min_dist_circle_sqr
        })
        .map(|(e, t, c)| (e, t.clone(), c.clone()))
        .next()
}
/// returns the Input Circle that the mouse is currently over
///
/// This is used to be able to connect different sides of a crossing with
/// another. (The Circle you clicked on represents one side of the crossing)
pub fn is_mouse_over_in_circle(
    mouse_pos: Vec2,
    windows: Res<Windows>,
    circles: Query<(Entity, &Transform, &InputCircle)>,
    camera: &Query<&Transform, With<Camera>>,
) -> Option<(Entity, Transform, InputCircle)> {
    let camera_transform = match camera.single() {
        Ok(cam) => cam,
        Err(_) => return None,
    };
    // camera scaling factor
    let scaling = camera_transform.scale.x;
    // get position of 0,0 of world coordinate system in screen coordinates
    let midpoint_screenspace = (get_primary_window_size(&windows) / 2.0)
        - (Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        )) / scaling;
    let min_dist_circle_sqr = CONNECTION_CIRCLE_RADIUS * CONNECTION_CIRCLE_RADIUS;
    circles
        .iter()
        .filter(|(_entity, transform, in_circle)| {
            // get shape position in screen coordinates
            let position = midpoint_screenspace
                + (Vec2::new(transform.translation.x, transform.translation.y)) / scaling;
            // calculate distance, squared to improve performance so does not need to be rooted
            let dist = (position - mouse_pos).length_squared();
            dist <= min_dist_circle_sqr
        })
        .map(|(e, t, c)| (e, t.clone(), c.clone()))
        .next()
}

pub fn get_shape_under_mouse<'a, T: Iterator<Item=(Entity, &'a Transform, &'a NodeType)>>(
    mouse_pos: Vec2,
    windows: Res<Windows>,
    shapes: T,// &Query<(Entity, &Transform, &NodeType)>,
    camera: &Query<&Transform, With<Camera>>,
) -> Option<(Entity, Transform, NodeType)> {
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
        let min_dist_io = IONODE_SIZE * IONODE_SIZE;
        let half_square_side_len = CROSSING_SIZE / 2.0;
        let mut shapes_under_cursor = shapes.filter(|(_entity, transform, node_type)| {
            match node_type {
                NodeType::CROSSING => {
                    // get shape position in screen coordinates
                    let position = midpoint_screenspace
                        + (Vec2::new(transform.translation.x, transform.translation.y)) / scaling;
                    // is the mouse in the square?
                    position.x - half_square_side_len <= mouse_pos.x
                        && mouse_pos.x <= position.x + half_square_side_len
                        && position.y - half_square_side_len <= mouse_pos.y
                        && mouse_pos.y <= position.y + half_square_side_len
                }
                NodeType::IONODE => {
                    // get shape position in screen coordinates
                    let position = midpoint_screenspace
                        + (Vec2::new(transform.translation.x, transform.translation.y)) / scaling;
                    // calculate distance, squared to improve performance so does not need to be rooted
                    let dist = (position - mouse_pos).length_squared();
                    dist <= min_dist_io
                }
                NodeType::STREET => false, // streets can't be selected
            }
        });
        return match shapes_under_cursor.next() {
            Some((e, t, n)) => Some((e, t.clone(), n.clone())),
            None => None,
        };
    }
    None
}

pub fn get_nearest_shapes<'a>(
    click_pos: Vec2,
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
        let shape_dists =
            shapes
                .iter()
                .map(|(entity, transform, node_type, sim_id, node_builder_ref)| {
                    // get shape position in scren coordinates
                    let position = midpoint_screenspace
                        + (Vec2::new(transform.translation.x, transform.translation.y)) / scaling;
                    // calculate distance, squared to improve performance so does not need to be rooted
                    let dist = (position - click_pos).length_squared();
                    let shape_clicked = ShapeClicked {
                        dist: Some(dist),
                        entity,
                        transform,
                        node_type,
                        sim_id,
                        node_builder_ref,
                    };
                    // check if this entity is the one currently selected
                    //  (we need to change the color later on when unselecting it)
                    //if let Some(selected_nb) = &uistate.selected_node {
                    //    if selected_nb.0 == shape_clicked.node_builder_ref.0 {
                    //        prev_selection =
                    //            Some(shape_clicked.clone());
                    //    }
                    //}
                    shape_clicked
                });
        shape_dists.for_each(|shape_information| {
            if let Some(old_nearest) = &closest_shape {
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
    pub dist: Option<f32>,
    pub entity: Entity,
    pub transform: &'a Transform,
    pub node_type: &'a NodeType,
    pub sim_id: &'a SimulationID,
    pub node_builder_ref: &'a NodeBuilderRef,
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

// pans canvas
pub fn keyboard_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    let speed: f32 = PAN_SPEED;
    for mut transform in camera.iter_mut() {
        let s: Vec3 = transform.scale;
        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += speed * s.x;
        }
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= speed * s.x;
        }
        if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += speed * s.y;
        }
        if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= speed * s.y;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            transform.scale += Vec3::from((0.1 * s.x, 0.1 * s.y, 0.0));
        }
        if keyboard_input.pressed(KeyCode::E) {
            transform.scale -= Vec3::from((0.1 * s.x, 0.1 * s.y, 0.0));
        }
    }
}
