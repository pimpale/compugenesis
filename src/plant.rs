#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
use cgmath::{Deg, InnerSpace, Matrix4, Rad, Transform, Vector3, Vector4};

use super::archetype::*;
use super::serde::{Deserialize, Serialize};
use super::shader::header;
use super::shader::header::ty;
use super::vertex::Vertex;
use super::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use super::vulkano::device::Device;
use std::sync::Arc;

pub const INVALID_INDEX: u32 = std::u32::MAX;

pub const STATUS_GARBAGE: u32 = 0; //Default For Plant, signifies that the plant is not instantiated
pub const STATUS_DEAD: u32 = 1; //Plant was once alive, but not anymre. It is susceptible to rot
pub const STATUS_ALIVE: u32 = 2; //Plant is currently alive, and could become dead
pub const STATUS_NEVER_ALIVE: u32 = 3; //Plant is not alive, and cannot die

#[derive(Clone, Serialize, Deserialize)]
pub struct PlantBuffer {
    plant_list: Vec<Plant>,
    free_stack: Vec<u32>,
    free_ptr: u32,
    max_size: u32,
}

fn perpendicular_vector(vec: Vector3<f32>) -> Vector3<f32> {
    // The cross product with itself will be zero, so we have 2 options
    if vec == Vector3::unit_x() {
        vec.cross(Vector3::unit_y()).normalize()
    } else {
        vec.cross(Vector3::unit_x()).normalize()
    }
}

fn cylgen(
    source_loc: Vector3<f32>,
    end_loc: Vector3<f32>,
    radius: f32,
    color1: [f32; 3],
    color2: [f32; 3],
) -> Vec<Vertex> {
    let vec = end_loc - source_loc;
    let mut hexvec = perpendicular_vector(vec) * radius;
    let mut face1: Vec<Vector3<f32>> = Vec::new();
    let mut face2: Vec<Vector3<f32>> = Vec::new();
    for i in 0..6 {
        face1.push(hexvec + source_loc);
        face2.push(hexvec + end_loc);
        hexvec =
            (Matrix4::from_axis_angle(vec.normalize(), Deg(60.0)) * hexvec.extend(1.0)).truncate();
    }

    let mut vertex_list: Vec<Vertex> = Vec::with_capacity(18);
    for i in 0..6 {
        vertex_list.push(Vertex {
            loc: face1[i].into(),
            color: color1,
        });
        //push next in line
        vertex_list.push(Vertex {
            loc: face1[(i + 1) % 6].into(),
            color: color1,
        });
        //push one from top
        vertex_list.push(Vertex {
            loc: face2[i].into(),
            color: color2,
        });
    }
    for i in 0..6 {
        vertex_list.push(Vertex {
            loc: face2[i].into(),
            color: color2,
        });
        //push next in line
        vertex_list.push(Vertex {
            loc: face2[(i + 1) % 6].into(),
            color: color2,
        });
        //push one from top
        vertex_list.push(Vertex {
            loc: face1[(i + 1) % 6].into(),
            color: color1,
        });
    }
    vertex_list
}

/// Returns the delta logistic growth
fn logisticDelta(current: f32, max: f32, scale: f32) -> f32 {
    current * (max - current) * scale
}

fn leafgen(
    source_loc: Vector3<f32>,
    end_loc: Vector3<f32>,
    up: Vector3<f32>,
    width: f32,
    color1: [f32; 3],
    color2: [f32; 3],
) -> Vec<Vertex> {
    let mut vertex_list: Vec<Vertex> = Vec::new();

    // The vector between the source and end
    let vec = end_loc - source_loc;
    // This is the horizontal part of the leaf
    let perpvec = vec.cross(up).normalize() * (width / 2.0);

    let point1 = source_loc - perpvec;
    let point2 = source_loc + perpvec;
    let point3 = end_loc - perpvec;
    let point4 = end_loc + perpvec;

    //push first triangle
    vertex_list.push(Vertex {
        loc: point1.into(),
        color: color1,
    });
    vertex_list.push(Vertex {
        loc: point2.into(),
        color: color1,
    });
    vertex_list.push(Vertex {
        loc: point3.into(),
        color: color2,
    });

    //push second triangle
    vertex_list.push(Vertex {
        loc: point3.into(),
        color: color2,
    });
    vertex_list.push(Vertex {
        loc: point4.into(),
        color: color2,
    });
    vertex_list.push(Vertex {
        loc: point2.into(),
        color: color1,
    });

    vertex_list
}

