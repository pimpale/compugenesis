#[macro_use]
extern crate vulkano;
extern crate cgmath;
extern crate csv;
extern crate gio;
extern crate gtk;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate vulkano_shaders;
extern crate vulkano_win;
extern crate winit;

use cgmath::{Deg, Matrix3, Matrix4, Point3, Rad, Vector3};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::format::Format;
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::SwapchainImage;
use vulkano::instance::debug::{DebugCallback, MessageTypes};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::ComputePipeline;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::FlushError;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::{Event, EventsLoop, VirtualKeyCode, Window, WindowBuilder, WindowEvent};

mod archetype;
mod camera;
mod grid;
mod gui;
mod node;
mod plant;
mod shader;
mod simulate;
mod util;
mod vertex;

use archetype::*;
use camera::*;
use grid::*;
use gui::*;
use node::*;
use plant::*;
use simulate::*;

fn create_instance() -> (Arc<Instance>, DebugCallback) {
    let instance = {
        let mut extensions = vulkano_win::required_extensions();
        extensions.ext_debug_report = true;
        Instance::new(
            None,
            &extensions,
            vec!["VK_LAYER_LUNARG_standard_validation"],
        )
        .unwrap()
    };

    let debug_callback = DebugCallback::new(
        &instance,
        MessageTypes {
            error: true,
            warning: true,
            performance_warning: false,
            information: false,
            debug: false,
        },
        |msg| {
            println!("validation layer: {:?}", msg.description);
        },
    )
    .unwrap();
    return (instance, debug_callback);
}

