use bevy::prelude::Color;
use bevy_egui::egui::Visuals;

/// This struct stores information about the visual style of the application
///
/// the colors of the simulator can be defined with fields like background
///
/// to change the visuals of the rest of the frontend, use the `egui_visuals` field
pub struct UITheme {
    pub background: Color,
    pub io_node: Color,
    pub street: Color,
    pub crossing: Color,
    pub highlight: Color,
    pub connector_in: Color,
    pub connector_out: Color,
    pub placing_street: Color,
    pub car_color: Color,
    pub egui_visuals: Visuals,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CurrentTheme {
    LIGHT,
    DRACULA,
}

impl UITheme {
    pub fn light() -> UITheme {
        UITheme {
            background: Color::rgb(240.0/255.0, 240.0/255.0, 240.0/255.0),
            io_node: Color::rgb(40.0/255.0, 40.0/255.0, 50.0/255.0),
            street: Color::rgb(120.0/255.0, 120.0/255.0, 130.0/255.0),
            crossing: Color::rgb(40.0/255.0, 40.0/255.0, 50.0/255.0),
            highlight: Color::rgb(140.0/255.0, 160.0/255.0, 140.0/255.0),
            connector_in: Color::rgb(240.0/255.0, 100.0/255.0, 0.0/255.0),
            connector_out: Color::rgb(240.0/255.0, 100.0/255.0, 0.0/255.0),
            placing_street: Color::rgb(100.0/255.0, 140.0/255.0, 100.0/255.0),
            car_color: Color::rgb(100.0/255.0, 240.0/255.0, 100.0/255.0),
            egui_visuals: Visuals::light(),
        }
    }
    pub fn dracula() -> UITheme {
        UITheme {
            background: Color::rgb(40.0/255.0, 42.0/255.0, 54.0/255.0),
            io_node: Color::rgb(255.0/255.0, 184.0/255.0, 108.0/255.0),
            street: Color::rgb(248.0/255.0, 248.0/255.0, 242.0/255.0),
            crossing: Color::rgb(255.0/255.0, 184.0/255.0, 108.0/255.0),
            highlight: Color::rgb(255.0/255.0, 85.0/255.0, 85.0/255.0),
            connector_in: Color::rgb(80.0/255.0, 250.0/255.0, 123.0/255.0),
            connector_out: Color::rgb(80.0/255.0, 250.0/255.0, 123.0/255.0),
            placing_street: Color::rgb(255.0/255.0, 85.0/255.0, 85.0/255.0),
            car_color: Color::rgb(80.0/255.0, 250.0/255.0, 123.0/255.0),
            egui_visuals: Visuals::dark(),
        }
    }
    pub fn from_enum(theme: &CurrentTheme) -> UITheme {
        match theme {
            CurrentTheme::LIGHT => UITheme::light(),
            CurrentTheme::DRACULA => UITheme::dracula(),
        }
    }
}
impl Default for UITheme{
    fn default() -> Self {
        UITheme::dracula()
    }
}