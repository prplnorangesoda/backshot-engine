use std::{any::type_name, ops::Deref, ptr::slice_from_raw_parts};

use crate::render_vec::{GlType, GlTypeList, StaticGlLayout};

// #[derive(Default, Debug, Clone, Copy)]
// pub struct Vector3_32 {
//     /// East/West.
//     pub x: f32,

//     /// Up/Down.
//     pub y: f32,

//     /// North/South.
//     pub z: f32,
// }

// impl Vector3_32 {
//     pub fn xyz(x: f32, y: f32, z: f32) -> Self {
//         Self { x, y, z }
//     }
//}

// impl From<[f32; 3]> for Vector3_32 {
//     fn from(value: [f32; 3]) -> Self {
//         Self::xyz(value[0], value[1], value[2])
//     }
// }

/// Remap a slice of T to a slice of bytes.
pub fn to_byte_slice<T>(slice: &[T]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast(), std::mem::size_of_val(slice)) }
}

/// Remap a slice of bytes to a slice of T.
///
/// # Safety
/// This function is only safe if used with bytes from [`to_byte_slice`].
/// Any other byte slice is undefined behaviour.
/// # Panics
/// Panics if `T` is not layout compatible with the bytes.
pub unsafe fn from_byte_slice<T>(bytes: &[u8]) -> &[T] {
    let bytes_stride_t = size_of::<T>();
    if !bytes.len().is_multiple_of(bytes_stride_t) {
        panic!(
            "Cannot make slice of {} safely from slice of bytes",
            type_name::<T>()
        );
    }
    // SAFETY: Caller guarantees this data will be valid.
    unsafe { std::slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len() / bytes_stride_t) }
}

// SAFETY:
// Vec3 is a vec of 3 Floats
unsafe impl StaticGlLayout for glm::Vec3 {
    const LEN: usize = 3;
    fn as_gl_bytes(&self) -> impl Deref<Target = [u8]> {
        // SAFETY:
        // glm::Vec3 is repr(C), meaning it's laid out in memory
        // exactly the same as an array of F32s.
        let slice: &[f32] = unsafe {
            slice_from_raw_parts((self as *const glm::Vec3).cast(), 3)
                .as_ref()
                .unwrap()
        };

        to_byte_slice(slice)
    }
    fn gl_type_layout() -> GlTypeList<3> {
        GlTypeList::new([GlType::Float, GlType::Float, GlType::Float])
    }
}

// #[derive(Default, Debug, Clone, Copy)]
// pub struct Vector3_64 {
//     /// East/West.
//     pub x: f64,

//     /// Up/Down.
//     pub y: f64,

//     /// North/South.
//     pub z: f64,
// }

// impl Vector3_64 {
//     pub fn xyz(x: f64, y: f64, z: f64) -> Self {
//         Self { x, y, z }
//     }
// }

// impl From<[f64; 3]> for Vector3_64 {
//     fn from(value: [f64; 3]) -> Self {
//         Self::xyz(value[0], value[1], value[2])
//     }
// }
