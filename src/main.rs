//! A rendering engine, which can load maps into memory, and render them to a screen.
//!
//! ## What is this/what will this be?
//! - [x] A 3d renderer
//! - [ ] A map loader
//! - [ ] Some form of backing for a game
//! ## What is this NOT?
//! * A portable interface for you to make your own games
//!   * at least, not yet
//! * A real project that will have an end
#![allow(dead_code, clippy::let_and_return)]
#![warn(clippy::missing_docs_in_private_items)]

extern crate render;
extern crate world;

use anyhow::{Context as _, Result, format_err};
use render::{
    Camera, Render, ScreenSpaceMesh, gl, gl_upd_viewport,
    imgui::{self, Context},
    imgui_wrappers::{renderer::ImguiRenderer, sdlplatform::SdlPlatform as ImguiSdlPlatform},
};
use sdl2::{
    EventPump, Sdl, VideoSubsystem,
    event::WindowEvent,
    keyboard::Keycode,
    video::{self, GLContext},
};
use std::{
    ffi::{CStr, c_char, c_void},
    fs::File,
    io::{self, Write},
    ptr::null,
    thread,
    time::{Duration, Instant},
};

mod map;
mod ui;

use world::World;

use crate::{
    map::parser::parse_map,
    ui::{Ui, ui_manager::UiManager},
};

/// This determines all values related to framecapping!
///
/// Note: this is SOFT due to the fact that we may or may not sleep
/// less, since we do calculations to not over-sleep, which may not
/// be perfect if the frame was rendered very quickly
pub const SOFT_FPS_CAP: u64 = 30;

pub const OPENGL_MAJOR_VER: u8 = 4;
pub const OPENGL_MINOR_VER: u8 = 3;

pub const MAX_MICROS_BETWEEN_FRAMES: u64 = 1_000_000 / SOFT_FPS_CAP;
pub const MAX_MILLIS_BETWEEN_FRAMES: u64 = MAX_MICROS_BETWEEN_FRAMES / 1000;

pub const DURATION_BETWEEN_FRAMES: Duration = Duration::from_micros(MAX_MICROS_BETWEEN_FRAMES);

pub const DURATION_PER_30FPS: Duration = Duration::from_micros(1_000_000 / 30);
pub const DURATION_PER_60FPS: Duration = Duration::from_micros(1_000_000 / 60);
pub const DURATION_PER_144FPS: Duration = Duration::from_micros(1_000_000 / 144);

pub const START_WIDTH: u32 = 800;
pub const START_HEIGHT: u32 = 600;

// void APIENTRY glDebugOutput(GLenum source, GLenum type, unsigned int id, GLenum severity,
//                            GLsizei length, const char *message, const void *userParam);

/// OpenGL debug output callback.
extern "system" fn gl_debug_output(
    _source: gl::types::GLenum,
    output_type: gl::types::GLenum,
    _id: gl::types::GLuint,
    _severity: gl::types::GLenum,
    _length: gl::types::GLsizei,
    message: *const c_char,
    _user_param: *mut c_void,
) {
    let type_str = match output_type {
        gl::DEBUG_TYPE_ERROR => "Type: Error",
        gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Type: Deprecated Behaviour",
        gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Type: Undefined Behaviour",
        gl::DEBUG_TYPE_PORTABILITY => "Type: Portability",
        gl::DEBUG_TYPE_PERFORMANCE => "Type: Performance",
        gl::DEBUG_TYPE_MARKER => "Type: Marker",
        gl::DEBUG_TYPE_PUSH_GROUP => "Type: Push Group",
        gl::DEBUG_TYPE_POP_GROUP => "Type: Pop Group",
        gl::DEBUG_TYPE_OTHER => "Type: Other",
        _ => unimplemented!(),
    };
    let message_str = unsafe { CStr::from_ptr(message) };
    let message_str = message_str.to_string_lossy();
    eprintln!("Debug output called. \n{type_str}\nMessage: {message_str}");
}

