use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::event::Event;
use winit::event::MouseButton;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

use wgpu_text::glyph_brush::Section as TextSection;
use wgpu_text::glyph_brush::Text;
use wgpu_text::BrushBuilder;
use wgpu_text::TextBrush; 

use std::borrow::Cow; 

use encase::ShaderType; 


#[derive(Debug, ShaderType)]
struct AppState {
    pub cursor_pos_x : f32, 
    pub cursor_pos_y : f32,
    pub zoom : f32,
    pub max_iterations : u32
}

impl AppState {
    fn as_wgsl_bytes(&self) -> encase::internal::Result<Vec<u8>> {
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(self)?;
        Ok(buffer.into_inner())
    }
}

impl Default for AppState {
    fn default() -> Self {
        AppState {
            cursor_pos_x: -1.5,
            cursor_pos_y: -1.0,
            zoom: 1.2,
            max_iterations: 128,
        }
    }
}

pub async fn start() {

    let event_loop = EventLoop::new().unwrap(); 
    let mut builder = winit::window::WindowBuilder::new()
        .with_title("Fractal Renderer")
        .with_inner_size(PhysicalSize::new(1920, 1080));

    let window = builder.build(&event_loop).unwrap();

    let mut size = window.inner_size(); 
    size.width = size.width.max(1);
    size.height = size.height.max(1);

    let instance = wgpu::Instance::default();
    let surface = instance.create_surface(&window).unwrap();

    let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface),
    }).await.expect("Failed to find an appropriate adapter!");


    let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
        label : None, 
        required_features : wgpu::Features::empty(),
        required_limits : wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
        memory_hints : wgpu::MemoryHints::MemoryUsage,
    },
    None
    ).await.expect("Failed to create device");

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label : None,
        source : wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/shader.wgsl"))),
    });

    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label : None, 
        size : std::mem::size_of::<AppState>() as u64,
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

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

    let mut config = surface.get_default_config(&adapter, size.width, size.height).unwrap();
    surface.configure(&device, &config);

    // Setting up text brush
    let mut brush = BrushBuilder::using_font_bytes(include_bytes!("fonts/Roboto/Roboto-Regular.ttf")).unwrap()
        .build(&device, config.width, config.height, config.format);

    let version = env!("CARGO_PKG_VERSION"); 
    let title_text = format!("Fractal Renderer {}", version); 
    let text_section_title = TextSection::default().add_text(Text::new(&title_text)
        .with_color([1.0, 1.0, 1.0, 1.0])
        .with_scale(24.0)
    ).with_screen_position((10.0, 10.0)); 

    
    let window = &window; 

    let mut app_state = Some(AppState::default()); 

    let mut last_frame_time : std::time::Instant = std::time::Instant::now();
    let mut fps : f64 = 0.0; 
    let mut text_update_time_seconds : f64 = 1.0; 
    let mut last_text_update_time : std::time::Instant = std::time::Instant::now();

    let mut current_cursor_position : (f64, f64) = (0.0, 0.0); 
    let mut last_cursor_position : (f64, f64) = (0.0, 0.0); 

    let mut is_pressed : bool = false; 
    
    event_loop.run(move |event, target| {

        // A hacky way of making the current scope borrow all the resources, this is needed to make
        // sure that all resources are properly cleaned up between the frames. 
        let _ = (&instance, &adapter, &shader, &pipeline_layout);

        if let Event::WindowEvent { window_id: _, event } = event {
            match event {
                
                WindowEvent::KeyboardInput {

                    event:
                        winit::event::KeyEvent {
                            physical_key : winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Backspace),
                            state : winit::event::ElementState::Pressed,
                            repeat : false,
                            ..
                        },
                    ..
                } => {
                        app_state.as_mut().unwrap().max_iterations = (app_state.as_ref().unwrap().max_iterations as i32 / 2).max(2).min(8192) as u32;
                }

                WindowEvent::KeyboardInput {

                    event:
                        winit::event::KeyEvent {
                            physical_key : winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space),
                            state : winit::event::ElementState::Pressed,
                            repeat : false,
                            ..
                        },
                    ..
                } => {
                        app_state.as_mut().unwrap().max_iterations = (app_state.as_ref().unwrap().max_iterations as i32 * 2).max(2).min(8192) as u32;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    current_cursor_position = (position.x, position.y); 
                    
                    if is_pressed {
                        let position_delta = (current_cursor_position.0 - last_cursor_position.0, current_cursor_position.1 - last_cursor_position.1);

                        app_state.as_mut().unwrap().cursor_pos_x = app_state.as_ref().unwrap().cursor_pos_x + 2.0 * (position_delta.0 * app_state.as_ref().unwrap().zoom as f64 / config.width as f64) as f32;
                        app_state.as_mut().unwrap().cursor_pos_y = app_state.as_ref().unwrap().cursor_pos_y + 2.0 * (position_delta.1 * app_state.as_ref().unwrap().zoom as f64 / config.height as f64) as f32;
                    }

                    last_cursor_position = (position.x, position.y); 

                }
                WindowEvent::MouseInput { state, button, .. } => {
                    
                    match button {
                        MouseButton::Left => {
                            
                            is_pressed = state == ElementState::Pressed; 
                        }
                        _ => {}
                    }
                }
                WindowEvent::MouseWheel { delta, ..  } => {
                    match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => {
                            
                            let dir : f32 = if y > 0.0 {1.0} else {-1.0};

                            let new_value : f32 = app_state.as_ref().unwrap().zoom + 0.1 * dir * app_state.as_ref().unwrap().zoom; 
                            app_state.as_mut().unwrap().zoom = new_value;
                        }
                        _=> {}
                    }
                }
                WindowEvent::Resized(new_size) => {
                    config.width = new_size.width.max(1); 
                    config.height = new_size.height.max(1); 

                    surface.configure(&device, &config);

                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    
                    let time_between_frames_micros : f64 = last_frame_time.elapsed().as_micros() as f64; 
                
                    last_frame_time = std::time::Instant::now(); 
                    
                    if last_text_update_time.elapsed().as_secs_f64() > text_update_time_seconds {

                        last_text_update_time = std::time::Instant::now(); 
                        fps = 1000.0 * 1000.0 / time_between_frames_micros; 
                    }

                    let fps_text = format!("FPS: {:.2}", fps);
                    let text_section_fps = TextSection::default().add_text(Text::new(&fps_text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(24.0)
                    ).with_screen_position((10.0, 35.0)); 

                    let state_ref = app_state.as_ref().unwrap(); 
                    
                    let x_pos_text = format!("x: {}", state_ref.cursor_pos_x);
                    let text_section_x_pos = TextSection::default().add_text(Text::new(&x_pos_text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(24.0)
                    ).with_screen_position((10.0, 65.0)); 
                    
                    let y_pos_text = format!("y: {}", state_ref.cursor_pos_y);
                    let text_section_y_pos = TextSection::default().add_text(Text::new(&y_pos_text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(24.0)
                    ).with_screen_position((10.0, 95.0)); 

                    let max_iterations_text = format!("max iterations: {}", state_ref.max_iterations);
                    let text_section_max_iterations = TextSection::default().add_text(Text::new(&max_iterations_text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(24.0)
                    ).with_screen_position((10.0, 120.0)); 

                    let zoom_text = format!("zoom: {}", state_ref.zoom);
                    let text_section_zoom = TextSection::default().add_text(Text::new(&zoom_text)
                        .with_color([1.0, 1.0, 1.0, 1.0])
                        .with_scale(24.0)
                    ).with_screen_position((10.0, 145.0)); 

                    let frame = surface.get_current_texture().expect("Failed to aquire swapchain texture");
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label : None,
                    });
                    
                    queue.write_buffer(&uniform_buffer, 0, &state_ref.as_wgsl_bytes().expect("Could not translate to WGSL bytes."));

                    // The recording of commands to the command buffer in the render pass needs to
                    // end before it is submitted to the queue. The recording ends implicitly when it exits
                    // its scope. Therefore, the underlying code is contained in its own scope. Each render pass recording must
                    // therefore happen in its respective scope. 
                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label : None,
                                color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                                    view : &view,
                                    resolve_target : None,
                                    ops : wgpu::Operations {
                                        load : wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                                        store : wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment : None,
                                timestamp_writes : None,
                                occlusion_query_set : None,
                        });
                        render_pass.set_pipeline(&render_pipeline); 
                        
                        render_pass.set_bind_group(0, &bind_group, &[]);
                        render_pass.draw(0..6, 0..1); 
                    }

                    // Queueing all the text section on the brush
                    brush.queue(&device, &queue, [&text_section_title, &text_section_fps, &text_section_x_pos, &text_section_y_pos, &text_section_max_iterations, &text_section_zoom]).unwrap(); 

                    // Another renderpass, exclusively for the foreground text
                    {

                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                label : None,
                                color_attachments : &[Some(wgpu::RenderPassColorAttachment {
                                    view : &view,
                                    resolve_target : None,
                                    ops : wgpu::Operations {
                                        load : wgpu::LoadOp::Load,
                                        store : wgpu::StoreOp::Store,
                                    },
                                })],
                                depth_stencil_attachment : None,
                                timestamp_writes : None,
                                occlusion_query_set : None,
                        });
                        render_pass.set_pipeline(&render_pipeline); 
                    
                        brush.draw(&mut render_pass); 
                    }

                    queue.submit(Some(encoder.finish())); 
                    frame.present(); 

                    // Requesting another redraw when we are done drawing this frame
                    window.request_redraw();
                }
                WindowEvent::CloseRequested => target.exit(),
                _ => {}
            };
        }
    }).unwrap();
}
