#[derive(Clone, Debug, Copy, Default)]
pub struct Vertex {
    pub loc: [f32; 3],
    pub color: [f32; 4],
}
impl_vertex!(Vertex, loc, color);
