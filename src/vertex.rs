use crate::vector3::Vector3_32;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub pos: Vector3_32,
}

#[derive(Clone, Debug)]
pub struct TriangleMesh(pub Vertex, pub Vertex, pub Vertex);
