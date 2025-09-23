//! Standalone functions and modules to wrap the nitty-gritty of
//! OpenGL objects with Rust structs.
#[macro_use]
pub mod program;
pub mod shader;

pub use program::*;
pub use shader::*;

/// Safely update the OpenGL viewport.
pub fn gl_upd_viewport(width: u32, height: u32) {
    let real_width: i32 = width.try_into().unwrap();
    let real_height: i32 = height.try_into().unwrap();
    // SAFETY:
    // gl::Viewport does not fail with non-negative values.
    unsafe {
        gl::Viewport(0, 0, real_width, real_height);
    }
}
