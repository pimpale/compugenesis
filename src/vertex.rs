use vulkano::{buffer::BufferContents, pipeline::graphics::vertex_input::Vertex};

#[repr(C)]
#[derive(BufferContents, Vertex, Clone, Copy)]
pub struct mVertex {
    #[format(R32G32B32_SFLOAT)]
    pub loc: [f32; 3],

    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
}
