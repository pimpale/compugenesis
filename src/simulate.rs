use super::cgmath::{Deg, Matrix3, Matrix4, Point3, Rad, Vector3};
use super::serde::{Deserialize, Serialize};
use super::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use super::vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use super::vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use super::vulkano::device::{Device, DeviceExtensions, Queue};
use super::vulkano::format::Format;
use super::vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use super::vulkano::image::attachment::AttachmentImage;
use super::vulkano::image::SwapchainImage;
use super::vulkano::instance::debug::{DebugCallback, MessageTypes};
use super::vulkano::instance::{Instance, PhysicalDevice};
use super::vulkano::pipeline::ComputePipeline;
use super::vulkano::swapchain;
use super::vulkano::sync;
use super::vulkano::sync::FlushError;
use super::vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::sync::RwLock;

use super::grid::*;
use super::node::*;
use super::shader;
use super::shader::header::ty;

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationState {
    node_buffer: NodeBuffer,
    grid_buffer: GridBuffer,
    plant_buffer: PlantBuffer,
}

#[derive(Clone)]
pub struct Control {
    // info
    current_node_count: u32,
    max_node_count: u32,
    current_cycle_count: u32,

    // end simulation
    should_terminate: bool,
    has_terminated: bool,

    // pause simulation
    should_pause: bool,
    has_paused: bool,

    // simulation speed
    target_fps: u32,
    current_fps: u32,
}

impl Control {
    pub fn new() -> Control {
        Control {
            current_node_count: 0,
            max_node_count: 0,
            current_cycle_count: 0,
            should_terminate: false,
            has_terminated: false,
            should_pause: true,
            has_paused: false,
            target_fps: 0,
            current_fps: 0,
        }
    }
}

fn run_cycle(
    grid_data_buffer_read: Arc<CpuAccessibleBuffer<[ty::GridCell]>>,
    grid_data_buffer_write: Arc<CpuAccessibleBuffer<[ty::GridCell]>>,
    grid_metadata_buffer_read: Arc<CpuAccessibleBuffer<ty::GridMetadata>>,
    grid_metadata_buffer_write: Arc<CpuAccessibleBuffer<ty::GridMetadata>>,

    node_data_buffer_read: Arc<CpuAccessibleBuffer<[ty::Node]>>,
    node_data_buffer_write: Arc<CpuAccessibleBuffer<[ty::Node]>>,

    node_freestack_buffer_read: Arc<CpuAccessibleBuffer<[ty::Node]>>,
    node_freestack_buffer_write: Arc<CpuAccessibleBuffer<[ty::Node]>>,

    node_metadata_buffer_read: Arc<CpuAccessibleBuffer<ty::NodeMetadata>>,
    node_metadata_buffer_write: Arc<CpuAccessibleBuffer<ty::NodeMetadata>>,

    plant_data_buffer_read: Arc<CpuAccessibleBuffer<[ty::Plant]>>,
    plant_data_buffer_write: Arc<CpuAccessibleBuffer<[ty::Plant]>>,

    plant_metadata_buffer_read: Arc<CpuAccessibleBuffer<ty::PlantMetadata>>,
    plant_metadata_buffer_write: Arc<CpuAccessibleBuffer<ty::PlantMetadata>>,
) -> () {

}

