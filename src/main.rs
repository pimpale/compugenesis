use cgmath::Point3;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use std::sync::Arc;
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceExtensions, Features};
use vulkano::format::*;
use vulkano::image::attachment::AttachmentImage;
use vulkano::image::view::ImageView;
use vulkano::image::{ImageAccess, ImageUsage, SwapchainImage};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::swapchain;
use vulkano::swapchain::{
    AcquireError, ColorSpace, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano::Version;

use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

mod archetype;
mod camera;
mod grid;
mod node;
mod plant;
mod shader;
mod util;
mod vertex;

use archetype::*;
use camera::*;
use grid::*;
use node::*;
use plant::*;

fn main() {
    let required_extensions = vulkano_win::required_extensions();

    let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();

    //Choose the first available Device
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();

    //Print some info about the device currently being used
    println!(
        "Using device: {} (type: {:?})",
        physical.properties().device_name,
        physical.properties().device_type
    );

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&event_loop, instance.clone())
        .unwrap();

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
        &Features::none(),
        &physical.required_extensions().union(&device_ext),
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();

        let alpha = caps.supported_composite_alpha.iter().next().unwrap();

        let format = caps.supported_formats[0].0;

        // Please take a look at the docs for the meaning of the parameters we didn't mention.
        Swapchain::start(device.clone(), surface.clone())
            .num_images(caps.min_image_count)
            .format(format)
            .dimensions(surface.window().inner_size().into())
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .transform(SurfaceTransform::Identity)
            .composite_alpha(alpha)
            .present_mode(PresentMode::Fifo)
            .color_space(ColorSpace::SrgbNonLinear)
            .build()
            .unwrap()
    };

    let vs = shader::vert::load(device.clone()).unwrap();
    let fs = shader::frag::load(device.clone()).unwrap();

    let render_pass = 
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
                    format: Format::D16_UNORM,
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {depth}
            }
        )
        .unwrap();

    let graphics_pipeline = 
        GraphicsPipeline::start()
        // We need to indicate the layout of the vertices.
        .vertex_input_state(BuffersDefinition::new().vertex::<vertex::Vertex>())
        // A Vulkan shader can in theory contain multiple entry points, so we have to specify
        // which one.
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        // The content of the vertex buffer describes a list of triangles.
        .input_assembly_state(InputAssemblyState::new())
        // Use a resizable viewport set to draw over the entire window
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        // See `vertex_shader`.
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        // We have to indicate which subpass of which render pass this pipeline is going to be used
        // in. The pipeline will only be usable from this particular subpass.
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
        .build(device.clone())
        .unwrap();


