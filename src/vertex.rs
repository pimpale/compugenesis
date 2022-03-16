use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub loc: [f32; 3],
    pub color: [f32; 4],
}
vulkano::impl_vertex!(Vertex, loc, color);
