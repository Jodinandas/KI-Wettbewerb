use bevy::prelude::Color;
use bevy_egui::egui::Visuals;
use bevy_egui::egui::Color32;
use bevy_egui::egui::style;
use bevy_egui::egui::Stroke;

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
    pub text_color: Color32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CurrentTheme {
    LIGHT,
    DRACULA,
}

impl UITheme {
    pub fn light() -> UITheme {
        let text_color = Color32::from_rgb(40,40,50);
        let light_bg_color = Color32::from_rgb(220,220,220);
        let light_bg_color_light = Color32::from_rgb(240,240,240);
        let light_bg_color_dark = Color32::from_rgb(185,185,185);
        let mut visuals = Visuals::light();
        visuals.override_text_color = Some(text_color);
        visuals.extreme_bg_color = light_bg_color;
        visuals.code_bg_color = light_bg_color;
        //visuals.widgets = style::Widgets::style(&self, style::WidgetVisuals::bg_fill(Color32::from_rgb(40,42,54)));
        visuals.widgets = style::Widgets{
            noninteractive: style::WidgetVisuals{
                bg_fill: light_bg_color_dark,
                bg_stroke: Stroke::new(0.5, light_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            inactive: style::WidgetVisuals{
                bg_fill: light_bg_color,
                bg_stroke: Stroke::new(0.5, light_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            hovered: style::WidgetVisuals{
                bg_fill: light_bg_color_light,
                bg_stroke: Stroke::new(0.5, light_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            active: style::WidgetVisuals{
                bg_fill: light_bg_color_light,
                bg_stroke: Stroke::new(0.5, light_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            open: style::WidgetVisuals{
                bg_fill: light_bg_color_light,
                bg_stroke: Stroke::new(0.5, light_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
        };
        UITheme {
            background: Color::rgb(220.0/255.0, 220.0/255.0, 220.0/255.0),
            io_node: Color::rgb(40.0/255.0, 40.0/255.0, 50.0/255.0),
            street: Color::rgb(80.0/255.0, 80.0/255.0, 90.0/255.0),
            crossing: Color::rgb(40.0/255.0, 40.0/255.0, 50.0/255.0),
            highlight: Color::rgb(160.0/255.0, 100.0/255.0, 100.0/255.0),
            connector_in: Color::rgb(240.0/255.0, 100.0/255.0, 0.0/255.0),
            connector_out: Color::rgb(240.0/255.0, 100.0/255.0, 0.0/255.0),
            placing_street: Color::rgb(160.0/255.0, 100.0/255.0, 100.0/255.0),
            car_color: Color::rgb(80.0/255.0, 180.0/255.0, 80.0/255.0),
            egui_visuals: visuals,
            text_color: Color32::from_rgb(0,0,0)
        }
    }
    pub fn dracula() -> UITheme {
        let text_color = Color32::from_rgb(248,248,242);
        let dracula_bg_color = Color32::from_rgb(40,42,54);
        let dracula_bg_color_light = Color32::from_rgb(60,63,81);
        let dracula_bg_color_dark = Color32::from_rgb(30,31,40);
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(text_color);
        visuals.extreme_bg_color = dracula_bg_color;
        visuals.code_bg_color = dracula_bg_color;
        //visuals.widgets = style::Widgets::style(&self, style::WidgetVisuals::bg_fill(Color32::from_rgb(40,42,54)));
        visuals.widgets = style::Widgets{
            noninteractive: style::WidgetVisuals{
                bg_fill: dracula_bg_color_dark,
                bg_stroke: Stroke::new(0.5, dracula_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            inactive: style::WidgetVisuals{
                bg_fill: dracula_bg_color,
                bg_stroke: Stroke::new(0.5, dracula_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            hovered: style::WidgetVisuals{
                bg_fill: dracula_bg_color_light,
                bg_stroke: Stroke::new(0.5, dracula_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            active: style::WidgetVisuals{
                bg_fill: dracula_bg_color_light,
                bg_stroke: Stroke::new(0.5, dracula_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
            open: style::WidgetVisuals{
                bg_fill: dracula_bg_color_light,
                bg_stroke: Stroke::new(0.5, dracula_bg_color_light),
                corner_radius: 0.0,
                fg_stroke: Stroke::new(0.5, text_color),
                expansion: 0.0,
            },
        };
        UITheme {

            egui_visuals: Visuals::dark(),
            background: Color::rgb(40.0/255.0, 42.0/255.0, 54.0/255.0),
            io_node: Color::rgb(255.0/255.0, 184.0/255.0, 108.0/255.0),
            street: Color::rgb(248.0/255.0, 248.0/255.0, 242.0/255.0),
            crossing: Color::rgb(255.0/255.0, 184.0/255.0, 108.0/255.0),
            highlight: Color::rgb(255.0/255.0, 85.0/255.0, 85.0/255.0),
            connector_in: Color::rgb(80.0/255.0, 250.0/255.0, 123.0/255.0),
            connector_out: Color::rgb(80.0/255.0, 250.0/255.0, 123.0/255.0),
            placing_street: Color::rgb(255.0/255.0, 85.0/255.0, 85.0/255.0),
            car_color: Color::rgb(80.0/255.0, 180.0/255.0, 100.0/255.0),
            egui_visuals: visuals,
            text_color,

            //egui_visuals: Visuals{
            //    dark_mode: true,
            //    override_text_color: Some(Color32::RED),
            //    widgets: Widgets::dark(),
            //    selection: Selection,
            //    hyperlink_color: Color32::RED,
            //    faint_bg_color: Color32::RED,
            //    extreme_bg_color: Color32::RED,
            //    code_bg_color: Color32::RED,
            //    window_corner_radius: 0.1,
            //    window_shadow: Shadow::small_dark(),
            //    popup_shadow: Shadow::small_dark(),
            //    resize_corner_size: 0.1,
            //    text_cursor_width: 0.1,
            //    text_cursor_preview: false,
            //    clip_rect_margin: 0.1,
            //    button_frame: false,
            //    collapsing_header_frame: false,
            //}
            //egui_visuals: Visuals::dark().visuals_mut().override_text_color = from_rgb(r: 248, g: 248, b: 24),
        }
    }
    pub fn from_enum(theme: &CurrentTheme) -> UITheme {
        match theme {
            CurrentTheme::LIGHT => UITheme::light(),
            CurrentTheme::DRACULA => UITheme::dracula(),
        }
    }
}
impl Default for UITheme {
    fn default() -> Self {
        UITheme::dracula()
    }
}
