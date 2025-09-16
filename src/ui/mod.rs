use std::time::Duration;

pub mod debug_ui;

pub trait Ui {
    fn update(delta_time: f64);
    fn render(context: &mut imgui::Context);
}
