fn thing() -> () 
    // GPU Buffers that will be used for the data
    let grid_data_buffer = grid_buffer.gen_data(device.clone());
    let grid_metadata_buffer = grid_buffer.gen_metadata(device.clone());

    let node_metadata_buffer = node_buffer.gen_metadata(device.clone());
    let node_data_buffer = node_buffer.gen_data(device.clone());
    let node_freestack_buffer = node_buffer.gen_freestack(device.clone());

    // Load shaders
    let gridupdategrid = shader::gridupdategrid::Shader::load(device.clone()).unwrap();
    let nodeupdategrid = shader::nodeupdategrid::Shader::load(device.clone()).unwrap();
    let gridupdatenode = shader::gridupdatenode::Shader::load(device.clone()).unwrap();
    let nodeupdatenode = shader::nodeupdatenode::Shader::load(device.clone()).unwrap();

    // Create pipelines for shaders
    let gridupdategrid_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &gridupdategrid.main_entry_point(), &()).unwrap(),
    );

    let nodeupdategrid_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &nodeupdategrid.main_entry_point(), &()).unwrap(),
    );

    let gridupdatenode_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &gridupdatenode.main_entry_point(), &()).unwrap(),
    );

    let nodeupdatenode_pipeline = Arc::new(
        ComputePipeline::new(device.clone(), &nodeupdatenode.main_entry_point(), &()).unwrap(),
    );

    // Create descriptor sets where the buffers can be placed
    let gridupdategrid_set = Arc::new(
        PersistentDescriptorSet::start(gridupdategrid_pipeline.clone(), 0)
            .add_buffer(grid_metadata_buffer.clone())
            .unwrap()
            .add_buffer(grid_data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let nodeupdategrid_set = Arc::new(
        PersistentDescriptorSet::start(nodeupdategrid_pipeline.clone(), 0)
            .add_buffer(node_metadata_buffer.clone())
            .unwrap()
            .add_buffer(node_data_buffer.clone())
            .unwrap()
            .add_buffer(grid_metadata_buffer.clone())
            .unwrap()
            .add_buffer(grid_data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let gridupdatenode_set = Arc::new(
        PersistentDescriptorSet::start(gridupdatenode_pipeline.clone(), 0)
            .add_buffer(node_metadata_buffer.clone())
            .unwrap()
            .add_buffer(node_data_buffer.clone())
            .unwrap()
            .add_buffer(grid_metadata_buffer.clone())
            .unwrap()
            .add_buffer(grid_data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    let nodeupdatenode_set = Arc::new(
        PersistentDescriptorSet::start(nodeupdatenode_pipeline.clone(), 0)
            .add_buffer(node_metadata_buffer.clone())
            .unwrap()
            .add_buffer(node_data_buffer.clone())
            .unwrap()
            .add_buffer(node_freestack_buffer.clone())
            .unwrap()
            .add_buffer(grid_metadata_buffer.clone())
            .unwrap()
            .add_buffer(grid_data_buffer.clone())
            .unwrap()
            .build()
            .unwrap(),
    );

    // Create command buffers that describe how the shader is to be exected
    let gridupdategrid_command_buffer =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
            .unwrap()
            .dispatch(
                [sim_x_size * sim_y_size * sim_z_size, 1, 1],
                gridupdategrid_pipeline.clone(),
                gridupdategrid_set.clone(),
                (),
            )
            .unwrap()
            .build()
            .unwrap();

    let nodeupdategrid_command_buffer =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
            .unwrap()
            .dispatch(
                [node_buffer.size(), 1, 1],
                nodeupdategrid_pipeline.clone(),
                nodeupdategrid_set.clone(),
                (),
            )
            .unwrap()
            .build()
            .unwrap();

    let gridupdatenode_command_buffer =
        AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family())
            .unwrap()
            .dispatch(
                [node_buffer.size(), 1, 1],
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
                [node_buffer.size(), 1, 1],
                nodeupdatenode_pipeline.clone(),
                nodeupdatenode_set.clone(),
                (),
            )
            .unwrap()
            .build()
            .unwrap();

    // We execute each shader in order, making sure to flush all changes before next
    let compute_future = sync::now(device.clone())
        .then_execute(queue.clone(), gridupdategrid_command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .then_execute(queue.clone(), nodeupdategrid_command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .then_execute(queue.clone(), gridupdatenode_command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap()
        .then_execute(queue.clone(), nodeupdatenode_command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    // Waits for all computation to finish
    compute_future.wait(None).unwrap();

    {
        let vec = node_data_buffer.read().unwrap();
        let u32vec: Vec<u32> = vec.iter().map(|n| n.age).collect();
        dbg!(u32vec);
    }
}
