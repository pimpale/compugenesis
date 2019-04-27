pub const INVALID_ARCHETYPE_INDEX: u32 = std::u32::MAX;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Archetype {
    pub color: [f32; 3],
    pub phototropism: f32,
}

pub struct ArchetypeTable {
    table: Vec<Archetype>,
}
