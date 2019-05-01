#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(non_snake_case)]
use cgmath::{Deg, InnerSpace, Matrix4, Rad, Transform, Vector3, Vector4};

use super::archetype::*;
use super::serde::{Deserialize, Serialize};
use super::shader::nodeupdategrid;
use super::vertex::Vertex;
use super::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use super::vulkano::device::Device;
use std::sync::Arc;

pub const INVALID_INDEX: u32 = std::u32::MAX;

pub const STATUS_GARBAGE: u32 = 0; //Default For Node, signifies that the node is not instantiated
pub const STATUS_DEAD: u32 = 1; //Node was once alive, but not anymre. It is susceptible to rot
pub const STATUS_ALIVE: u32 = 2; //Node is currently alive, and could become dead
pub const STATUS_NEVER_ALIVE: u32 = 3; //Node is not alive, and cannot die

#[derive(Clone, Serialize, Deserialize)]
pub struct NodeBuffer {
    node_list: Vec<Node>,
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

impl NodeBuffer {
    pub fn gen_metadata(
        &self,
        device: Arc<Device>,
    ) -> Arc<CpuAccessibleBuffer<nodeupdategrid::ty::NodeMetadata>> {
        CpuAccessibleBuffer::from_data(
            device.clone(),
            BufferUsage::uniform_buffer(),
            nodeupdategrid::ty::NodeMetadata {
                freePtr: self.free_ptr,
                nodeDataCapacity: self.max_size,
            },
        )
        .unwrap()
    }

