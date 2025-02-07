use sdl2::keyboard::Keycode;

mod vertex;

struct World {
    triangles: Vec<vertex::TriangleMesh>,
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
    video_ctx.gl_attr().set_context_version(3, 3);
    let window = video_ctx
        .window("SDL", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    gl::load_with(|s| video_ctx.gl_get_proc_address(s).cast());

    let gl_ctx = window.gl_create_context().unwrap();

    let mut event_pump = sdl_ctx.event_pump().unwrap();

    'going: loop {
        for event in event_pump.poll_iter() {
            use sdl2::event::Event as Ev;
            match event {
                Ev::Quit { .. }
                | Ev::KeyDown {
                    keycode: Some(Keycode::ESCAPE),
                    ..
                } => {
                    break 'going;
                }
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            render_world();
            render_ui();
        }
        window.gl_swap_window();
    }
    println!("Hello, world!");
}

fn render_world() {
    ()
}

fn render_ui() {
    ()
}
