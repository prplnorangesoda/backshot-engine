use crate::{brush::Brush, entity::Entity};

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
    pub fn add_brush<T: Brush + 'static>(&mut self, brush: T) {
        self.brushes.push(Box::new(brush));
    }
}
