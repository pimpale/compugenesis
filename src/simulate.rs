use super::cgmath::{Deg, Matrix3, Matrix4, Point3, Rad, Vector3};
use super::serde::{Deserialize, Serialize};
use super::vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use super::vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use super::vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use super::vulkano::descriptor::pipeline_layout::PipelineLayout;
use super::vulkano::device::{Device, DeviceExtensions, Queue};
use super::vulkano::format::Format;
use super::vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use super::vulkano::image::attachment::AttachmentImage;
use super::vulkano::image::SwapchainImage;
use super::vulkano::instance::debug::{DebugCallback, MessageTypes};
use super::vulkano::instance::{Instance, PhysicalDevice};
use super::vulkano::pipeline::ComputePipeline;
use super::vulkano::pipeline::ComputePipelineAbstract;
use super::vulkano::swapchain;
use super::vulkano::sync;
use super::vulkano::sync::FlushError;
use super::vulkano::sync::GpuFuture;

use std::sync::Arc;
use std::sync::RwLock;

use super::grid;
use super::node;
use super::plant;

use super::shader;
use super::shader::header::ty;

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationState {
    node_buffer: node::NodeBuffer,
    grid_buffer: grid::GridBuffer,
    plant_buffer: plant::PlantBuffer,
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
    queue: Arc<Queue>,
    device: Arc<Device>,
    previous_frame_end: Box<GpuFuture>,

    // Node Count
    node_count: u32,

    // Pipelines
    gridupdatenode_pipeline: Arc<ComputePipeline<PipelineLayout<shader::gridupdatenode::Shader>>>,
    nodeupdatenode_pipeline: Arc<ComputePipeline<PipelineLayout<shader::nodeupdatenode::Shader>>>,

    // Grid Data
    grid_data_buffer: Arc<CpuAccessibleBuffer<[ty::GridCell]>>,
    grid_data_buffer_alt: Arc<CpuAccessibleBuffer<[ty::GridCell]>>,

    // Grid Metadata
    grid_metadata_buffer: Arc<CpuAccessibleBuffer<ty::GridMetadata>>,
    grid_metadata_buffer_alt: Arc<CpuAccessibleBuffer<ty::GridMetadata>>,

    // Node Data
    node_data_buffer: Arc<CpuAccessibleBuffer<[ty::Node]>>,
    node_data_buffer_alt: Arc<CpuAccessibleBuffer<[ty::Node]>>,

    // Node Freestack
    node_freestack_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    node_freestack_buffer_alt: Arc<CpuAccessibleBuffer<[u32]>>,

    // Node Metadata
    node_metadata_buffer: Arc<CpuAccessibleBuffer<ty::NodeMetadata>>,
    node_metadata_buffer_alt: Arc<CpuAccessibleBuffer<ty::NodeMetadata>>,

    // Plant data
    plant_data_buffer: Arc<CpuAccessibleBuffer<[ty::Plant]>>,
    plant_data_buffer_alt: Arc<CpuAccessibleBuffer<[ty::Plant]>>,

    // Plant Freestack
    plant_freestack_buffer: Arc<CpuAccessibleBuffer<[u32]>>,
    plant_freestack_buffer_alt: Arc<CpuAccessibleBuffer<[u32]>>,

    // Plant Metadata
    plant_metadata_buffer: Arc<CpuAccessibleBuffer<ty::PlantMetadata>>,
    plant_metadata_buffer_alt: Arc<CpuAccessibleBuffer<ty::PlantMetadata>>,
) -> Box<GpuFuture> {
    let gridupdatenode_set = Arc::new(
        PersistentDescriptorSet::start(gridupdatenode_pipeline.clone(), 0)
            // Read grid
            .add_buffer(grid_metadata_buffer.clone())
            .unwrap()
            .add_buffer(grid_data_buffer.clone())
            .unwrap()
            // Write grid
            .add_buffer(grid_metadata_buffer_alt.clone())
            .unwrap()
            .add_buffer(grid_data_buffer_alt.clone())
            .unwrap()
            // Read node
            .add_buffer(node_metadata_buffer.clone())
            .unwrap()
            .add_buffer(node_data_buffer.clone())
            .unwrap()
            // Write Node
            .add_buffer(node_metadata_buffer_alt.clone())
            .unwrap()
            .add_buffer(node_data_buffer_alt.clone())
            .unwrap()
            // Build
            .build()
            .unwrap(),
    );

    // This runs afterwards. We read from the newly written grid and write to the old grid
    let nodeupdatenode_set = Arc::new(
        PersistentDescriptorSet::start(nodeupdatenode_pipeline.clone(), 0)
            // Read grid
            .add_buffer(grid_metadata_buffer_alt.clone())
            .unwrap()
            .add_buffer(grid_data_buffer_alt.clone())
            .unwrap()
            // Write grid
            .add_buffer(grid_metadata_buffer.clone())
            .unwrap()
            .add_buffer(grid_data_buffer.clone())
            .unwrap()
            // Read node
            .add_buffer(node_metadata_buffer_alt.clone())
            .unwrap()
            .add_buffer(node_data_buffer_alt.clone())
            .unwrap()
            // Write Node
            .add_buffer(node_metadata_buffer.clone())
            .unwrap()
            .add_buffer(node_data_buffer.clone())
            .unwrap()
            // Build
            .build()
            .unwrap(),
    );

    let gridupdatenode_command_buffer =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
            .unwrap()
            .dispatch(
                [node_count, 1, 1],
                gridupdatenode_pipeline.clone(),
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
                [node_count, 1, 1],
                nodeupdatenode_pipeline.clone(),
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

    Box::new(future) as Box<_>
}

fn simulate(
    state: SimulationState,
    control: Arc<RwLock<Control>>,
    queue: Arc<Queue>,
    device: Arc<Device>,
) -> () {
    let node_buffer = state.node_buffer;
    let grid_buffer = state.grid_buffer;
    let plant_buffer = state.plant_buffer;

    // GPU Buffers that will be used for the data
    let grid_data_buffer = grid_buffer.gen_data(device.clone());
    let grid_metadata_buffer = grid_buffer.gen_metadata(device.clone());

    let grid_data_buffer_alt = grid_buffer.gen_data(device.clone());
    let grid_metadata_buffer_alt = grid_buffer.gen_metadata(device.clone());

    let node_metadata_buffer = node_buffer.gen_metadata(device.clone());
    let node_data_buffer = node_buffer.gen_data(device.clone());
    let node_freestack_buffer = node_buffer.gen_freestack(device.clone());

    let node_metadata_buffer_alt = node_buffer.gen_metadata(device.clone());
    let node_data_buffer_alt = node_buffer.gen_data(device.clone());
    let node_freestack_buffer_alt = node_buffer.gen_freestack(device.clone());

    let plant_metadata_buffer = plant_buffer.gen_metadata(device.clone());
    let plant_data_buffer = plant_buffer.gen_data(device.clone());
    let plant_freestack_buffer = plant_buffer.gen_freestack(device.clone());

    let plant_metadata_buffer_alt = plant_buffer.gen_metadata(device.clone());
    let plant_data_buffer_alt = plant_buffer.gen_data(device.clone());
    let plant_freestack_buffer_alt = plant_buffer.gen_freestack(device.clone());

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
        previous_frame_end = run_cycle(
            queue.clone(),
            device.clone(),
            previous_frame_end,
            node_buffer.size(),
            gridupdatenode_pipeline.clone(),
            nodeupdatenode_pipeline.clone(),
            grid_data_buffer.clone(),
            grid_data_buffer_alt.clone(),
            grid_metadata_buffer.clone(),
            grid_metadata_buffer_alt.clone(),
            node_data_buffer.clone(),
            node_data_buffer_alt.clone(),
            node_freestack_buffer.clone(),
            node_freestack_buffer_alt.clone(),
            node_metadata_buffer.clone(),
            node_metadata_buffer_alt.clone(),
            plant_data_buffer.clone(),
            plant_data_buffer_alt.clone(),
            plant_freestack_buffer.clone(),
            plant_freestack_buffer_alt.clone(),
            plant_metadata_buffer.clone(),
            plant_metadata_buffer_alt.clone(),
        );
    }
}