fn main() {
    let (instance, _debug_callback) = create_instance();
    //Choose the first available Device
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    //Print some info about the device currently being used
    println!(
        "Using device: {} (type: {:?})",
        physical.name(),
        physical.ty()
    );

    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();
    let window = surface.window();

    let queue_family = physical
        .queue_families()
        .find(|&q| {
            q.supports_graphics()
                && q.supports_compute()
                && surface.is_supported(q).unwrap_or(false)
        })
        .unwrap();

    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let settings_packet = Arc::new(RwLock::new(gui::SettingsPacket {
        paused: true,
        request_stop: false,
        requested_fps: None,
        simulation_duration: None, //In cycles
    }));

    /* TODO make gui
    std::thread::spawn(move || {
        gtk_run(settings_packet.clone());
    });
    */

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let usage = caps.supported_usage_flags;
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let initial_dimensions = if let Some(dimensions) = window.get_inner_size() {
            let dimensions: (u32, u32) = dimensions.to_physical(window.get_hidpi_factor()).into();
            [dimensions.0, dimensions.1]
        } else {
            return;
        };

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            initial_dimensions,
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        )
        .unwrap()
    };

    let vs = shader::vert::Shader::load(device.clone()).unwrap();
    let fs = shader::frag::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(device.clone(),
            attachments: {
                color: {
                    load: Clear,
                    store: Store,
                    format: swapchain.format(),
                    samples: 1,
                },
                depth: {
                    load: Clear,
                    store: DontCare,
                    format: Format::D16Unorm,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap(),
    );

    let graphics_pipeline = Arc::new(
        GraphicsPipeline::start()
            .vertex_input_single_buffer()
            .vertex_shader(vs.main_entry_point(), ())
            .triangle_list()
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            .depth_stencil_simple_depth()
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
    };

    let mut camera = Camera::new(Point3::new(0.0, 0.0, -1.0), 50, 50);
    let mut framebuffers = window_size_dependent_setup(
        device.clone(),
        &images,
        render_pass.clone(),
        &mut dynamic_state,
        &mut camera,
    );

    //Compute stuff

    // The 3d size of the simulation in meters
    let sim_x_size: u32 = 10;
    let sim_y_size: u32 = 10;
    let sim_z_size: u32 = 10;

    let mut plant_buffer = PlantBuffer::new(50);
    //
    // The maximum node capacity of the node buffer
    let mut node_buffer = NodeBuffer::new(50);
    for i in 0..5 {
        let pindex = plant_buffer.alloc();
        let mut plant = Plant::new();

        plant.location = [0.0, i as f32, 0.0];
        plant.status = STATUS_ALIVE;
        plant_buffer.set(pindex, plant);

        let nindex = node_buffer.alloc();

        let mut node = Node::new();

        node.status = STATUS_ALIVE;
        node.archetypeId = GROWING_BUD_ARCHETYPE_INDEX;
        node.visible = 1;
        node.plantId = pindex;
        node.length = 0.05;
        node.radius = 0.01;
        node.volume = 0.1;

        node_buffer.set(nindex, node);
    }
    let mut grid_buffer = GridBuffer::new(sim_x_size, sim_y_size, sim_z_size);

    for x in 0..sim_x_size {
        for z in 0..sim_y_size {
            let height = ((sim_y_size as f32) * rand::random::<f32>()) as u32;
            for y in 0..sim_z_size {
                grid_buffer.set(
                    x,
                    y,
                    z,
                    GridCell {
                        //Initialize the array to be filled with dirt halfway
                        typeCode: if y > height {
                            grid::GRIDCELL_TYPE_AIR
                        } else {
                            grid::GRIDCELL_TYPE_SOIL
                        },
                        temperature: 0,
                        moisture: 0,
                        sunlight: 0,
                        gravity: 0,
                        plantDensity: 0,
                    },
                );
            }
        }
    }
    //

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

    loop {
        // Add delay
        // thread::sleep(Duration::from_millis(40));
        // Graphics

        previous_frame_end.cleanup_finished();

        node_buffer.update_all();
        let vertex_buffer = {
            let mut vecs = node_buffer.gen_vertex(&plant_buffer);
            vecs.append(&mut grid_buffer.gen_vertex());
            CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), vecs.iter().cloned())
                .unwrap()
        };

        if recreate_swapchain {
            let dimensions = if let Some(dimensions) = window.get_inner_size() {
                let dimensions: (u32, u32) =
                    dimensions.to_physical(window.get_hidpi_factor()).into();
                [dimensions.0, dimensions.1]
            } else {
                return;
            };

            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(dimensions) {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => continue,
                Err(err) => panic!("{:?}", err),
            };

            swapchain = new_swapchain;
            framebuffers = window_size_dependent_setup(
                device.clone(),
                &new_images,
                render_pass.clone(),
                &mut dynamic_state,
                &mut camera,
            );

            recreate_swapchain = false;
        }

        let (image_num, acquire_future) =
            match swapchain::acquire_next_image(swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    recreate_swapchain = true;
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

        let command_buffer =
            AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
                .unwrap()
                .begin_render_pass(
                    framebuffers[image_num].clone(),
                    false,
                    // Sky blue
                    vec![[0.53, 0.81, 0.92, 1.0].into(), 1f32.into()],
                )
                .unwrap()
                .draw(
                    graphics_pipeline.clone(),
                    &dynamic_state,
                    vertex_buffer.clone(),
                    (),
                    shader::vert::ty::PushConstantData {
                        mvp: camera.mvp().into(),
                    },
                )
                .unwrap()
                .end_render_pass()
                .unwrap()
                .build()
                .unwrap();

        let future = previous_frame_end
            .join(acquire_future)
            .then_execute(queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                previous_frame_end = Box::new(future) as Box<_>;
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Box::new(sync::now(device.clone())) as Box<_>;
            }
        }

        let mut done = false;
        events_loop.poll_events(|ev| match ev {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => done = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => recreate_swapchain = true,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { device_id, input },
                ..
            } => {
                let _ = device_id;
                let kc = input.virtual_keycode;
                if kc.is_some() {
                    match kc.unwrap() {
                        VirtualKeyCode::W => camera.dir_move(CameraMovementDir::Forward),
                        VirtualKeyCode::A => camera.dir_move(CameraMovementDir::Left),
                        VirtualKeyCode::S => camera.dir_move(CameraMovementDir::Backward),
                        VirtualKeyCode::D => camera.dir_move(CameraMovementDir::Right),
                        VirtualKeyCode::Q => camera.dir_move(CameraMovementDir::Upward),
                        VirtualKeyCode::E => camera.dir_move(CameraMovementDir::Downward),

                        VirtualKeyCode::Up => camera.dir_rotate(CameraRotationDir::Upward),
                        VirtualKeyCode::Left => camera.dir_rotate(CameraRotationDir::Left),
                        VirtualKeyCode::Down => camera.dir_rotate(CameraRotationDir::Downward),
                        VirtualKeyCode::Right => camera.dir_rotate(CameraRotationDir::Right),
                        VirtualKeyCode::PageUp => camera.dir_rotate(CameraRotationDir::Clockwise),
                        VirtualKeyCode::PageDown => {
                            camera.dir_rotate(CameraRotationDir::Counterclockwise)
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        });
        if done {
            return;
        }
    }
}

fn serialize_to_path<P: AsRef<Path>>(path: P, state: SimulationState) -> Result<(), Box<Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let res = serde_json::to_writer(writer, &state)?;
    Ok(res)
}

fn deserialize_from_path<P: AsRef<Path>>(path: P) -> Result<SimulationState, Box<Error>> {
    // Open the file in read-only mode with buffer.
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let u = serde_json::from_reader(reader)?;

    // Return the data.
    Ok(u)
}

fn window_size_dependent_setup(
    device: Arc<Device>,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
    camera: &mut Camera,
) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    camera.setscreen(dimensions[0], dimensions[1]);
    dynamic_state.viewports = Some(vec![viewport]);

    let depth_buffer =
        AttachmentImage::transient(device.clone(), dimensions, Format::D16Unorm).unwrap();

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .add(depth_buffer.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
