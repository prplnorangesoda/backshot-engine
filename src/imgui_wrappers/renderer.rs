// THIS FILE INCLUDES CODE ADAPTED FROM THIRDPARTY CODE!
// The LICENSE is included below.
// https://github.com/imgui-rs/imgui-glow-renderer
// Copyright (c) 2021 The imgui-rs Developers
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//

use std::{borrow::Cow, error::Error, ffi::CString, fmt::Display, mem::offset_of, str::FromStr};

use imgui::{
    Context, DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert, FontAtlas, TextureId, Textures,
    internal::RawWrapper,
};

use crate::{
    construct_program,
    gl_wrappers::{program::Program, shader::Shader},
    vector3::to_byte_slice,
    vertex,
};

pub struct ImguiRenderer {
    imgui_texture_map: Textures<gl::types::GLuint>,
    font_atlas_texture: gl::types::GLuint,
    shaders: Shaders,
    vbo_handle: gl::types::GLuint,
    ebo_handle: gl::types::GLuint,
}

/// Trait for mapping imgui texture IDs to OpenGL textures.
///
/// [`register`] should be called after uploading a texture to OpenGL to get a
/// [`TextureId`] corresponding to it.
///
/// [`register`]: Self::register
///
/// Then [`gl_texture`] can be called to find the OpenGL texture corresponding to
/// that [`TextureId`].
///
/// [`gl_texture`]: Self::gl_texture
pub trait TextureMap {
    fn register(&mut self, gl_texture: gl::types::GLuint) -> Option<TextureId>;

    fn gl_texture(&self, imgui_texture: TextureId) -> Option<gl::types::GLuint>;
}

/// Texture map where the imgui texture ID is simply numerically equal to the
/// OpenGL texture ID.
#[derive(Default)]
pub struct SimpleTextureMap();

impl TextureMap for SimpleTextureMap {
    #[inline(always)]
    fn register(&mut self, gl_texture: gl::types::GLuint) -> Option<TextureId> {
        Some(TextureId::new(gl_texture as _))
    }

    #[inline(always)]
    fn gl_texture(&self, imgui_texture: TextureId) -> Option<gl::types::GLuint> {
        #[allow(clippy::cast_possible_truncation)]
        Some(imgui_texture.id() as _)
    }
}

/// [`Textures`] is a simple choice for a texture map.
impl TextureMap for Textures<gl::types::GLuint> {
    fn register(&mut self, gl_texture: gl::types::GLuint) -> Option<TextureId> {
        Some(self.insert(gl_texture))
    }

    fn gl_texture(&self, imgui_texture: TextureId) -> Option<gl::types::GLuint> {
        self.get(imgui_texture).copied()
    }
}

