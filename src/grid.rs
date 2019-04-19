#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
use cgmath::{Matrix4, Rad, Transform, Vector3, Vector4};

use super::shader::gridupdategrid::ty::GridCell;
use super::shader::gridupdategrid::ty::GridMetadata;
use super::vertex::Vertex;
use super::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use super::vulkano::device::Device;
use std::sync::Arc;

pub const GRIDCELL_TYPE_INVALID_MATERIAL: u32 = 0;
pub const GRIDCELL_TYPE_AIR: u32 = 1;
pub const GRIDCELL_TYPE_WATER: u32 = 2;
pub const GRIDCELL_TYPE_STONE: u32 = 3;
pub const GRIDCELL_TYPE_SOIL: u32 = 4;

#[derive(Clone)]
pub struct GridBuffer {
    grid_cells: Vec<GridCell>,
    xsize: u32,
    ysize: u32,
    zsize: u32,
}

impl GridBuffer {
    pub fn new(xsize: u32, ysize: u32, zsize: u32) -> GridBuffer {
        GridBuffer {
            xsize: xsize,
            ysize: ysize,
            zsize: zsize,
            grid_cells: vec![GridCell::new(); (xsize * ysize * zsize) as usize],
        }
    }

    fn toId(&self, x: u32, y: u32, z: u32) -> usize {
        (self.ysize * self.xsize * z + self.xsize * y + x) as usize
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> GridCell {
        self.grid_cells[toId(x, y, z)].clone()
    }

    pub fn set(&mut self, cell:GridCell, x:u32, y:u32, z:u32) -> () {
        self.grid_cells[toId(x,y,z)] = cell.clone();
    }

    pub 

}

impl GridCell {
    pub fn new() -> GridCell {
        GridCell {
            typeCode: GRIDCELL_TYPE_INVALID_MATERIAL,
            temperature: 0.0,
            moisture: 0.0,
            sunlight: 0.0,
            gravity: 0.0,
            plantDensity: 0.0,
        }
    }
}
