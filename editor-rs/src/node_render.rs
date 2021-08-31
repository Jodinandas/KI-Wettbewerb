/// This file includes render implementations for the nodes

use simulator::{Crossing, IONode, Street};
use traits::RenderNode;

impl RenderNode for Crossing {
    fn update_graphics(&mut self, 
        transform: &mut Transform,
        sprite: &mut Sprite) 
    -> Result<(), Box<dyn Error>> {
        for car in self.traversible.get_cars() {
            
        }
    }
}