const fn imgui_index_type_as_gl() -> u32 {
    match size_of::<DrawIdx>() {
        1 => gl::UNSIGNED_BYTE,
        2 => gl::UNSIGNED_SHORT,
        _ => gl::UNSIGNED_INT,
    }
}
impl ImguiRenderer {
    pub fn new(imgui_context: &mut Context) -> Self {
        let mut imgui_texture_map = Textures::new();

        let font_atlas_texture =
            prepare_font_atlas(imgui_context.fonts(), &mut imgui_texture_map).unwrap();
        let shaders = Shaders::new(true).unwrap();
        let vbo_handle = unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            if id == 0 {
                Err(String::from("Failed to create buffer object"))
            } else {
                Ok(id)
            }
        }
        .unwrap();
        let ebo_handle = unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            if id == 0 {
                Err(String::from("Failed to create buffer object"))
            } else {
                Ok(id)
            }
        }
        .unwrap();
        Self {
            shaders,
            imgui_texture_map,
            font_atlas_texture,
            vbo_handle,
            ebo_handle,
        }
    }
    pub fn pre_render(&mut self, data: &DrawData, frame_width: f32, frame_height: f32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Enable(gl::BLEND);
            gl::BlendEquation(gl::FUNC_ADD);
            gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );
            gl::Disable(gl::CULL_FACE);
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::STENCIL_TEST);
            gl::Enable(gl::SCISSOR_TEST);

            gl::Disable(gl::PRIMITIVE_RESTART);
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL);

            gl::Viewport(0, 0, frame_width as _, frame_height as _);
        }

        let clip_origin_is_lower_left = unsafe {
            let mut value = 0;
            gl::GetIntegerv(gl::CLIP_ORIGIN, &mut value);
            value != gl::UPPER_LEFT as i32
        };
        let projection_matrix = calculate_matrix(data, clip_origin_is_lower_left);

        unsafe {
            gl::UseProgram(self.shaders.program.id());
            gl::Uniform1i(self.shaders.texture_uniform_location, 0);
            gl::UniformMatrix4fv(
                self.shaders.matrix_uniform_location,
                1,
                gl::FALSE,
                projection_matrix.as_ptr(),
            );
        }

        unsafe { gl::BindSampler(0, 0) }

        const POSITION_FIELD_OFFSET: gl::types::GLuint = offset_of!(DrawVert, pos) as _;
        const UV_FIELD_OFFSET: gl::types::GLuint = offset_of!(DrawVert, uv) as _;
        const COLOR_FIELD_OFFSET: gl::types::GLuint = offset_of!(DrawVert, col) as _;

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo_handle);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo_handle);
            gl::EnableVertexAttribArray(self.shaders.position_attribute_index);
            gl::VertexAttribPointer(
                self.shaders.position_attribute_index,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawVert>() as _,
                POSITION_FIELD_OFFSET as *const _,
            );
            gl::EnableVertexAttribArray(self.shaders.uv_attribute_index);
            gl::VertexAttribPointer(
                self.shaders.uv_attribute_index,
                2,
                gl::FLOAT,
                gl::FALSE,
                size_of::<DrawVert>() as _,
                UV_FIELD_OFFSET as *const _,
            );
            gl::EnableVertexAttribArray(self.shaders.color_attribute_index);
            gl::VertexAttribPointer(
                self.shaders.color_attribute_index,
                4,
                gl::UNSIGNED_BYTE,
                gl::TRUE,
                size_of::<DrawVert>() as _,
                COLOR_FIELD_OFFSET as *const _,
            );
        }
    }

    pub fn render(&mut self, data: &DrawData) {
        let frame_width = data.display_size[0] * data.framebuffer_scale[0];
        let frame_height = data.display_size[1] * data.framebuffer_scale[1];
        if frame_width <= 0.0 || frame_height <= 0.0 {
            return;
        }
        let mut vertex_array_object = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut vertex_array_object);
            gl::BindVertexArray(vertex_array_object);
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
                    DrawCmd::Elements { count, cmd_params } => {
                        self.render_elements(count, cmd_params, data, frame_width, frame_height)
                    }
                    DrawCmd::RawCallback { callback, raw_cmd } => unsafe {
                        callback(draw_list.raw(), raw_cmd)
                    },
                    DrawCmd::ResetRenderState => self.pre_render(data, frame_width, frame_height),
                }
            }
        }
        unsafe {
            gl::DeleteVertexArrays(1, &vertex_array_object);
        }
        self.post_render(data, frame_width, frame_height)
    }

    fn post_render(&mut self, data: &DrawData, frame_width: f32, frame_height: f32) {
        unsafe {
            gl::Disable(gl::SCISSOR_TEST);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn render_elements(
        &self,
        element_count: usize,
        element_params: DrawCmdParams,
        draw_data: &DrawData,
        fb_width: f32,
        fb_height: f32,
    ) {
        #![allow(clippy::similar_names)]

        let DrawCmdParams {
            clip_rect,
            texture_id,
            vtx_offset,
            idx_offset,
        } = element_params;
        let clip_off = draw_data.display_pos;
        let scale = draw_data.framebuffer_scale;

        let clip_x1 = (clip_rect[0] - clip_off[0]) * scale[0];
        let clip_y1 = (clip_rect[1] - clip_off[1]) * scale[1];
        let clip_x2 = (clip_rect[2] - clip_off[0]) * scale[0];
        let clip_y2 = (clip_rect[3] - clip_off[1]) * scale[1];

        if clip_x1 >= fb_width || clip_y1 >= fb_height || clip_x2 < 0.0 || clip_y2 < 0.0 {
            return;
        }

        unsafe {
            gl::Scissor(
                clip_x1 as i32,
                (fb_height - clip_y2) as i32,
                (clip_x2 - clip_x1) as i32,
                (clip_y2 - clip_y1) as i32,
            );

            gl::BindTexture(
                gl::TEXTURE_2D,
                self.imgui_texture_map.gl_texture(texture_id).unwrap(),
            );

            gl::DrawElementsBaseVertex(
                gl::TRIANGLES,
                element_count as _,
                imgui_index_type_as_gl(),
                (idx_offset * size_of::<DrawIdx>()) as _,
                vtx_offset as _,
            );
        }
    }

    fn configure_imgui_context(&self, imgui_context: &mut Context) {
        imgui_context.set_renderer_name(Some(format!(
            "backshot-engine {}",
            env!("CARGO_PKG_VERSION")
        )));
        imgui_context
            .io_mut()
            .backend_flags
            .insert(imgui::BackendFlags::RENDERER_HAS_VTX_OFFSET);
    }
}

struct Shaders {
    program: Program,
    texture_uniform_location: gl::types::GLint,
    matrix_uniform_location: gl::types::GLint,
    position_attribute_index: gl::types::GLuint,
    uv_attribute_index: gl::types::GLuint,
    color_attribute_index: gl::types::GLuint,
}

impl Shaders {
    fn new(output_srgb: bool) -> Result<Self, ShaderError> {
        let (vertex_source, fragment_source) = Self::get_shader_sources(output_srgb)?;

        let vertex_source = CString::from_str(&vertex_source).unwrap();
        let fragment_source = CString::from_str(&fragment_source).unwrap();

        let vertex_shader = Shader::vertex(vertex_source)
            .compile()
            .map_err(ShaderError::CreateShader)?;

        let fragment_shader = Shader::fragment(fragment_source)
            .compile()
            .map_err(ShaderError::CreateShader)?;

        let program = construct_program!(vertex_shader, fragment_shader;)
            .map_err(ShaderError::CreateProgram)?;

        Ok(Self {
            texture_uniform_location: program
                .get_uniform_location("tex")
                .ok_or_else(|| ShaderError::UniformNotFound("tex".into()))?,
            matrix_uniform_location: program
                .get_uniform_location("matrix")
                .ok_or_else(|| ShaderError::UniformNotFound("matrix".into()))?,
            position_attribute_index: program
                .get_attrib_location("position")
                .ok_or_else(|| ShaderError::AttributeNotFound("position".into()))?
                as _,
            uv_attribute_index: program
                .get_attrib_location("uv")
                .ok_or_else(|| ShaderError::AttributeNotFound("uv".into()))?
                as _,
            color_attribute_index: program
                .get_attrib_location("color")
                .ok_or_else(|| ShaderError::AttributeNotFound("color".into()))?
                as _,
            program,
        })
    }

    fn get_shader_sources(output_srgb: bool) -> Result<(String, String), ShaderError> {
        const VERTEX_BODY: &str = include_str!("../../glsl/imgui/vert.glsl");
        const FRAGMENT_BODY: &str = include_str!("../../glsl/imgui/frag.glsl");

        let vertex_source = String::from(VERTEX_BODY);
        let fragment_source = format!(
            "#version 430 core\n{defines}\n{body}",
            defines = if output_srgb {
                "\n#define OUTPUT_SRGB"
            } else {
                ""
            },
            body = FRAGMENT_BODY,
        );

        Ok((vertex_source, fragment_source))
    }
}

#[derive(Debug)]
pub enum ShaderError {
    IncompatibleVersion(String),
    CreateShader(String),
    CreateProgram(String),
    CompileShader(String),
    LinkProgram(String),
    UniformNotFound(Cow<'static, str>),
    AttributeNotFound(Cow<'static, str>),
}

impl Error for ShaderError {}

impl Display for ShaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IncompatibleVersion(msg) => write!(
                f,
                "Shader not compatible with OpenGL version found in the context: {}",
                msg
            ),
            Self::CreateShader(msg) => write!(f, "Error creating shader object: {}", msg),
            Self::CreateProgram(msg) => write!(f, "Error creating program object: {}", msg),
            Self::CompileShader(msg) => write!(f, "Error compiling shader: {}", msg),
            Self::LinkProgram(msg) => write!(f, "Error linking shader program: {}", msg),
            Self::UniformNotFound(uniform_name) => {
                write!(f, "Uniform `{}` not found in shader program", uniform_name)
            }
            Self::AttributeNotFound(attribute_name) => {
                write!(
                    f,
                    "Attribute `{}` not found in shader program",
                    attribute_name
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum InitError {
    Shader(ShaderError),
    CreateBufferObject(String),
    CreateTexture(String),
    RegisterTexture,
    UserError(String),
}

fn prepare_font_atlas<T: TextureMap>(
    fonts: &mut FontAtlas,
    texture_map: &mut T,
) -> Result<gl::types::GLuint, InitError> {
    #![allow(clippy::cast_possible_wrap)]

    let atlas_texture = fonts.build_rgba32_texture();

    let gl_texture = unsafe {
        // new: put the genned texture in a maybeuninit
        let mut tex_id: gl::types::GLuint = 0;
        gl::GenTextures(1, &mut tex_id);
        if tex_id == 0 {
            Err(String::from("Unable to create Texture object"))
        } else {
            Ok(tex_id)
        }
    }
    .map_err(InitError::CreateTexture)?;

    unsafe {
        gl::BindTexture(gl::TEXTURE_2D, gl_texture);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as _);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as _);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            gl::SRGB8_ALPHA8 as _,
            atlas_texture.width as _,
            atlas_texture.height as _,
            0,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            atlas_texture.data.as_ptr().cast(),
        );
    }

    fonts.tex_id = texture_map
        .register(gl_texture)
        .ok_or(InitError::RegisterTexture)?;

    Ok(gl_texture)
}

#[allow(clippy::deprecated_cfg_attr)]
fn calculate_matrix(draw_data: &DrawData, clip_origin_is_lower_left: bool) -> [f32; 16] {
    let left = draw_data.display_pos[0];
    let right = draw_data.display_pos[0] + draw_data.display_size[0];
    let top = draw_data.display_pos[1];
    let bottom = draw_data.display_pos[1] + draw_data.display_size[1];

    let (top, bottom) = if clip_origin_is_lower_left {
        (top, bottom)
    } else {
        (bottom, top)
    };

    #[cfg_attr(rustfmt, rustfmt::skip)]
    {
        [
        2.0 / (right - left)           , 0.0                            , 0.0 , 0.0,
        0.0                            , (2.0 / (top - bottom))         , 0.0 , 0.0,
        0.0                            , 0.0                            , -1.0, 0.0,
        (right + left) / (left - right), (top + bottom) / (bottom - top), 0.0 , 1.0,
        ]
    }
}
