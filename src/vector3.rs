#[derive(Default, Debug, Clone, Copy)]
pub struct Vector3 {
    /// East/West.
    pub x: f64,

    /// Up/Down.
    pub y: f64,

    /// North/South.
    pub z: f64,
}

impl Vector3 {
    pub fn xyz(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}
