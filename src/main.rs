#![allow(dead_code, clippy::let_and_return)]
use std::{
    thread,
    time::{Duration, Instant},
};

use glm::Vec3;
use imgui::{Context, DrawData};
use imgui_sdl2_support::SdlPlatform as ImguiSdlPlatform;
use render::Render;
use sdl2::{event::WindowEvent, keyboard::Keycode, video};

mod brush;
mod entity;
mod imgui_wrappers;
#[macro_use]
mod gl_wrappers;
mod render;
mod vector3;
mod vertex;
mod world;

use vertex::Vertex;
use world::World;

use crate::{
    brush::{BrushPlane, NGonPlane, TriPlane},
    imgui_wrappers::renderer::ImguiRenderer,
};

/// This determines all values related to framecapping!
///
/// Note: this is SOFT due to the fact that we may or may not sleep
/// less, since we do calculations to not over-sleep, which may not
/// be perfect because for some reason keeping time is difficult
const SOFT_FPS_CAP: u64 = 1000;

const OPENGL_MAJOR_VER: u8 = 4;
const OPENGL_MINOR_VER: u8 = 3;

const MAX_MICROS_BETWEEN_FRAMES: u64 = 1_000_000 / SOFT_FPS_CAP;
const MAX_MILLIS_BETWEEN_FRAMES: u64 = MAX_MICROS_BETWEEN_FRAMES / 1000;

const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(MAX_MICROS_BETWEEN_FRAMES);

/// The actual planes rendered to the screen.
#[derive(Default, Debug)]
struct ScreenSpaceMesh {
    planes: Vec<BrushPlane>,
}

impl ScreenSpaceMesh {
    fn add_tri(&mut self, tri: TriPlane) {
        self.planes.push(BrushPlane::Triangle(tri));
    }
    fn add_ngon(&mut self, ngon: NGonPlane) {
        self.planes.push(BrushPlane::NGon(ngon))
    }

    fn clear(&mut self) {
        self.planes.clear();
    }
}

struct Camera {
    pos: Vec3,
    /// XYZ Euler angles. (0,0,0) means upwards.
    /// X: Roll
    /// Y: Pitch
    /// Z: Yaw
    orientation: Vec3,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            orientation: glm::vec3(-90.0, 0.0, 0.0),
            pos: glm::to_vec3(0.),
        }
    }
}

fn main() {
    let sdl_ctx = sdl2::init().unwrap();

    let video_ctx = sdl_ctx.video().unwrap();
    video_ctx.gl_load_library_default().unwrap();

    video_ctx
        .gl_attr()
        .set_context_flags()
        .forward_compatible()
        .debug()
        .set();
    video_ctx
        .gl_attr()
        .set_context_major_version(OPENGL_MAJOR_VER);
    video_ctx
        .gl_attr()
        .set_context_minor_version(OPENGL_MINOR_VER);
    video_ctx
        .gl_attr()
        .set_context_profile(video::GLProfile::Core);
    let window = video_ctx
        .window("SDL world test", 800, 600)
        .position_centered()
        // .resizable()
        .opengl()
        .build()
        .unwrap();

    let main_id = window.id();

    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    unsafe {
        gl::Viewport(0, 0, 800, 600);
    }

    let gl_ctx = window.gl_create_context().unwrap();
    let mut render_ctx = Render::init(&gl_ctx);
    let mut imgui = Context::create();
    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let mut imgui_platform = ImguiSdlPlatform::new(&mut imgui);
    // let mut imgui_renderer = ImguiRenderer::new();

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    let mut world = World::new();
    eprintln!("World: 0x{:x}", (&raw const world).addr());
    let mut _camera = Camera::default();

    let mut screen_world = ScreenSpaceMesh::default();
    screen_world.add_tri(TriPlane([
        // Left
        Vertex {
            pos: glm::vec3(-0.5, -0.5, 0.0),
        },
        // Right
        Vertex {
            pos: glm::vec3(0.5, -0.5, 0.0),
        },
        // Up
        Vertex {
            pos: glm::vec3(-0.5, 0.5, 0.0),
        },
    ]));

    // screen_world.add_tri(TriangleMesh(
    //     Vertex {
    //         pos: glm::vec3(0.5, -0.5, 0.0),
    //     },
    //     Vertex {
    //         pos: glm::vec3(0.5, 0.5, 0.0),
    //     },
    //     Vertex {
    //         pos: glm::vec3(-0.5, 0.5, 0.0),
    //     },
    // ));
    let mut updated_opengl_viewport_this_frame;

    let mut frame_count: u64 = 1;

    let mut frametime_collector = Vec::with_capacity(SOFT_FPS_CAP.try_into().unwrap());
    let mut last_frametime_check = Instant::now();
    'going: loop {
        let time_before_render = Instant::now();
        updated_opengl_viewport_this_frame = false;
        for event in event_pump.poll_iter() {
            imgui_platform.handle_event(&mut imgui, &event);
            use sdl2::event::Event as Ev;
            match event {
                Ev::Quit { .. }
                | Ev::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => {
                    break 'going;
                }
                Ev::Window {
                    timestamp: _,
                    window_id,
                    win_event: WindowEvent::Resized(width, height),
                } if window_id == main_id => {
                    // Only resize the opengl window once per update
                    // The event loop poll happens once per frame, but many
                    // resize events can stack up.
                    if !updated_opengl_viewport_this_frame {
                        updated_opengl_viewport_this_frame = true;
                        unsafe {
                            gl::Viewport(0, 0, width, height);
                        }
                    }
                }
                _ => {}
            }
        }

        render_ctx.clear().unwrap();
        render_ctx.render_world(&screen_world).unwrap();

        // imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);
        // let draw_data = create_ui(&mut imgui);

        // // imgui_renderer.render(draw_data);
        window.gl_swap_window();

        let before_sleep = Instant::now();
        // Soft cap fps
        thread::sleep(
            DURATION_BETWEEN_FRAMES
                .checked_sub(before_sleep.duration_since(time_before_render))
                .unwrap_or(Duration::ZERO),
        );
        let now = Instant::now();
        let time_slept = now.duration_since(before_sleep).as_secs_f64();
        frametime_collector.push(now.duration_since(time_before_render).as_secs_f64());

        // If it's been over a second since
        // last debug print, print it
        if now.duration_since(last_frametime_check).as_secs() >= 1 {
            // can't reduce since we're keeping this Vec around
            let total_time = frametime_collector.iter().fold(0., |acc, item| acc + *item);
            // i love you rust but why can't i turbofish into()
            let len_int32: i32 = frametime_collector.len().try_into().unwrap();
            let len_float: f64 = len_int32.into();
            let avg_time: f64 = total_time / len_float;
            eprintln!(
                "frametime: {avg_time:0.8}, FPS: {:0.8}, frames counted: {:05}",
                1. / avg_time,
                frametime_collector.len()
            );

            frametime_collector.clear();
            last_frametime_check = Instant::now();
        }

        frame_count += 1;
    }
}

fn create_ui(imgui: &mut Context) -> &DrawData {
    let ui = imgui.new_frame();
    ui.show_demo_window(&mut true);
    // ui.text("Hello");

    imgui.render()
}
