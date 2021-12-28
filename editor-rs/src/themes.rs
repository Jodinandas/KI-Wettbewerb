use bevy::prelude::Color;
use bevy_egui::egui::Visuals;

/// This struct stores information about the visual style of the application
///
/// the colors of the simulator can be defined with fields like background
///
/// to change the visuals of the rest of the frontend, use the `egui_visuals` field
#[derive(Default)]
pub struct UITheme {
    pub background: Color,
    pub io_node: Color,
    pub street: Color,
    pub crossing: Color,
    pub highlight: Color,
    pub egui_visuals: Visuals,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CurrentTheme {
    LIGHT,
    DARK,
}

impl UITheme {
    pub fn light() -> UITheme {
        UITheme {
            background: Color::rgb(255.0, 255.0, 255.0),
            io_node: Color::rgb(0.0, 200.0, 0.0),
            street: Color::rgb(100., 50., 0.),
            crossing: Color::rgb(0.0, 200.0, 0.0),
            highlight: Color::rgb(255.0, 0.0, 0.0),
            egui_visuals: Visuals::light(),
        }
    }
    pub fn dark() -> UITheme {
        UITheme {
            background: Color::rgb(0.0, 0.0, 0.0),
            io_node: Color::rgb(200.0, 200.0, 0.0),
            street: Color::rgb(255., 255., 255.),
            crossing: Color::rgb(200.0, 200.0, 0.0),
            highlight: Color::rgb(255.0, 0.0, 0.0),
            egui_visuals: Visuals::dark(),
        }
    }
    pub fn from_enum(theme: &CurrentTheme) -> UITheme {
        match theme {
            CurrentTheme::LIGHT => UITheme::light(),
            CurrentTheme::DARK => UITheme::dark(),
        }
    }
}