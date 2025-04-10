#[derive(Clone, Debug)]
pub struct Vertex {
    pub pos: glm::Vec3,
}

#[derive(Clone, Debug)]
pub struct TriangleMesh(pub Vertex, pub Vertex, pub Vertex);
