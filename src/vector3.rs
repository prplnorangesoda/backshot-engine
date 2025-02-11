use crate::render_vec::{GlLayout, GlType};

#[derive(Default, Debug, Clone, Copy)]
pub struct Vector3_32 {
    /// East/West.
    pub x: f32,

    /// Up/Down.
    pub y: f32,

    /// North/South.
    pub z: f32,
}

impl Vector3_32 {
    pub fn xyz(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

impl From<[f32; 3]> for Vector3_32 {
    fn from(value: [f32; 3]) -> Self {
        Self::xyz(value[0], value[1], value[2])
    }
}

fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4) }
}

unsafe impl GlLayout for Vector3_32 {
    fn as_gl_bytes(&self) -> Box<[u8]> {
        let out = {
            let slice = [self.x, self.y, self.z];

            let res = to_byte_slice(&slice);
            let mut out = Vec::new();
            out.extend_from_slice(res);
            out.into_boxed_slice()
        };
        out
    }
    fn gl_type_layout() -> Box<[GlType]> {
        Box::new([GlType::Float, GlType::Float, GlType::Float])
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Vector3_64 {
    /// East/West.
    pub x: f64,

    /// Up/Down.
    pub y: f64,

    /// North/South.
    pub z: f64,
}

impl Vector3_64 {
    pub fn xyz(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl From<[f64; 3]> for Vector3_64 {
    fn from(value: [f64; 3]) -> Self {
        Self::xyz(value[0], value[1], value[2])
    }
}
