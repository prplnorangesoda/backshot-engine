use crate::ui::{Ui, debug_ui::DebugUi};

/// A struct to hold all the UI for main.rs
/// to render in one call.
pub struct UiManager {
    pub debug: DebugUi,
}

impl Ui for UiManager {
    fn update(&mut self, delta_time: f64) {
        self.debug.update(delta_time);
    }
    fn draw(&mut self, context: &mut imgui::Ui) {
        self.debug.draw(context);
    }
}

impl UiManager {
    pub fn new() -> Self {
        Self {
            debug: DebugUi::new(),
        }
    }
}
