use bevy::{
    input::{
        mouse::{MouseMotion, MouseWheel},
        Input,
    },
    math::{Vec2, Vec3},
    prelude::{
        Commands, Entity, EventReader, GlobalTransform, KeyCode, MouseButton, Or, Query, QuerySet,
        Res, ResMut, Transform, With, Without,
    },
    window::Windows,
};

use crate::{
    get_primary_window_size,
    node_bundles::{InputCircle, OutputCircle},
    tool_systems::mouse_to_world_space,
    toolbar::ToolType,
    Camera, NodeBuilderRef, NodeType, SimulationID, UIState, UnderCursor, CONNECTION_CIRCLE_RADIUS,
    CROSSING_SIZE, IONODE_SIZE,
};

const MIN_X: f32 = 300.0;
const MAX_X: f32 = 100.0;
const PAN_SPEED: f32 = 10.0;

/// This is used to be able to connect different sides of a crossing with
/// another. (The Circle you clicked on represents one side of the crossing)
///
/// If a connector is under the cursor, an [UnderCursor] component is added to it
pub fn mark_connector_under_cursor(
    mut commands: Commands,
    windows: Res<Windows>,
    queries: QuerySet<(
        // previously marked nodes that need to be unmarked
        Query<
            Entity,
            (
                Or<(With<OutputCircle>, With<InputCircle>)>,
                With<UnderCursor>,
            ),
        >,
        // candidates for selection
        Query<(Entity, &GlobalTransform), (Or<(With<OutputCircle>, With<InputCircle>)>)>,
    )>,
    camera: Query<&Transform, With<Camera>>,
) {
    let camera_transform = match camera.single() {
        Ok(cam) => cam,
        Err(_) => return,
    };
    let window = windows.get_primary().unwrap();
    let mouse_pos = match window.cursor_position() {
        Some(p) => mouse_to_world_space(camera_transform, p, &windows),
        None => return,
    };
    // remove connectors previously under the cursor
    queries.q0().iter().for_each(|prev_selected| {
        commands.entity(prev_selected).remove::<UnderCursor>();
    });

    let min_dist_circle_sqr = CONNECTION_CIRCLE_RADIUS * CONNECTION_CIRCLE_RADIUS;
    queries.q1().iter().for_each(|(entity, transform)| {
        let position = Vec2::new(transform.translation.x, transform.translation.y);
        // calculate distance, squared to improve performance so does not need to be rooted
        let dist = (position - mouse_pos).length_squared();
        // mark the node if it is in range
        if dist <= min_dist_circle_sqr {
            commands.entity(entity).insert(UnderCursor);
        }
    });
}

pub fn get_shape_under_mouse<'a, T: Iterator<Item = (Entity, &'a Transform, &'a NodeType)>>(
    m_pos: Vec2,
    windows: Res<Windows>,
    shapes: T, // &Query<(Entity, &Transform, &NodeType)>,
    camera: &Query<&Transform, With<Camera>>,
) -> Option<(Entity, Transform, NodeType)> {
    // println!("{:?}", click_pos);
    if let Ok(camera_transform) = camera.single() {
        // camera scaling factor
        // let scaling = camera_transform.scale.x;
        // get position of 0,0 of world coordinate system in screen coordinates
        let mouse_pos = mouse_to_world_space(camera_transform, m_pos, &windows);
        // dbg!(mouse_pos);
        let min_dist_io = IONODE_SIZE * IONODE_SIZE;
        let half_square_side_len = CROSSING_SIZE / 2.0;
        let mut shapes_under_cursor = shapes.filter(|(_entity, transform, node_type)| {
            match node_type {
                NodeType::CROSSING => {
                    let position = Vec2::new(transform.translation.x, transform.translation.y);
                    // is the mouse in the square?
                    position.x - half_square_side_len <= mouse_pos.x
                        && mouse_pos.x <= position.x + half_square_side_len
                        && position.y - half_square_side_len <= mouse_pos.y
                        && mouse_pos.y <= position.y + half_square_side_len
                }
                NodeType::IONODE => {
                    let position = Vec2::new(transform.translation.x, transform.translation.y);
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

// used by the code that checks if a shape was clicked
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
