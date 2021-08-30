use bevy::prelude::*;

/// This trait is used to update the Node 
/// 
/// The idea is to for this function to call the
/// the `update_graphics` function of the Movable internally
/// to render the Movable. (The movable internally does not 
/// know its position)
pub trait RenderNode {
    fn update_graphics(&mut self, transform: &mut Transform, sprite: &mut Sprite) -> Result<(), Box<dyn Error>>;
}
/// Should update the look of the car
pub trait RenderMovable {
    fn update_graphics(&mut self, velocity: Vec2, transform: &mut Transform, sprite: &mut Sprite) -> Result<(), Box<dyn Error>>;
}