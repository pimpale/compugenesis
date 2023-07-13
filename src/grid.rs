#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
use cgmath::{Matrix4, Rad, Transform, Vector3, Vector4};

use super::vertex::mVertex as Vertex;
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
    pub fn xsize(&self) -> u32 {
        self.xsize
    }

    pub fn ysize(&self) -> u32 {
        self.ysize
    }

    pub fn zsize(&self) -> u32 {
        self.zsize
    }

    pub fn new(xsize: u32, ysize: u32, zsize: u32) -> GridBuffer {
        GridBuffer {
            xsize,
            ysize,
            zsize,
            grid_cells: vec![GridCell::new(); (xsize * ysize * zsize) as usize],
        }
    }

    fn toId(&self, x: u32, y: u32, z: u32) -> usize {
        (self.ysize * self.xsize * z + self.xsize * y + x) as usize
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> GridCell {
        self.grid_cells[self.toId(x, y, z)].clone()
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, cell: GridCell) {
        let id = self.toId(x, y, z);
        self.grid_cells[id] = cell.clone();
    }

    fn gen_vertex_cell(&self, x: u32, y: u32, z: u32) -> Vec<Vertex> {
        if self.get(x, y, z).typeCode != GRIDCELL_TYPE_SOIL {
            return vec![];
        }

        let lbu = Vertex {
            loc: [x as f32, y as f32, z as f32],
            color: [0.5, 0.9, 0.5, 1.0],
        };
        let rbu = Vertex {
            loc: [(x + 1) as f32, y as f32, z as f32],
            color: [0.5, 0.5, 0.9, 1.0],
        };
        let lfu = Vertex {
            loc: [x as f32, y as f32, (z + 1) as f32],
            color: [0.9, 0.5, 0.5, 1.0],
        };
        let rfu = Vertex {
            loc: [(x + 1) as f32, y as f32, (z + 1) as f32],
            color: [0.5, 0.9, 0.5, 1.0],
        };
        let lbl = Vertex {
            loc: [x as f32, (y + 1) as f32, z as f32],
            color: [0.5, 0.5, 0.9, 1.0],
        };
        let rbl = Vertex {
            loc: [(x + 1) as f32, (y + 1) as f32, z as f32],
            color: [0.9, 0.5, 0.5, 1.0],
        };
        let lfl = Vertex {
            loc: [x as f32, (y + 1) as f32, (z + 1) as f32],
            color: [0.5, 0.5, 0.5, 1.0],
        };
        let rfl = Vertex {
            loc: [(x + 1) as f32, (y + 1) as f32, (z + 1) as f32],
            color: [0.5, 0.5, 0.5, 1.0],
        };

        vec![
            lbu, rbu, lfu, lfu, rfu, rbu, // upper square
            lbl, rbl, lfl, lfl, rfl, rbl, // lower square
            lfu, rfu, lfl, lfl, rfl, rfu, // front square
            lbu, rbu, lbl, lbl, rbl, rbu, // back square
            lbu, lfu, lbl, lbl, lfl, lfu, // left square
            rbu, rfu, rbl, rbl, rfl, rfu, // right square
        ]
    }

    pub fn gen_vertex(&self) -> Vec<Vertex> {
        let mut vertex_list: Vec<Vertex> = Vec::new();
        for x in 0..self.xsize {
            for y in 0..self.ysize {
                for z in 0..self.zsize {
                    vertex_list.append(&mut self.gen_vertex_cell(x, y, z));
                }
            }
        }
        vertex_list
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GridCell {
    pub typeCode: u32,
    pub temperature: u32,
    pub moisture: u32,
    pub sunlight: u32,
    pub gravity: u32,
    pub plantDensity: u32,
}

impl GridCell {
    pub fn new() -> GridCell {
        GridCell {
            typeCode: GRIDCELL_TYPE_INVALID_MATERIAL,
            temperature: 0,
            moisture: 0,
            sunlight: 0,
            gravity: 0,
            plantDensity: 0,
        }
    }

}