//            .vertex_input_single_buffer::<vertex::Vertex>()
//            .vertex_shader(vs.main_entry_point(), ())
//            .triangle_list()
//            .viewports_dynamic_scissors_irrelevant(1)
//            .fragment_shader(fs.main_entry_point(), ())
//            .depth_stencil_simple_depth()
//            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
//            .build(device.clone())
//            .unwrap(),

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [0.0, 0.0],
        depth_range: 0.0..1.0,
    };

    let mut camera = Camera::new(Point3::new(0.0, 0.0, -1.0), 50, 50);

    let mut framebuffers = window_size_dependent_setup(
        device.clone(),
        &images,
        render_pass.clone(),
        &mut viewport,
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

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                recreate_swapchain = true;
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let Some(kc) = input.virtual_keycode {
                    match kc {
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
                        _ => (),
                    }
                }
            }
            Event::RedrawEventsCleared => {
                node_buffer.update_all();
                let vertex_buffer = {
                    let mut vecs = node_buffer.gen_vertex(&plant_buffer);
                    vecs.append(&mut grid_buffer.gen_vertex());
                    CpuAccessibleBuffer::from_iter(
                        device.clone(),
                        BufferUsage::all(),
                        false,
                        vecs.iter().cloned(),
                    )
                    .unwrap()
                };
                vertex_buffer.len();

                // free memory
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                // Whenever the window resizes we need to recreate everything dependent on the window size.
                // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
                if recreate_swapchain {
                    // Get the new dimensions of the window.
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_images) =
                        match swapchain.recreate().dimensions(dimensions).build() {
                            Ok(r) => r,
                            // This error tends to happen when the user is manually resizing the window.
                            // Simply restarting the loop is the easiest way to fix this issue.
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    swapchain = new_swapchain;
                    // Because framebuffers contains an Arc on the old swapchain, we need to
                    // recreate framebuffers as well.
                    framebuffers = window_size_dependent_setup(
                        device.clone(),
                        &new_images,
                        render_pass.clone(),
                        &mut viewport,
                        &mut camera,
                    );
                    recreate_swapchain = false;
                }

                // Before we can draw on the output, we have to *acquire* an image from the swapchain. If
                // no image is available (which happens if you submit draw commands too quickly), then the
                // function will block.
                // This operation returns the index of the image that we are allowed to draw upon.
                //
                // This function can block if no image is available. The parameter is an optional timeout
                // after which the function call will return an error.
                let (image_num, suboptimal, acquire_future) =
                    match swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                // acquire_next_image can be successful, but suboptimal. This means that the swapchain image
                // will still work, but it may not display correctly. With some drivers this can be when
                // the window resizes, but it may not cause the swapchain to become out of date.
                if suboptimal {
                    recreate_swapchain = true;
                }

                // In order to draw, we have to build a *command buffer*. The command buffer object holds
                // the list of commands that are going to be executed.
                //
                // Building a command buffer is an expensive operation (usually a few hundred
                // microseconds), but it is known to be a hot path in the driver and is expected to be
                // optimized.
                //
                // Note that we have to pass a queue family when we create the command buffer. The command
                // buffer will only be executable on that given queue family.
                let mut builder = AutoCommandBufferBuilder::primary(
                    device.clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit,
                )
                .unwrap();

                // Finish building the command buffer by calling `build`.
                builder
                    .begin_render_pass(
                        framebuffers[image_num].clone(),
                        SubpassContents::Inline,
                        vec![[0.53, 0.81, 0.92, 1.0].into(), 1f32.into()],
                    )
                    .unwrap()
                    .set_viewport(0, vec![viewport.clone()])
                    .bind_pipeline_graphics(graphics_pipeline.clone())
                    .bind_vertex_buffers(0, vertex_buffer.clone())
                    .push_constants(
                        graphics_pipeline.layout().clone(),
                        // TODO: location?
                        0,
                        shader::vert::ty::PushConstantData {
                            mvp: camera.mvp().into(),
                        },
                    )
                    .draw(vertex_buffer.len() as u32, 1, 0, 0)
                    .unwrap()
                    // We leave the render pass by calling `draw_end`. Note that if we had multiple
                    // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
                    // next subpass.
                    .end_render_pass()
                    .unwrap();

                let command_buffer = builder.build().unwrap();
                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    // The color output is now expected to contain our triangle. But in order to show it on
                    // the screen, we have to *present* the image by calling `present`.
                    //
                    // This function does not actually present the image immediately. Instead it submits a
                    // present command at the end of the queue. This means that it will only be presented once
                    // the GPU has finished executing the command buffer that draws the triangle.
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => (),
        }
    });
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    device: Arc<Device>,
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
    camera: &mut Camera,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions();
    viewport.dimensions = [dimensions.width() as f32, dimensions.height() as f32];
    camera.set_screen(dimensions.width(), dimensions.height());

    let depth_buffer = ImageView::new(
        AttachmentImage::transient(device, dimensions.width_height(), Format::D16_UNORM).unwrap(),
    )
    .unwrap();

    images
        .iter()
        .map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Framebuffer::start(render_pass.clone())
                .add(view)
                .unwrap()
                .add(depth_buffer.clone())
                .unwrap()
                .build()
                .unwrap()
        })
        .collect::<Vec<_>>()
}