    pub fn gen_data(
        &self,
        device: Arc<Device>,
    ) -> Arc<CpuAccessibleBuffer<[nodeupdategrid::ty::Node]>> {
        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            self.node_list.to_vec().drain(..).map(|n| n.gpu()),
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

    pub fn new(size: u32) -> NodeBuffer {
        if size == 0 || size == INVALID_INDEX {
            panic!("invalid size for node buffer")
        }
        NodeBuffer {
            node_list: vec![Node::new(); size as usize], // Create list with default nodes
            free_stack: (0..size).collect(),             // Create list of all free node locations
            free_ptr: size,                              // The current pointer to the active stack
            max_size: size, // The maximum size to which the stack may grow
        }
    }

    pub fn get(&self, index: u32) -> Node {
        self.node_list[index as usize].clone()
    }

    pub fn set(&mut self, index: u32, node: Node) {
        self.node_list[index as usize] = node.clone();
    }

    /// Returns the index of a free spot in the array (user needs to mark the spot as not garbage)
    pub fn alloc(&mut self) -> u32 {
        if self.free_ptr == 0 {
            panic!("No Memory Left In NodeBuffer");
        } else {
            self.free_ptr = self.free_ptr - 1;
            self.free_stack[self.free_ptr as usize]
        }
    }

    pub fn alloc_insert(&mut self, node: Node) -> () {
        let index = self.alloc();
        self.set(index, node.clone());
    }

    /// Marks an index in the array as free to use, marks any node as garbage
    pub fn free(&mut self, index: u32) -> () {
        self.node_list[index as usize].status = STATUS_GARBAGE;
        if self.free_ptr == self.max_size {
            panic!("Free Stack Full (This should not happen)");
        } else {
            self.free_stack[self.free_ptr as usize] = index;
            self.free_ptr = self.free_ptr + 1;
        }
    }

    /// Rreturns maximum size that the node list could grow to.
    pub fn size(&self) -> u32 {
        self.max_size
    }

    /// Returns the current size that the node buffer is at
    pub fn current_size(&self) -> u32 {
        self.max_size - self.free_ptr
    }

    /// Generates a list of vertexes to be rendered
    pub fn gen_vertex(&self) -> Vec<Vertex> {
        //Vector to hold all new vertexes
        let mut vertex_list = Vec::new();

        //search for root node (null parent, visible)
        for node_index in 0..self.max_size {
            let node = &self.node_list[node_index as usize];
            // If its a root node
            if node.status != STATUS_GARBAGE && node.parentIndex == INVALID_INDEX {
                // call gen_node_vertex
                vertex_list.append(&mut self.gen_node_vertex(
                    tov(node.absolutePositionCache),
                    Matrix4::one(),
                    node_index,
                ));
            }
        }
        vertex_list
    }
    /// Internal recursive algorithm that traverses tree structure
    fn gen_node_vertex(
        &self,
        source_loc: Vector3<f32>,
        parent_rotation: Matrix4<f32>,
        node_index: u32,
    ) -> Vec<Vertex> {
        let mut vertex_list = Vec::new();
        let node = self.node_list[node_index as usize];
        // The rotation of this node
        let total_rotation = parent_rotation * tomat(node.transformation);
        // The endpoint in space where this node ends
        let end_loc = source_loc + total_rotation.transform_vector(Vector3::unit_y() * node.length);
        if node.visible == 1 {
            if node.archetypeId == LEAF_ARCHETYPE_INDEX {
                vertex_list.append(&mut leafgen(
                    source_loc,
                    end_loc,
                    Vector3::unit_y(),
                    node.radius,
                    [0.0, 1.0, 0.0],
                    [1.0, 1.0, 0.0],
                ));
            } else {
                vertex_list.append(&mut cylgen(
                    source_loc,
                    end_loc,
                    node.radius,
                    [0.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ));
            }
        }
        // Get child vertex_list, and add it to our own
        if node.leftChildIndex != INVALID_INDEX {
            vertex_list.append(&mut self.gen_node_vertex(
                end_loc,
                total_rotation,
                node.leftChildIndex,
            ));
        }
        // Do the same for the right node
        if node.rightChildIndex != INVALID_INDEX {
            vertex_list.append(&mut self.gen_node_vertex(
                end_loc,
                total_rotation,
                node.rightChildIndex,
            ));
        }
        // Return the list
        vertex_list
    }

    /// Sets the left child of parent to child, and if child is not invalid, sets its parent to the parent
    pub fn set_left_child(&mut self, parent: u32, child: u32) -> () {
        self.node_list[parent as usize].leftChildIndex = child;
        if child != INVALID_INDEX {
            self.node_list[child as usize].parentIndex = parent;
        }
    }

    /// Sets the right child of parent to child, and if child is not invalid, sets its parent to the parent
    pub fn set_right_child(&mut self, parent: u32, child: u32) -> () {
        self.node_list[parent as usize].rightChildIndex = child;
        if child != INVALID_INDEX {
            self.node_list[child as usize].parentIndex = parent;
        }
    }

    /// Divides node segment in two, allocating a new node for the upper half, with the current
    /// node as a parent. The children are transferred to the new node. All properties of the old
    /// node are maintained, except for children. The left child is the new node, and the right one
    /// is left empty
    /// returns the index of the newly created node
    pub fn divide(&mut self, percentbreak: f32, node_index: u32) -> u32 {
        // Allocate a spot for the new node
        let new_node_index = self.alloc();

        // New node shares all properties with old one
        self.node_list[new_node_index as usize] = self.node_list[node_index as usize].clone();
        // Set lengths so they add up to same amount TODO ensure percentbreak is less than one
        let origlength = self.node_list[node_index as usize].length;
        self.node_list[node_index as usize].length = percentbreak * origlength;
        self.node_list[new_node_index as usize].length = (1.0 - percentbreak) * origlength;

        //Remove any transformation from the new node
        self.node_list[new_node_index as usize].transformation = cgmath::Matrix4::one().into();

        //Join the two children of the current node on as children of the new node
        self.set_left_child(
            new_node_index,
            self.node_list[node_index as usize].leftChildIndex,
        );
        self.set_right_child(
            new_node_index,
            self.node_list[node_index as usize].rightChildIndex,
        );

        // Join the new node as the left child of the current one, and emptying the right one
        self.set_left_child(node_index, new_node_index);
        self.set_right_child(node_index, INVALID_INDEX);
        //Return the index of the new node created
        new_node_index
    }

    /// Does a nodeupdatenode on all nodes within the buffer that are not garbage
    pub fn update_all(&mut self) {
        for ni in 0..self.max_size {
            let mut node = self.node_list[ni as usize];
            if node.status != STATUS_GARBAGE {
                self.node_list[ni as usize].age += 1;

                match node.archetypeId {
                    INVALID_ARCHETYPE_INDEX => (),
                    GROWING_BUD_ARCHETYPE_INDEX => {
                        if rand::random::<f32>() > 0.999 && node.age < 5000 {
                            let leftchildindex = self.alloc();
                            self.node_list[leftchildindex as usize] = node.clone();
                            self.node_list[leftchildindex as usize].transformation =
                                Matrix4::one().into();
                            node.archetypeId = STEM_ARCHETYPE_INDEX;
                            node.length = 0.001;
                            self.node_list[ni as usize] = node;
                            self.set_left_child(ni, leftchildindex);
                            if rand::random::<f32>() > 0.3 {
                                let rightchildindex = self.alloc();
                                // Create new node for leaf
                                let mut leafnode = Node::new();
                                leafnode.archetypeId = LEAF_ARCHETYPE_INDEX;
                                leafnode.visible = 1;
                                leafnode.status = STATUS_ALIVE;
                                leafnode.length = 0.001;
                                leafnode.radius = 0.001;
                                leafnode.transformation = (Matrix4::from_angle_z(Rad(
                                    (rand::random::<f32>() - 0.5) * 2.0,
                                )) * Matrix4::from_angle_x(Rad(
                                    (rand::random::<f32>() - 0.5) * 2.0,
                                )))
                                .into();
                                self.node_list[rightchildindex as usize] = leafnode;
                                self.set_right_child(ni, rightchildindex);
                            }
                        }
                    }
                    STEM_ARCHETYPE_INDEX => {
                        self.node_list[ni as usize].length += logisticDelta(node.length, 0.1, 1.0);
                        self.node_list[ni as usize].radius += logisticDelta(node.radius, 0.02, 1.0);
                    }
                    LEAF_ARCHETYPE_INDEX => {
                        self.node_list[ni as usize].length += logisticDelta(node.length, 0.3, 0.1);
                        self.node_list[ni as usize].radius += logisticDelta(node.radius, 0.05, 0.5);
                    }
                    _ => println!("oof"),
                }
            }
        }
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
pub struct Node {
    pub leftChildIndex: u32,
    pub rightChildIndex: u32,
    pub parentIndex: u32,
    pub age: u32,
    pub archetypeId: u32,
    pub status: u32,
    pub visible: u32,
    pub length: f32,
    pub radius: f32, // also can be width
    pub volume: f32,
    pub absolutePositionCache: [f32; 3],
    pub transformation: [[f32; 4]; 4],
}

impl Node {
    // Node has some dummy variables and this function makes it easier to create a default instance
    pub fn new() -> Node {
        Node {
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

    pub fn fromgpu(node: nodeupdategrid::ty::Node) -> Node {
        Node {
            leftChildIndex: node.leftChildIndex,
            rightChildIndex: node.rightChildIndex,
            parentIndex: node.parentIndex,
            age: node.age,
            archetypeId: node.archetypeId,
            status: node.status,
            visible: node.visible,
            length: node.length,
            radius: node.radius,
            volume: node.volume,
            absolutePositionCache: node.absolutePositionCache,
            transformation: node.transformation,
        }
    }

    pub fn gpu(&self) -> nodeupdategrid::ty::Node {
        nodeupdategrid::ty::Node {
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
