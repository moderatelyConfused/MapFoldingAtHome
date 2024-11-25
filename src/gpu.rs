use wgpu::util::DeviceExt;

const MAX_N: usize = 64;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    dim: u32,
    n: u32,
    mod_val: u32,
    res: i32,
}

pub struct PrecomputedArrays {
    big_p: [i32; MAX_N],
    c: [[i32; MAX_N]; MAX_N],
    d: Box<[i32; MAX_N * MAX_N * MAX_N]>,
}

impl Default for PrecomputedArrays {
    fn default() -> Self {
        Self {
            big_p: [0; MAX_N],
            c: [[0; MAX_N]; MAX_N],
            d: Box::new([0; MAX_N * MAX_N * MAX_N]),
        }
    }
}

pub struct StampFolder {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    params_buffer: wgpu::Buffer,
    big_p_buffer: wgpu::Buffer,
    c_buffer: wgpu::Buffer,
    d_buffer: wgpu::Buffer,
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
                        min_binding_size: None,
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
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
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        // Calculate n and ensure it's within bounds
        let n: i32 = dimensions.iter().product();
        assert!(n < MAX_N as i32, "Dimension too large: product must be less than MAX_N");

        // Create params
        let params = Params {
            dim: dimensions.len() as u32,
            n: n as u32,
            mod_val,
            res,
        };

        // Precompute arrays
        let mut precomputed = PrecomputedArrays::default();

        // Calculate big_p
        precomputed.big_p[0] = 1;
        for i in 1..=dimensions.len() {
            precomputed.big_p[i] = precomputed.big_p[i - 1] * dimensions[i - 1];
        }

        // Calculate c array
        for i in 1..=dimensions.len() {
            for m in 1..=n as usize {
                let big_p_im1 = precomputed.big_p[i - 1];
                let big_p_i = precomputed.big_p[i];
                let p_im1 = dimensions[i - 1];
                precomputed.c[i][m] = (m as i32 - 1) / big_p_im1 - ((m as i32 - 1) / big_p_i) * p_im1 + 1;
            }
        }

        // Calculate d array
        for i in 1..=dimensions.len() {
            for l in 1..=n as usize {
                for m in 1..=l {
                    let idx = i * MAX_N * MAX_N + l * MAX_N + m;
                    let delta = precomputed.c[i][l] - precomputed.c[i][m];
                    
                    precomputed.d[idx] = if (delta & 1) == 0 {
                        if precomputed.c[i][m] == 1 {
                            m as i32
                        } else {
                            m as i32 - precomputed.big_p[i - 1]
                        }
                    } else if precomputed.c[i][m] == dimensions[i - 1] || (m as i32 + precomputed.big_p[i - 1] > l as i32) {
                        m as i32
                    } else {
                        m as i32 + precomputed.big_p[i - 1]
                    };
                }
            }
        }

        // Create buffers
        let params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Buffer"),
            contents: bytemuck::cast_slice(&[params]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
        });

        let big_p_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Big P Buffer"),
            contents: bytemuck::cast_slice(&precomputed.big_p),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let c_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("C Buffer"),
            contents: bytemuck::cast_slice(&precomputed.c),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let d_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("D Buffer"),
            contents: bytemuck::cast_slice(precomputed.d.as_ref()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });

        let count_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Count Buffer"),
            contents: bytemuck::cast_slice(&[0i32; MAX_N]),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
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
                    resource: big_p_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: c_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: d_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: count_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            pipeline,
            bind_group,
            params_buffer,
            big_p_buffer,
            c_buffer,
            d_buffer,
            count_buffer,
        }
    }

    pub async fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> i64 {
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
        device.poll(wgpu::Maintain::Wait);

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: std::mem::size_of::<i32>() as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &self.count_buffer,
            0,
            &staging_buffer,
            0,
            std::mem::size_of::<i32>() as u64,
        );

        queue.submit(Some(encoder.finish()));

        let buffer_slice = staging_buffer.slice(..);
        let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        device.poll(wgpu::Maintain::Wait);

        if receiver.receive().await.unwrap().is_ok() {
            let data = buffer_slice.get_mapped_range();
            let result: i32 = bytemuck::cast_slice(&*data)[0];
            result as i64
        } else {
            0
        }
    }

    pub async fn calculate_sequence(dimensions: &[i32]) -> i64 {
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
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None
        ).await.unwrap();

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

    #[tokio::test]
    async fn test_sequence_n_3() {
        let expected = vec![
            1, 6, 60, 1368, 15552, 201240, 2016432, 21582624
        ];

        for (i, &expected_value) in expected.iter().enumerate() {
            let dimensions = vec![i as i32, 3];
            let result = StampFolder::calculate_sequence(&dimensions).await;
            assert_eq!(
                result,
                expected_value,
                "Failed for n={}, width=3: expected {}, got {}",
                i,
                expected_value,
                result
            );
        }
    }

    #[tokio::test]
    async fn test_sequence_n_n() {
        let expected = vec![1, 1, 8, 1368, 300608];

        for (i, &expected_value) in expected.iter().enumerate() {
            let n = i as i32;
            let dimensions = vec![n, n];
            let result = StampFolder::calculate_sequence(&dimensions).await;
            assert_eq!(
                result,
                expected_value,
                "Failed for n×n where n={}: expected {}, got {}",
                n,
                expected_value,
                result
            );
        }
    }
}
