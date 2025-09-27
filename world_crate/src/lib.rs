#![crate_type = "rlib"]
#![allow(dead_code, clippy::let_and_return)]

pub mod brush;
pub mod entity;
pub mod vertex;

pub use vertex::Vertex;

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
