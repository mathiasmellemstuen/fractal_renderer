use winit::dpi::PhysicalSize;
use winit::event::ElementState;
use winit::event::Event;
use winit::event::MouseButton;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;

use wgpu_text::BrushBuilder;

use std::sync::Arc;

use crate::text_section::create_new_text_section;
use crate::fractal_config::FractalConfig;
use crate::wgpu_context::WGPUContext; 

pub fn start_app() {

    let event_loop = EventLoop::new().unwrap(); 
    let builder = winit::window::WindowBuilder::new()
        .with_title("Fractal Renderer")
        .with_inner_size(PhysicalSize::new(1920, 1080));

    let window = Arc::new(builder.build(&event_loop).unwrap());

    let mut wgpu_context = WGPUContext::setup(window.clone());

    // Setting up text brush
    let mut brush = BrushBuilder::using_font_bytes(include_bytes!("fonts/Roboto/Roboto-Regular.ttf")).unwrap()
        .build(&wgpu_context.device, wgpu_context.surface_config.width, wgpu_context.surface_config.height, wgpu_context.surface_config.format);

    // Creating the title text section
    let version = env!("CARGO_PKG_VERSION"); 
    let title_text = format!("Fractal Renderer {}", version); 
    let text_section_title = create_new_text_section(&title_text, (10.0, 10.0));
    let text_update_time_seconds : f64 = 1.0; 

    let mut fractal_config = Some(FractalConfig::default()); 

    let mut last_frame_time : std::time::Instant = std::time::Instant::now();
    let mut last_text_update_time : std::time::Instant = std::time::Instant::now();

    let mut fps : f64 = 0.0; 
    let mut last_cursor_position : (f64, f64) = (0.0, 0.0); 
    let mut is_pressed : bool = false; 
    
    event_loop.run(move |event, target| {

        if let Event::WindowEvent { window_id: _, event } = event {
            match event {
                
                // Listening for an event where the backspace key is pressed
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
                        fractal_config.as_mut().unwrap().max_iterations = (fractal_config.as_ref().unwrap().max_iterations as i32 / 2).max(2).min(8192) as u32;
                }

                // Listening for an event where the space key is pressed
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
                        fractal_config.as_mut().unwrap().max_iterations = (fractal_config.as_ref().unwrap().max_iterations as i32 * 2).max(2).min(8192) as u32;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let position_delta = (position.x - last_cursor_position.0, position.y - last_cursor_position.1);
                    
                    if is_pressed {
                        fractal_config.as_mut().unwrap().cursor_pos_x = fractal_config.as_ref().unwrap().cursor_pos_x + 2.0 * (position_delta.0 * fractal_config.as_ref().unwrap().zoom as f64 / wgpu_context.surface_config.width as f64) as f32;
                        fractal_config.as_mut().unwrap().cursor_pos_y = fractal_config.as_ref().unwrap().cursor_pos_y + 2.0 * (position_delta.1 * fractal_config.as_ref().unwrap().zoom as f64 / wgpu_context.surface_config.height as f64) as f32;
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

                            let new_value : f32 = fractal_config.as_ref().unwrap().zoom + 0.1 * dir * fractal_config.as_ref().unwrap().zoom as f32; 
                            fractal_config.as_mut().unwrap().zoom = new_value;
                        }
                        _=> {}
                    }
                }
                WindowEvent::Resized(new_size) => {
                    wgpu_context.surface_config.width = new_size.width.max(1); 
                    wgpu_context.surface_config.height = new_size.height.max(1); 

                    fractal_config.as_mut().unwrap().resolution_x = wgpu_context.surface_config.width as f32; 
                    fractal_config.as_mut().unwrap().resolution_y = wgpu_context.surface_config.height as f32; 

                    wgpu_context.surface.configure(&wgpu_context.device, &wgpu_context.surface_config);

                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    
                    // Calculating fps
                    let time_between_frames_micros : f64 = last_frame_time.elapsed().as_micros() as f64; 
                    last_frame_time = std::time::Instant::now();

                    if last_text_update_time.elapsed().as_secs_f64() > text_update_time_seconds {

                        last_text_update_time = std::time::Instant::now(); 
                        fps = 1000.0 * 1000.0 / time_between_frames_micros; 
                    }

                    // Aquire view descriptor and swapchain texture to create encoder (i.e. command
                    // buffer). This will later be used to create the two render-passes in the
                    // application.
                    let frame = wgpu_context.surface.get_current_texture().expect("Failed to aquire swapchain texture");
                    let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                    let mut encoder = wgpu_context.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label : None,
                    });
                    

                    // Creating a reference to the fractal_config and writing it to the uniform buffer 
                    let fractal_config_ref = fractal_config.as_ref().unwrap(); 
                    wgpu_context.queue.write_buffer(&wgpu_context.uniform_buffer, 0, &fractal_config_ref.as_wgsl_bytes().expect("Could not translate to WGSL bytes."));

                    // The recording of commands to the command buffer in the render pass needs to
                    // end before it is submitted to the queue. The recording ends implicitly when it exits
                    // its scope. Therefore, the underlying code is contained within its own scope. Each render pass recording must
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

                        render_pass.set_pipeline(&wgpu_context.pipeline); 
                        
                        // Set bind group to binding 0, binding the uniform buffer with
                        // fractal_config data to binding 0, set 0 in the shader
                        render_pass.set_bind_group(0, &wgpu_context.bind_group, &[]);

                        // Making a draw call with 6 vertices (2x triangles). This will draw the
                        // mandelbrot set, using the mandelbrot shader spesified in the render
                        // pipeline.
                        render_pass.draw(0..6, 0..1); 
                    }

                    // Creating GUI text sections 
                    let fps_text = format!("FPS: {:.2}", fps);
                    let text_section_fps  = create_new_text_section(&fps_text, (10.0, 35.0)); 
                    
                    let x_pos_text = format!("Re(z): {}", fractal_config_ref.cursor_pos_x);
                    let text_section_x_pos = create_new_text_section(&x_pos_text, (10.0, 65.0));

                    let y_pos_text = format!("Im(z): {}", fractal_config_ref.cursor_pos_y);
                    let text_section_y_pos = create_new_text_section(&y_pos_text, (10.0, 95.0)); 

                    let max_iterations_text = format!("max iterations: {}", fractal_config_ref.max_iterations);

                    let text_section_max_iterations = create_new_text_section(&max_iterations_text, (10.0, 120.0)); 

                    let zoom_text = format!("zoom: {}", fractal_config_ref.zoom);
                    let text_section_zoom = create_new_text_section(&zoom_text, (10.0, 145.0)); 

                    // Queueing all the text section on the wgpu_text brush
                    brush.queue(&wgpu_context.device, &wgpu_context.queue,
                        [
                            &text_section_title,
                            &text_section_fps,
                            &text_section_x_pos,
                            &text_section_y_pos,
                            &text_section_max_iterations,
                            &text_section_zoom

                        ]).unwrap(); 

                    // Another renderpass, exclusively used for the foreground text on top of the
                    // previous renderpass.
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
                    
                        brush.draw(&mut render_pass); 
                    }
                    
                    // Submitting renderpasses in commandbuffer / encoder, and calling present for
                    // scheduling the next step in swapchain.
                    wgpu_context.queue.submit(Some(encoder.finish())); 
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
