pub const INVALID_ARCHETYPE_INDEX: u32 = std::u32::MAX;
pub const ROOT_ARCHETYPE_INDEX: u32 = 1;
pub const LEAF_ARCHETYPE_INDEX: u32 = 2;
pub const STEM_ARCHETYPE_INDEX: u32 = 4;
pub const GROWING_BUD_ARCHETYPE_INDEX: u32 = 5;

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Archetype {
    pub color: [f32; 3],

    pub phototropism: f32,
}

pub struct ArchetypeTable {
    table: Vec<Archetype>,
}
