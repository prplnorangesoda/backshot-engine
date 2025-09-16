pub mod brush;
pub mod entity;

use brush::Brush;
use entity::Entity;

pub struct World<'a> {
    entities: Vec<&'a dyn Entity>,
    brushes: Vec<Box<dyn Brush>>,
}

impl World<'_> {
    pub fn new() -> Self {
        World {
            entities: vec![],
            brushes: vec![],
        }
    }
    // Add a brush to this world.
    pub fn add_brush(&mut self, brush: Box<dyn Brush>) {
        self.brushes.push(brush);
    }
}