fn simulate(
    state: SimulationState,
    control: Arc<RwLock<Control>>,
    queue: Arc<Queue>,
    device: Arc<Device>,
) -> () {
    let node_buffer = state.node_buffer;
    let grid_buffer = state.grid_buffer;

    // GPU Buffers that will be used for the data
    let grid_data_buffer_1 = grid_buffer.gen_data(device.clone());
    let grid_metadata_buffer_1 = grid_buffer.gen_metadata(device.clone());

    let grid_data_buffer_2 = grid_buffer.gen_data(device.clone());
    let grid_metadata_buffer_2 = grid_buffer.gen_metadata(device.clone());

    let node_metadata_buffer_1 = node_buffer.gen_metadata(device.clone());
    let node_data_buffer_1 = node_buffer.gen_data(device.clone());
    let node_freestack_buffer_1 = node_buffer.gen_freestack(device.clone());

    let node_metadata_buffer_2 = node_buffer.gen_metadata(device.clone());
    let node_data_buffer_2 = node_buffer.gen_data(device.clone());
    let node_freestack_buffer_2 = node_buffer.gen_freestack(device.clone());

    // Load shaders
    let gridupdatenode = shader::gridupdatenode::Shader::load(device.clone()).unwrap();
    let nodeupdatenode = shader::nodeupdatenode::Shader::load(device.clone()).unwrap();

    let gridupdatenode_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &gridupdatenode.main_entry_point(), &()).unwrap(),
    );

    let nodeupdatenode_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &nodeupdatenode.main_entry_point(), &()).unwrap(),
    );

    let mut previous_frame_end = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

    loop {
        let gridupdatenode_set_1 = Arc::new(
            PersistentDescriptorSet::start(gridupdatenode_pipeline.clone(), 0)
                .add_buffer(node_metadata_buffer_1.clone())
                .unwrap()
                .add_buffer(node_data_buffer_1.clone())
                .unwrap()
                .add_buffer(grid_metadata_buffer_1.clone())
                .unwrap()
                .add_buffer(grid_data_buffer_1.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let gridupdatenode_set_2 = Arc::new(
            PersistentDescriptorSet::start(gridupdatenode_pipeline.clone(), 0)
                .add_buffer(node_metadata_buffer_2.clone())
                .unwrap()
                .add_buffer(node_data_buffer_2.clone())
                .unwrap()
                .add_buffer(grid_metadata_buffer_2.clone())
                .unwrap()
                .add_buffer(grid_data_buffer_2.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        let nodeupdatenode_set_1 = Arc::new(
            PersistentDescriptorSet::start(nodeupdatenode_pipeline.clone(), 0)
                .add_buffer(node_metadata_buffer_1.clone())
                .unwrap()
                .add_buffer(node_data_buffer_1.clone())
                .unwrap()
                .add_buffer(node_freestack_buffer_1.clone())
                .unwrap()
                .add_buffer(grid_metadata_buffer_1.clone())
                .unwrap()
                .add_buffer(grid_data_buffer_1.clone())
                .unwrap()
                .build()
                .unwrap(),
        );
        let nodeupdatenode_set_2 = Arc::new(
            PersistentDescriptorSet::start(nodeupdatenode_pipeline.clone(), 0)
                .add_buffer(node_metadata_buffer_2.clone())
                .unwrap()
                .add_buffer(node_data_buffer_2.clone())
                .unwrap()
                .add_buffer(node_freestack_buffer_2.clone())
                .unwrap()
                .add_buffer(grid_metadata_buffer_2.clone())
                .unwrap()
                .add_buffer(grid_data_buffer_2.clone())
                .unwrap()
                .build()
                .unwrap(),
        );

        // Create command buffers that describe how the shader is to be executed
        let gridupdatenode_command_buffer_1 =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [node_buffer.size(), 1, 1],
                    gridupdatenode_pipeline.clone(),
                    gridupdatenode_set_1.clone(),
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let gridupdatenode_command_buffer_2 =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [node_buffer.size(), 1, 1],
                    gridupdatenode_pipeline_2.clone(),
                    gridupdatenode_set.clone(),
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let nodeupdatenode_command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [node_buffer.size(), 1, 1],
                    nodeupdatenode_pipeline_1.clone(),
                    nodeupdatenode_set.clone(),
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let nodeupdatenode_command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .dispatch(
                    [node_buffer.size(), 1, 1],
                    nodeupdatenode_pipeline_2.clone(),
                    nodeupdatenode_set.clone(),
                    (),
                )
                .unwrap()
                .build()
                .unwrap();

        let future = previous_frame_end
            .then_execute(queue.clone(), gridupdatenode_command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap()
            .then_execute(queue.clone(), nodeupdatenode_command_buffer)
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        previous_frame_end = Box::new(future) as Box<_>;
    }
}
