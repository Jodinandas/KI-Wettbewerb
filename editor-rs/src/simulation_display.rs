use bevy::{
    ecs::schedule::ShouldRun,
    math::{Vec2, Vec3},
    prelude::{Color, Commands, Query, Res, ResMut, Transform, Entity},
};
use bevy_egui::egui::Color32;
use bevy_prototype_lyon::{
    entity::ShapeBundle,
    prelude::{DrawMode, FillOptions, GeometryBuilder, ShapeColors},
    shapes,
};
use simulator::SimManager;

use crate::{themes::UITheme, SimulationID, StreetLinePosition, UIState, CAR_SIZE, CAR_Z};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

pub struct CarID(u32);


pub fn run_if_simulating(ui_state: Res<UIState>) -> ShouldRun {
    match ui_state.mode {
        crate::UIMode::Simulator => ShouldRun::Yes,
        _ => ShouldRun::No,
    }
}

fn render_car(pos: Vec2, color: Color) -> ShapeBundle {
    let circle = shapes::Circle {
        radius: CAR_SIZE,
        ..shapes::Circle::default()
    };
    GeometryBuilder::build_as(
        &circle,
        ShapeColors::outlined(color, Color::WHITE),
        DrawMode::Fill(FillOptions::default()),
        Transform::from_xyz(pos.x, pos.y, CAR_Z),
    )
}

/// Displays all cars that are on a street
/// TODO: Jonas' car magic
pub fn display_cars(
    mut commands: Commands,
    sim_manager: ResMut<SimManager>,
    nodes: Query<(&SimulationID, &StreetLinePosition)>,
    mut cars: Query<(Entity, &CarID, &mut Transform)>,
    theme: Res<UITheme>,
) {
    if let Some(updates) = sim_manager.get_status_updates() {
        nodes.for_each(|(sim_id, line)| {
            let id = sim_id.0;
            let start = line.0;
            let end = line.1;
            // println!("start: {}, end: {}", start, end);
            match updates.get(&id) {
                Some(stati) => {
                    stati.iter().for_each(|status| {
                        let new_car_position = start + (end - start) * status.position;
                        match cars.iter_mut().find(|(_e, id, _)| id.0 == status.movable_id) {
                            Some((entity, _, mut transform)) => {
                                match status.delete {
                                    true => commands.entity(entity).despawn(),
                                    false => *transform.translation = *Vec3::new(new_car_position.x, new_car_position.y, CAR_Z)
                                }
                                trace!("Generated new car at {}", new_car_position);
                            }
                            None => {
                                let new_car = render_car(new_car_position, theme.car_color);
                                commands
                                    .spawn_bundle(new_car)
                                    .insert(CarID(status.movable_id));
                                trace!("Generated new car at {}", new_car_position);
                            }
                        };
                    });
                }
                None => {
                    trace!("There is no MovableStatus for node with id {}", id)
                }
            }
        });
    } else {
        // println!("No Updates");
    }
}
