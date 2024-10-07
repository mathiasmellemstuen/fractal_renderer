use std::borrow::Cow; 
use crate::fractal_config::FractalConfig;
use std::sync::Arc; 

pub struct WGPUContext {
    pub surface : wgpu::Surface<'static>,
    pub surface_config : wgpu::SurfaceConfiguration,
    pub device : wgpu::Device,
    pub queue : wgpu::Queue,
    pub pipeline : wgpu::RenderPipeline,
    pub bind_group : wgpu::BindGroup,
    pub uniform_buffer : wgpu::Buffer
}

impl WGPUContext {
    pub fn setup(window : Arc<winit::window::Window>) -> WGPUContext {
        
        let mut size = window.inner_size(); 
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).expect("Failed to find an appropriate adapter!");


        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label : None, 
            required_features : wgpu::Features::empty(),
            required_limits : wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            memory_hints : wgpu::MemoryHints::MemoryUsage,
        },
        None
        )).expect("Failed to create device");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label : None,
            source : wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/mandelbrot.wgsl"))),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label : None, 
            size : std::mem::size_of::<FractalConfig>() as u64,
            usage : wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation : false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label : None, 
            entries : &[wgpu::BindGroupLayoutEntry {
                binding : 0, 
                visibility : wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty : wgpu::BindingType::Buffer {
                    ty : wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset : false,
                    min_binding_size : None,
                },
                count : None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label : None,
            layout : &bind_group_layout,
            entries : &[wgpu::BindGroupEntry {
                binding : 0,
                resource : wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer : &uniform_buffer,
                    offset : 0,
                    size : None,
                }),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label : None, 
            bind_group_layouts : &[&bind_group_layout],
            push_constant_ranges : &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter); 
        let swapchain_format = swapchain_capabilities.formats[0];

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label : None, 
            layout : Some(&pipeline_layout),
            vertex : wgpu::VertexState {
                module : &shader,
                entry_point : "vs_main",
                buffers : &[],
                compilation_options : Default::default(),
            },
            fragment : Some(wgpu::FragmentState {
                module : &shader,
                entry_point : "fs_main",
                compilation_options : Default::default(), 
                targets : &[Some(swapchain_format.into())],
            }),
            primitive : wgpu::PrimitiveState::default(),
            depth_stencil : None, 
            multisample : wgpu::MultisampleState::default(),
            multiview : None,
            cache : None,
        });

        let surface_config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
        surface.configure(&device, &surface_config);

        Self {
            surface,
            surface_config,
            device,
            queue,
            pipeline,
            bind_group,
            uniform_buffer
        }
    }
}
