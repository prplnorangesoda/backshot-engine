use imgui::{DrawCmd, DrawData, internal::RawWrapper};

use crate::vector3::to_byte_slice;

pub struct ImguiRenderer {}

#[cfg(never)]
impl ImguiRenderer {
    pub fn new() -> Self {
        Self {}
    }
    pub fn pre_render(&mut self, data: &DrawData, frame_width: f32, frame_height: f32) {}

    pub fn render(&mut self, data: &DrawData) {
        let frame_width = data.display_size[0] * data.framebuffer_scale[0];
        let frame_height = data.display_size[1] * data.framebuffer_scale[1];
        if frame_width <= 0.0 || frame_height <= 0.0 {
            return;
        }
        self.pre_render(data, frame_width, frame_height);
        for draw_list in data.draw_lists() {
            unsafe {
                let vtx_slice = to_byte_slice(draw_list.vtx_buffer());
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    vtx_slice.len().cast_signed(),
                    vtx_slice.as_ptr().cast(),
                    gl::STREAM_DRAW,
                );
                let idx_slice = to_byte_slice(draw_list.idx_buffer());
                gl::BufferData(
                    gl::ELEMENT_ARRAY_BUFFER,
                    idx_slice.len().cast_signed(),
                    idx_slice.as_ptr().cast(),
                    gl::STREAM_DRAW,
                );
            }
            for command in draw_list.commands() {
                match command {
                    DrawCmd::Elements { count, cmd_params } => self.render_elements(
                        texture_map,
                        count,
                        cmd_params,
                        data,
                        frame_width,
                        frame_height,
                    ),
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd)
                    },
                    DrawCmd::ResetRenderState => self.pre_render(data, frame_width, frame_height),
                }
            }
        }
    }
}
