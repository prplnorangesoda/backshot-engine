use std::time::Instant;

use crate::{SOFT_FPS_CAP, ui::Ui};

pub struct DebugUi {
    frametime_collector: Vec<f64>,
    last_debug_check: Instant,
    formatted_str: Option<String>,
}

impl Ui for DebugUi {
    fn update(&mut self, _: f64) {
        if Instant::now()
            .duration_since(self.last_debug_check)
            .as_secs()
            >= 1
        {
            // can't reduce since we're keeping this Vec around
            let total_time = self
                .frametime_collector
                .iter()
                .fold(0., |acc, item| acc + *item);
            let len_float: f64 = self.frametime_collector.len() as f64;
            let avg_time: f64 = total_time / len_float;

            let millis = avg_time * 1000.;

            let formatted = format!(
                "frametime: {millis:0.2}ms, FPS: {:0.1}, frames counted: {:05}",
                1. / avg_time,
                self.frametime_collector.len()
            );
            eprintln!("{}", &formatted);

            self.formatted_str = Some(formatted);
            self.frametime_collector.clear();
            self.last_debug_check = Instant::now();
        }
    }
    fn draw(&mut self, ui: &mut imgui::Ui) {
        let render_str: &str = match &self.formatted_str {
            Some(string) => string,
            None => "no frametime data yet",
        };
        ui.window("debug frametime")
            .size([400., 100.], imgui::Condition::Once)
            .position([0., 0.], imgui::Condition::Once)
            .build(|| {
                ui.bullet_text(render_str);
                ui.tree_node_config("Details").build(|| {
                    ui.text("none yet");
                })
            });
    }
}

impl DebugUi {
    pub fn new() -> Self {
        let frametime_collector = Vec::with_capacity(SOFT_FPS_CAP as usize);

        Self {
            frametime_collector,
            last_debug_check: Instant::now(),
            formatted_str: None,
        }
    }
    pub fn push(&mut self, frametime: f64) {
        self.frametime_collector.push(frametime)
    }
}
