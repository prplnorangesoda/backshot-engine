//! Exports [`Shader`] and [`CompiledShader`].
use std::{ffi::CString, ptr::null};

/// An uncompiled OpenGL shader.
/// Contains the source code necessary to compile it.
pub struct Shader {
    /// GL ID for this shader.
    inner: gl::types::GLuint,
    /// The source code for this shader.
    source: CString,
    /// Was this shader
    was_compiled: bool,
}

/// Represents the type of a shader object.
pub enum ShaderType {
    /// This shader is a Fragment shader.
    Fragment,
    /// This shader is a Geometry shader.
    Geometry,
    /// This shader is a Vertex shader.
    Vertex,
}

impl Shader {
    /// Wrap shader source code into a type-safe Rust struct.
    pub fn new(shader_type: ShaderType, source: impl Into<CString>) -> Self {
        let shader = unsafe {
            match shader_type {
                ShaderType::Fragment => gl::CreateShader(gl::FRAGMENT_SHADER),
                ShaderType::Geometry => gl::CreateShader(gl::GEOMETRY_SHADER),
                ShaderType::Vertex => gl::CreateShader(gl::VERTEX_SHADER),
            }
        };
        Self {
            inner: shader,
            source: source.into(),
            was_compiled: false,
        }
    }
    /// Helper function for `Shader::new()` with vertex shaders.
    pub fn vertex(source: CString) -> Self {
        Self::new(ShaderType::Vertex, source)
    }
    /// Helper function for `Shader::new()` with fragment shaders.
    pub fn fragment(source: CString) -> Self {
        Self::new(ShaderType::Fragment, source)
    }
    /// Helper function for `Shader::new()` with geometry shaders.
    pub fn geometry(source: CString) -> Self {
        Self::new(ShaderType::Geometry, source)
    }

    /// Compile this shader.
    ///
    /// Returns a [`CompiledShader`], for use in [`Program`](super::Program)s.
    ///
    /// # Errors
    /// Errors if compilation was unsuccessful, with the response from OpenGL.
    pub fn compile(mut self) -> Result<CompiledShader, String> {
        let compiled_shader = unsafe {
            gl::ShaderSource(self.inner, 1, &self.source.as_ptr(), null());
            gl::CompileShader(self.inner);

            let mut success = 0;
            gl::GetShaderiv(self.inner, gl::COMPILE_STATUS, &mut success);

            if success != gl::TRUE.into() {
                let mut infolog: Vec<u8> = vec![0; 512];
                let mut length = 0;
                gl::GetShaderInfoLog(self.inner, 512, &mut length, infolog.as_mut_ptr().cast());
                infolog.truncate(length.try_into().unwrap());
                let loggable_string = CString::new(infolog).unwrap().into_string().unwrap();
                return Err(format!("Shader compilation error: {}", loggable_string));
            }
            self.was_compiled = true;
            self.inner
        };
        // Safety: we check for
        unsafe { Ok(CompiledShader::new_unchecked(compiled_shader)) }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            if !self.was_compiled {
                gl::DeleteShader(self.inner);
            }
        }
    }
}
impl Drop for CompiledShader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        };
    }
}
/// A compiled shader object.
/// This can be linked and used in [`Program`](super::Program)s.
pub struct CompiledShader {
    /// GL ID for this compiled shader.
    id: gl::types::GLuint,
}

impl CompiledShader {
    /// Create a new CompiledShader from the GL ID of a compiled shader.
    ///
    /// # Errors
    /// Errors if `shader` is not the index of a valid compiled shader in the OpenGL context.
    pub fn new(_shader: gl::types::GLuint) -> Result<Self, ()> {
        unimplemented!()
    }

    /// Alias for [`CompiledShader::new`].
    pub fn from_opengl_uint(uint: gl::types::GLuint) -> Result<Self, ()> {
        Self::new(uint)
    }

    /// Create a new CompiledShader from the GL ID of a compiled shader.
    /// For a safe version, see [`CompiledShader::new`].
    ///
    /// # Safety
    /// The uint passed into this function MUST be a uint returned by `gl::CompileShader`.
    /// Otherwise, using this struct is undefined behaviour!
    pub unsafe fn new_unchecked(shader: gl::types::GLuint) -> Self {
        Self { id: shader }
    }
    /// Get the internal GL ID of this shader.
    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}
