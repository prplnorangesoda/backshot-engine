pub struct Vertex {
    /// East/West.
    pub x: f64,

    /// North/South.
    pub y: f64,

    /// Up/Down.
    pub z: f64,
}

pub struct TriangleMesh(Vertex, Vertex, Vertex);
