use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    dim: u32,
    n: u32,
    mod_val: u32,
    res: i32,
}

pub struct StampFolder {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    p_array_buffer: wgpu::Buffer,
    count_buffer: wgpu::Buffer,
}

impl StampFolder {
    pub async fn new(device: &wgpu::Device, dimensions: &[i32], mod_val: u32, res: i32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Stamp Folding Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Stamp Folding Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(std::num::NonZeroU64::new(std::mem::size_of::<Params>() as u64).unwrap()),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Stamp Folding Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipeline
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Stamp Folding Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });

        // Calculate n and ensure it's within bounds
        let n: i32 = dimensions.iter().product();
        assert!(n < 64, "Dimension too large: product must be less than 64");

        // Create params
        let params = Params {
            dim: dimensions.len() as u32,
            n: n as u32,
            mod_val,
            res,
        };

        println!("Creating params: {:?}", params);

        // Create a reference to params as a slice
        let params_slice = std::slice::from_ref(&params);
        println!("Params as slice bytes: {:?}", bytemuck::bytes_of(&params));

        // Main params buffer for shader use
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
        });

        let mut padded_dimensions = vec![0i32; 64];
        padded_dimensions[..dimensions.len()].copy_from_slice(dimensions);
        println!("Padded dimensions: {:?}", &padded_dimensions[..dimensions.len()]);

        let p_array_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("P Array Buffer"),
            contents: bytemuck::cast_slice(&padded_dimensions),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        // Initialize count buffer with zeros
        let count_buffer_data = vec![0i32; 64];
        let count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Count Buffer"),
            contents: bytemuck::cast_slice(&count_buffer_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Stamp Folding Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: p_array_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            params_buffer,
            p_array_buffer,
            count_buffer,
        }
    }

    pub async fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> i64 {
        // Submit compute pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });

        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Stamp Folding Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }

        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait); // Wait for compute to finish

        // Create staging buffers
        let params_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Params Staging Buffer"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let results_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Results Staging Buffer"),
            size: self.count_buffer.size(),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Copy data to staging buffers
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &self.params_buffer,
            0,
            &params_staging_buffer,
            0,
            params_staging_buffer.size()
        );

        encoder.copy_buffer_to_buffer(
            &self.count_buffer,
            0,
            &results_staging_buffer,
            0,
            results_staging_buffer.size()
        );

        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::Maintain::Wait); // Wait for copies to finish

        // Read parameters
        let params_slice = params_staging_buffer.slice(..);
        let (params_sender, params_receiver) = futures_intrusive::channel::shared::oneshot_channel();
        params_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = params_sender.send(result);
        });

        let results_slice = results_staging_buffer.slice(..);
        let (results_sender, results_receiver) = futures_intrusive::channel::shared::oneshot_channel();
        results_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = results_sender.send(result);
        });

        device.poll(wgpu::Maintain::Wait);

        // Wait for both mappings to complete
        if params_receiver.receive().await.unwrap().is_ok() {
            let params_data = params_slice.get_mapped_range();
            let params: &[Params] = bytemuck::cast_slice(&*params_data);
            println!("Shader params: {:?}", params[0]);
        }

        if results_receiver.receive().await.unwrap().is_ok() {
            let data = results_slice.get_mapped_range();
            let result: Vec<i32> = bytemuck::cast_slice(&data).to_vec();
            println!("Raw results: {:?}", &result[..64]);
            result.iter().take(64).map(|&x| x as i64).sum()
        } else {
            0
        }
    }

    pub async fn calculate_sequence(dimensions: &[i32]) -> i64 {
        // Special case: if any dimension is 0, return 1
        if dimensions.iter().any(|&d| d == 0) {
            return 1;
        }

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
            .await
            .unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None
        ).await.unwrap();

        println!("\nProcessing dimensions: {:?}", dimensions);
        let compute = StampFolder::new(&device, dimensions, 0, 0).await;
        compute.compute(&device, &queue).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sequence_n_2() {
        let expected = vec![
            1, 2, 8, 60, 320, 1980, 10512, 60788, 320896,
            1787904, 9381840, 51081844
        ];

        for (i, &expected_value) in expected.iter().enumerate() {
            let dimensions = vec![i as i32, 2];
            let result = StampFolder::calculate_sequence(&dimensions).await;
            assert_eq!(
                result,
                expected_value,
                "Failed for n={}, width=2: expected {}, got {}",
                i,
                expected_value,
                result
            );
        }
    }
}