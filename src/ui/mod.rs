//! UI, UI elements and associated functions.
pub mod debug_ui;
pub mod ui_manager;

use crate::imgui;

/// A drawable UI element.
pub trait Ui {
    fn update(&mut self, delta_time: f64);
    fn draw(&mut self, context: &mut imgui::Ui);
}