impl PlantBuffer {
    pub fn gen_metadata(&self, device: Arc<Device>) -> Arc<CpuAccessibleBuffer<ty::PlantMetadata>> {
        CpuAccessibleBuffer::from_data(
            device.clone(),
            BufferUsage::uniform_buffer(),
            ty::PlantMetadata {
                freePtr: self.free_ptr,
                plantDataCapacity: self.max_size,
            },
        )
        .unwrap()
    }

    pub fn gen_data(&self, device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[ty::Plant]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            self.plant_list.to_vec().drain(..).map(|n| n.gpu()),
        )
        .unwrap()
    }

    pub fn gen_freestack(&self, device: Arc<Device>) -> Arc<CpuAccessibleBuffer<[u32]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            self.free_stack.to_vec().drain(..),
        )
        .unwrap()
    }

    pub fn new(size: u32) -> PlantBuffer {
        if size == 0 || size == INVALID_INDEX {
            panic!("invalid size for plant buffer")
        }
        PlantBuffer {
            plant_list: vec![Plant::new(); size as usize], // Create list with default plants
            free_stack: (0..size).collect(),             // Create list of all free plant locations
            free_ptr: size,                              // The current pointer to the active stack
            max_size: size, // The maximum size to which the stack may grow
        }
    }

    pub fn from_gpu_buffer(
        metadata: Arc<CpuAccessibleBuffer<ty::PlantMetadata>>,
        data: Arc<CpuAccessibleBuffer<[ty::Plant]>>,
        freestack: Arc<CpuAccessibleBuffer<[u32]>>,
    ) -> PlantBuffer {
        let plant_data = data.read().unwrap();
        let plant_metadata = metadata.read().unwrap();
        let plant_freestack = freestack.read().unwrap();
        PlantBuffer {
            plant_list: plant_data.iter().map(|&n| Plant::fromgpu(n)).collect(),
            free_stack: plant_freestack.iter().cloned().collect(),
            free_ptr: plant_metadata.freePtr,
            max_size: plant_metadata.plantDataCapacity,
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

    pub fn alloc_insert(&mut self, plant: Plant) -> () {
        let index = self.alloc();
        self.set(index, plant.clone());
    }

    /// Marks an index in the array as free to use, marks any plant as garbage
    pub fn free(&mut self, index: u32) -> () {
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

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Plant {
    pub leftChildIndex: u32,
    pub rightChildIndex: u32,
    pub parentIndex: u32,
    pub age: u32,
    pub archetypeId: u32,
    pub plantId: u32,
    pub status: u32,
    pub visible: u32,
    pub length: f32,
    pub radius: f32, // also can be width
    pub volume: f32,
    pub transformation: [[f32; 4]; 4],
}

impl Plant {
    // Plant has some dummy variables and this function makes it easier to create a default instance
    pub fn new() -> Plant {
        Plant {
            leftChildIndex: INVALID_INDEX,
            rightChildIndex: INVALID_INDEX,
            parentIndex: INVALID_INDEX,
            age: 0,
            archetypeId: INVALID_ARCHETYPE_INDEX,
            status: STATUS_GARBAGE,
            visible: 0,
            length: 0.0,
            radius: 0.0, // also can be width
            volume: 0.0,
            absolutePositionCache: [0.0, 0.0, 0.0],
            transformation: Matrix4::one().into(),
        }
    }

    pub fn fromgpu(plant: ty::Plant) -> Plant {
        Plant {
            leftChildIndex: plant.leftChildIndex,
            rightChildIndex: plant.rightChildIndex,
            parentIndex: plant.parentIndex,
            age: plant.age,
            archetypeId: plant.archetypeId,
            status: plant.status,
            visible: plant.visible,
            length: plant.length,
            radius: plant.radius,
            volume: plant.volume,
            absolutePositionCache: plant.absolutePositionCache,
            transformation: plant.transformation,
        }
    }

    pub fn gpu(&self) -> ty::Plant {
        ty::Plant {
            leftChildIndex: self.leftChildIndex,
            rightChildIndex: self.rightChildIndex,
            parentIndex: self.parentIndex,
            age: self.age,
            archetypeId: self.archetypeId,
            status: self.status,
            visible: self.visible,
            length: self.length,
            radius: self.radius,
            volume: self.volume,
            absolutePositionCache: self.absolutePositionCache,
            transformation: self.transformation,
            _dummy0: [0; 8],
            _dummy1: [0; 4],
        }
    }
}
