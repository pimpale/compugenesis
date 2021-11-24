#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
use cgmath::{Deg, InnerSpace, Matrix4, Rad, Transform, Vector3, Vector4};
use std::convert::TryInto;

use super::archetype::*;
use super::vertex::Vertex;
use std::sync::Arc;

pub const INVALID_INDEX: u32 = std::u32::MAX;

pub const STATUS_GARBAGE: u32 = 0; //Default For Plant, signifies that the plant is not instantiated
pub const STATUS_DEAD: u32 = 1; //Plant was once alive, but not anymre. It is susceptible to rot
pub const STATUS_ALIVE: u32 = 2; //Plant is currently alive, and could become dead
pub const STATUS_NEVER_ALIVE: u32 = 3; //Plant is not alive, and cannot die

#[derive(Clone)]
pub struct PlantBuffer {
    plant_list: Vec<Plant>,
    free_stack: Vec<u32>,
    free_ptr: u32,
    max_size: u32,
}

impl PlantBuffer {
    pub fn new(size: u32) -> PlantBuffer {
        if size == 0 || size == INVALID_INDEX {
            panic!("invalid size for plant buffer")
        }
        PlantBuffer {
            plant_list: vec![Plant::new(); size as usize], // Create list with default plants
            free_stack: (0..size).collect(), // Create list of all free plant locations
            free_ptr: size,                  // The current pointer to the active stack
            max_size: size,                  // The maximum size to which the stack may grow
        }
    }

    pub fn get(&self, index: u32) -> Plant {
        self.plant_list[index as usize].clone()
    }

    pub fn set(&mut self, index: u32, plant: Plant) {
        self.plant_list[index as usize] = plant.clone();
    }

    /// Returns the index of a free spot in the array (user needs to mark the spot as not garbage)
    pub fn alloc(&mut self) -> u32 {
        if self.free_ptr == 0 {
            panic!("No Memory Left In PlantBuffer");
        } else {
            self.free_ptr = self.free_ptr - 1;
            self.free_stack[self.free_ptr as usize]
        }
    }

    pub fn alloc_insert(&mut self, plant: Plant) {
        let index = self.alloc();
        self.set(index, plant.clone());
    }

    /// Marks an index in the array as free to use, marks any plant as garbage
    pub fn free(&mut self, index: u32) {
        self.plant_list[index as usize].status = STATUS_GARBAGE;
        if self.free_ptr == self.max_size {
            panic!("Free Stack Full (This should not happen)");
        } else {
            self.free_stack[self.free_ptr as usize] = index;
            self.free_ptr = self.free_ptr + 1;
        }
    }

    /// Rreturns maximum size that the plant list could grow to.
    pub fn size(&self) -> u32 {
        self.max_size
    }

    /// Returns the current size that the plant buffer is at
    pub fn current_size(&self) -> u32 {
        self.max_size - self.free_ptr
    }
}

fn tomat(mat: [[f32; 4]; 4]) -> Matrix4<f32> {
    Matrix4::from_cols(
        Vector4::from(mat[0]),
        Vector4::from(mat[1]),
        Vector4::from(mat[2]),
        Vector4::from(mat[3]),
    )
}

fn tov(v3: [f32; 3]) -> Vector3<f32> {
    Vector3::new(v3[0], v3[1], v3[2])
}

fn to3(v: Vector3<f32>) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn scale3(a: [f32; 3], scalar: f32) -> [f32; 3] {
    [a[0] * scalar, a[1] * scalar, a[2] * scalar]
}

fn add3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[derive(Clone, Copy, Debug)]
pub struct Plant {
    pub status: u32,
    pub age: u32,
    pub location: [f32; 3],
}

impl Plant {
    // Plant has some dummy variables and this function makes it easier to create a default instance
    pub fn new() -> Plant {
        Plant {
            status: STATUS_GARBAGE,
            age: 0,
            location: [0.0, 0.0, 0.0],
        }
    }
}