fn main() -> Result<()> {
    let (_sdl_ctx, video_ctx, mut event_pump) = init_sdl()?;

    let (window, main_id, gl_ctx) = make_main_window(&video_ctx).map_err(|e| format_err!(e))?;

    // setup gl loading with sdl
    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    let mut s = String::with_capacity(64);

    // print!("map: ");
    // io::stdout().flush()?;
    // io::stdin().read_line(&mut s)?;

    s.pop();
    s.push_str("test");
    let map = format!("maps/{}.map", s);
    let map_file = File::open(&map)?;
    let map_data = parse_map(map_file).context(map)?;
    gl_upd_viewport(START_WIDTH, START_HEIGHT);
    gl_setup();

    let mut render_ctx = Render::init(&gl_ctx);

    render_ctx
        .clear()
        .map_err(|_| format_err!("Error clearing screen"))?;
    window.gl_swap_window();

    let (mut imgui, mut imgui_platform, mut imgui_renderer) = imgui_create();

    let world = World::new();
    eprintln!("World: 0x{:x}", (&raw const world).addr());
    let mut _camera = Camera::default();

    let screen_world = ScreenSpaceMesh::simple();

    let mut frame_width: u32 = START_WIDTH;
    let mut frame_height: u32 = START_HEIGHT;

    let mut ui = UiManager::new();

    // how much time last frame took to render
    let mut delta_time = 0.;
    'going: loop {
        let instant_loop_start = Instant::now();
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
                    frame_width = width.try_into()?;
                    frame_height = height.try_into()?;
                }
                _ => {}
            }
        }

        gl_upd_viewport(frame_width, frame_height);
        render_ctx
            .clear()
            .map_err(|_| format_err!("Error clearing screen"))?;
        render_ctx
            .render_world(&screen_world)
            .map_err(|_| format_err!("Error rendering world"))?;

        imgui_platform.prepare_frame(&mut imgui, &window, &event_pump);
        let frame = imgui.new_frame();

        ui.update(delta_time);
        ui.draw(frame);

        let draw_data = imgui.render();
        imgui_renderer.render(draw_data);

        window.gl_swap_window();

        let instant_before_sleep = Instant::now();

        // Soft cap fps
        let opt = DURATION_BETWEEN_FRAMES
            .checked_sub(instant_before_sleep.duration_since(instant_loop_start));
        // Are we under the max time between frames?
        if let Some(time) = opt {
            thread::sleep(time);
        }

        let instant_after_sleep = Instant::now();

        let frametime = instant_after_sleep
            .duration_since(instant_loop_start)
            .as_secs_f64();

        ui.debug.push(frametime);

        delta_time = Instant::now()
            .duration_since(instant_loop_start)
            .as_secs_f64();
    }
    Ok(())
}

/// Setup all the things that we need for this opengl context.
/// Currently only handles debug callbacks.
fn gl_setup() {
    unsafe {
        // setup debug logging and filtering
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        gl::DebugMessageCallback(Some(gl_debug_output), null());
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DONT_CARE,
            0,
            null(),
            gl::FALSE,
        );
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DEBUG_SEVERITY_HIGH,
            0,
            null(),
            gl::TRUE,
        );
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DEBUG_SEVERITY_MEDIUM,
            0,
            null(),
            gl::TRUE,
        );
        gl::DebugMessageControl(
            gl::DONT_CARE,
            gl::DONT_CARE,
            gl::DEBUG_SEVERITY_LOW,
            0,
            null(),
            gl::TRUE,
        );
    }
}

/// Initialize all values necessary for SDL.
fn init_sdl() -> Result<(Sdl, VideoSubsystem, EventPump)> {
    let sdl_ctx = sdl2::init().map_err(|e| format_err!(e))?;

    let video_ctx = sdl_ctx.video().map_err(|e| format_err!(e))?;
    video_ctx
        .gl_load_library_default()
        .map_err(|e| format_err!(e))?;

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

    let event_pump = sdl_ctx.event_pump().map_err(|e| format_err!(e))?;

    Ok((sdl_ctx, video_ctx, event_pump))
}

/// Create the main SDL window.
/// Returns the window, its id, and its OpenGL Context.
fn make_main_window(
    video_ctx: &sdl2::VideoSubsystem,
) -> Result<(video::Window, u32, GLContext), String> {
    let window = video_ctx
        .window("SDL world test", 800, 600)
        .position_centered()
        // .resizable()
        .opengl()
        .build()
        .map_err(|_| String::from(concat!("Error creating window. {} {}", file!(), line!())))?;

    let gl_ctx = window.gl_create_context()?;
    video_ctx.gl_set_swap_interval(0)?;

    let main_id = window.id();

    Ok((window, main_id, gl_ctx))
}

/// Setup and create everything for ImGui.
fn imgui_create() -> (Context, ImguiSdlPlatform, ImguiRenderer) {
    let mut imgui = Context::create();
    /* disable creation of files on disc */
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);

    /* setup platform and renderer, and fonts to imgui */
    imgui
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    let imgui_platform = ImguiSdlPlatform::new(&mut imgui);
    let imgui_renderer = ImguiRenderer::new(&mut imgui);
    (imgui, imgui_platform, imgui_renderer)
}
