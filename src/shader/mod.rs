//Shader modules for rendering the data
pub mod frag;
pub mod vert;

pub const GRIDCELL_TYPE_INVALID_MATERIAL: u32 = 0;
pub const GRIDCELL_TYPE_AIR: u32 = 1;
pub const GRIDCELL_TYPE_WATER: u32 = 2;
pub const GRIDCELL_TYPE_STONE: u32 = 3;
pub const GRIDCELL_TYPE_SOIL: u32 = 4;

//Compute shader modules
pub mod gridupdategrid;
pub mod gridupdatenode;
pub mod nodeupdategrid;
