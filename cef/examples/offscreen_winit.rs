// cef/examples/offscreen_winit.rs

use cef::{args::Args, rc::*, *};
use std::sync::{Arc, LazyLock, Mutex};
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::*, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}
};

// CEF render handler for offscreen rendering
struct OffscreenRenderHandler {
    object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>,
    dimensions: Arc<Mutex<(usize, usize)>>,
    frame_buffer: Arc<Mutex<Option<Vec<u8>>>>,
}

impl OffscreenRenderHandler {
    fn new(frame_buffer: Arc<Mutex<Option<Vec<u8>>>>, dimensions: Arc<Mutex<(usize, usize)>>) -> RenderHandler {
        RenderHandler::new(Self {
            object: std::ptr::null_mut(),
            dimensions,
            frame_buffer,
        })
    }
}

impl WrapRenderHandler for OffscreenRenderHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_render_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for OffscreenRenderHandler {
    fn clone(&self) -> Self {
        unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
        }
        Self {
            object: self.object,
            dimensions: self.dimensions.clone(),
            frame_buffer: self.frame_buffer.clone(),
        }
    }
}

impl Rc for OffscreenRenderHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplRenderHandler for OffscreenRenderHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_render_handler_t {
        self.object.cast()
    }

    fn view_rect(&self, _browser: Option<&mut Browser>, rect: Option<&mut Rect>) {
        if let Some(rect) = rect {
            if let Ok(dimensions) = self.dimensions.lock() {
                *rect = Rect {
                    x: 0,
                    y: 0,
                    width: dimensions.0 as i32,
                    height: dimensions.1 as i32,
                };
            }
        }
    }

    fn on_paint(
        &self,
        _browser: Option<&mut Browser>,
        _type_: PaintElementType,
        _dirty_rect_count: usize,
        _dirty_rects: Option<&Rect>,
        buffer: *const u8,
        width: ::std::os::raw::c_int,
        height: ::std::os::raw::c_int,
    ) {
        let buffer_size = (width * height * 4) as usize;
        unsafe {
            let buffer_slice = std::slice::from_raw_parts(buffer, buffer_size);
            if let Ok(mut frame_buffer) = self.frame_buffer.lock() {
                *frame_buffer = Some(buffer_slice.to_vec());
            }
            dbg!(buffer_slice.iter().take(4).collect::<Vec<_>>());
            dbg!(buffer_slice.last());
        }
    }
}

// CEF App implementation
struct OffscreenApp {
    object: *mut RcImpl<cef_dll_sys::_cef_app_t, Self>,
    frame_buffer: Arc<Mutex<Option<Vec<u8>>>>,
}

impl OffscreenApp {
    fn new(frame_buffer: Arc<Mutex<Option<Vec<u8>>>>) -> App {
        App::new(Self {
            object: std::ptr::null_mut(),
            frame_buffer,
        })
    }
}

impl WrapApp for OffscreenApp {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_app_t, Self>) {
        self.object = object;
    }
}

impl Clone for OffscreenApp {
    fn clone(&self) -> Self {
        unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
        }
        Self {
            object: self.object,
            frame_buffer: self.frame_buffer.clone(),
        }
    }
}

impl Rc for OffscreenApp {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplApp for OffscreenApp {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_app_t {
        self.object.cast()
    }

    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(OffscreenBrowserProcessHandler::new(
            self.frame_buffer.clone(),
        ))
    }
}

// Browser process handler
struct OffscreenBrowserProcessHandler {
    object: *mut RcImpl<cef_dll_sys::cef_browser_process_handler_t, Self>,
    frame_buffer: Arc<Mutex<Option<Vec<u8>>>>,
}

impl OffscreenBrowserProcessHandler {
    fn new(frame_buffer: Arc<Mutex<Option<Vec<u8>>>>) -> BrowserProcessHandler {
        BrowserProcessHandler::new(Self {
            object: std::ptr::null_mut(),
            frame_buffer,
        })
    }
}

impl WrapBrowserProcessHandler for OffscreenBrowserProcessHandler {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_browser_process_handler_t, Self>) {
        self.object = object;
    }
}

impl Clone for OffscreenBrowserProcessHandler {
    fn clone(&self) -> Self {
        unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
        }
        Self {
            object: self.object,
            frame_buffer: self.frame_buffer.clone(),
        }
    }
}

impl Rc for OffscreenBrowserProcessHandler {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplBrowserProcessHandler for OffscreenBrowserProcessHandler {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_browser_process_handler_t {
        self.object.cast()
    }

    fn on_context_initialized(&self) {
        println!("CEF context initialized - ready for offscreen rendering");
    }
}
struct OffscreenClient {
    object: *mut RcImpl<cef_dll_sys::_cef_client_t, Self>,
    render_handler: RenderHandler,
}

impl OffscreenClient {
    fn new(render_handler: RenderHandler) -> Client {
        Client::new(Self {
            object: std::ptr::null_mut(),
            render_handler,
        })
    }
}

impl WrapClient for OffscreenClient {
    fn wrap_rc(&mut self, object: *mut RcImpl<cef_dll_sys::_cef_client_t, Self>) {
        self.object = object;
    }
}

impl Clone for OffscreenClient {
    fn clone(&self) -> Self {
        unsafe {
            let rc_impl = &mut *self.object;
            rc_impl.interface.add_ref();
        }
        Self {
            object: self.object,
            render_handler: self.render_handler.clone(),
        }
    }
}

impl Rc for OffscreenClient {
    fn as_base(&self) -> &cef_dll_sys::cef_base_ref_counted_t {
        unsafe {
            let base = &*self.object;
            std::mem::transmute(&base.cef_object)
        }
    }
}

impl ImplClient for OffscreenClient {
    fn get_raw(&self) -> *mut cef_dll_sys::_cef_client_t {
        self.object.cast()
    }

    fn render_handler(&self) -> Option<RenderHandler> {
        Some(self.render_handler.clone())
    }
}

// Graphics state for wgpu rendering
struct GraphicsState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    render_pipeline: wgpu::RenderPipeline,
    sampler: wgpu::Sampler,
}

impl GraphicsState {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        // Create shader module
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let pos = array(
        // 1st triangle
        vec2f( -1.0,  -1.0),  // center
        vec2f( 1.0,  -1.0),  // right, center
        vec2f( -1.0,  1.0),  // center, top

        // 2nd triangle
        vec2f( -1.0,  1.0),  // center, top
        vec2f( 1.0,  -1.0),  // right, center
        vec2f( 1.0,  1.0),  // right, top
    );
    let xy = pos[vertex_index];
    out.clip_position = vec4f(xy , 0.0, 1.0);
    let coords = (xy/ 2. + 0.5);
    out.tex_coords = vec2f(coords.x, 1. - coords.y);
    // // Generate a fullscreen triangle
    // let x = f32(i32(vertex_index) - 1);
    // let y = f32(i32(vertex_index & 1u) * 2 - 1);

    // out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    // out.tex_coords = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);

    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Test: use texture coordinates as colors to debug
    // return vec4<f32>(in.tex_coords.x, in.tex_coords.y, 0.0, 1.0);
    // Uncomment this line to use CEF texture:
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
"#
                .into(),
            ),
        });

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let mut graphics_state = Self {
            surface,
            device,
            queue,
            config,
            size,
            texture: None,
            bind_group: None,
            render_pipeline,
            sampler,
        };

        // Initialize with a test pattern so we always have something to render
        let initial_data = vec![128u8; 800 * 600 * 4]; // Gray texture
        graphics_state.update_texture(&initial_data, 800, 600);

        graphics_state
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn update_texture(&mut self, data: &[u8], width: u32, height: u32) {



        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("CEF Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.render_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        self.texture = Some(texture);
        self.bind_group = Some(bind_group);
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            if let Some(bind_group) = &self.bind_group {
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw(0..6, 0..1); // Draw 3 vertices for fullscreen triangle
            } else {
                println!("No bind group available - showing clear color only");
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

// Application state
struct WinitApp {
    window: Option<Arc<Window>>,
    graphics_state: Option<GraphicsState>,
    frame_buffer: Arc<Mutex<Option<Vec<u8>>>>,
    browser: Option<Browser>,
    cef_app: Option<App>,
    dimensions: Arc<Mutex<(usize, usize)>>,
    resize: Arc<Mutex<bool>>,
    mouse_position: Option<(i32, i32)>,
}

impl WinitApp {
    fn new() -> Self {
        Self {
            window: None,
            graphics_state: None,
            frame_buffer: Arc::new(Mutex::new(None)),
            browser: None,
            cef_app: None,
            dimensions: Arc::new(Mutex::new((1200, 800))),
            resize: Arc::new(Mutex::new(true)),
            mouse_position: None,
        }
    }
}

impl ApplicationHandler for WinitApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let Ok(dimensions) = self.dimensions.lock() {
            let window = Arc::new(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("CEF Offscreen Rendering")
                            .with_inner_size(winit::dpi::LogicalSize::new(dimensions.0 as u32, dimensions.1 as u32)),
                    )
                    .unwrap(),
            );
            let graphics_state = pollster::block_on(GraphicsState::new(window.clone()));

            self.window = Some(window);
            self.graphics_state = Some(graphics_state);

            println!("Winit window created and ready");
        };
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if let Ok(mut resize) = self.resize.lock() {
                    if let Ok(mut dimensions) = self.dimensions.lock() {
                        *dimensions = (physical_size.width as usize, physical_size.height as usize);
                        *resize = true;
                    }
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position = Some((position.x as i32, position.y as i32));
                if let Some(browser) = &self.browser {
                    browser.host().unwrap().send_mouse_move_event(
                        self.mouse_position
                            .map(|(x, y)| MouseEvent { x, y, modifiers: 0 })
                            .as_ref(),
                        0,
                    );
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(browser) = &self.browser {
                    if let Some(host) = browser.host() {
                        if let Some((x, y)) = self.mouse_position {
                            let mouse_event = MouseEvent { x, y, modifiers: 0 };

                            let cef_button = match button {
                                winit::event::MouseButton::Left => MouseButtonType::from(
                                    cef_dll_sys::cef_mouse_button_type_t::MBT_LEFT,
                                ),
                                winit::event::MouseButton::Right => MouseButtonType::from(
                                    cef_dll_sys::cef_mouse_button_type_t::MBT_RIGHT,
                                ),
                                winit::event::MouseButton::Middle => MouseButtonType::from(
                                    cef_dll_sys::cef_mouse_button_type_t::MBT_MIDDLE,
                                ),
                                _ => MouseButtonType::from(
                                    cef_dll_sys::cef_mouse_button_type_t::MBT_LEFT,
                                ),
                            };

                            let mouse_up = match state {
                                winit::event::ElementState::Pressed => 0,
                                winit::event::ElementState::Released => 1,
                            };

                            host.send_mouse_click_event(
                                Some(&mouse_event),
                                cef_button,
                                mouse_up,
                                1, // click count
                            );

                            println!("Mouse {:?} {:?} at ({}, {})", state, button, x, y);
                        }
                    }
                }
            }
            // WindowEvent::MouseInput { state, button, .. } => {
            //     if let Some(browser) = &self.browser {
            //         dbg!(button);
            //         browser.host().unwrap().send_mouse_click_event(
            //             self.mouse_position
            //                 .map(|(x, y)| MouseEvent { x, y, modifiers: 0 })
            //                 .as_ref(),
            //             MouseButtonType::from(cef_dll_sys::cef_mouse_button_type_t::MBT_RIGHT),
            //             if state == ElementState::Pressed { 0 } else { 1 },
            //             1,
            //         );
            //     }
            // }
            WindowEvent::RedrawRequested => {
                if let Ok(mut resize) = self.resize.lock() {

                    if *resize {
                        if let Some(browser) = &self.browser {
                            browser.host().unwrap().was_resized();
                        }
                    }

                    do_message_loop_work();

                    if let Ok(dimensions) = self.dimensions.lock() {
                        if *resize {
                            if let Some(graphics_state) = &mut self.graphics_state {
                                graphics_state.resize(PhysicalSize::new(dimensions.0 as u32, dimensions.1 as u32));
                            }
                        }
                        if let Ok(frame_buffer) = self.frame_buffer.lock() {
                            let width = dimensions.0;
                            let height = dimensions.1;
                            if let Some(data) = &*frame_buffer {
                                if (width * height * 4) == data.len() {
                                    if let Some(graphics_state) = &mut self.graphics_state {
                                        graphics_state.update_texture(data, width as u32, height as u32);
                                    }
                                }
                            } else {
                                // No CEF frame yet, use test pattern
                                if let Some(graphics_state) = &mut self.graphics_state {
                                    let mut test_data = vec![0u8; width * height * 4];
                                    for y in 0..height {
                                        for x in 0..width {
                                            let idx = (y * width + x) * 4;
                                            test_data[idx] = (x * 255 / width) as u8; // Blue
                                            test_data[idx + 1] = (y * 255 / height) as u8; // Green
                                            test_data[idx + 2] = 255; // Red
                                            test_data[idx + 3] = 255; // Alpha
                                        }
                                    }
                                    graphics_state.update_texture(&test_data, width as u32, height as u32);
                                }
                            }
                        }

                        if let Some(graphics_state) = &mut self.graphics_state {
                            match graphics_state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => {
                                    if let Some(window) = &self.window {
                                        graphics_state.resize(window.inner_size());
                                    }
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    event_loop.exit();
                                }
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }

                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }

                        *resize = false;
                    }
                }
            }
            _ => {}
        }
    }
}

impl WinitApp {
    fn create_cef_browser(&mut self) {
        let render_handler = OffscreenRenderHandler::new(self.frame_buffer.clone(), self.dimensions.clone());
        let mut client = OffscreenClient::new(render_handler);

        let url = CefString::from("file:///home/user/workspace/test.html");

        let mut window_info = WindowInfo::default();
        window_info.windowless_rendering_enabled = 1;

        let mut settings = BrowserSettings::default();
        settings.windowless_frame_rate = 60;
        settings.background_color = 0x0;

        let browser = browser_host_create_browser_sync(
            Some(&window_info),
            Some(&mut client),
            Some(&url),
            Some(&settings),
            Option::<&mut DictionaryValue>::None,
            Option::<&mut RequestContext>::None,
        );

        dbg!(browser.is_some());
        self.browser = browser;
    }
}

fn main() {
    // Initialize CEF
    #[cfg(target_os = "macos")]
    let _loader = {
        let loader = library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), false);
        assert!(loader.load());
        loader
    };

    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);

    let args = Args::new();
    let cmd = args.as_cmd_line().unwrap();

    let switch = CefString::from("type");
    let is_browser_process = cmd.has_switch(Some(&switch)) != 1;

    if !is_browser_process {
        let process_type = CefString::from(&cmd.switch_value(Some(&switch)));
        println!("launch process {process_type}");
        let ret = execute_process(
            Some(args.as_main_args()),
            Option::<&mut App>::None,
            std::ptr::null_mut(),
        );
        assert!(ret >= 0, "cannot execute non-browser process");
        return;
    }

    let mut settings = Settings::default();
    settings.windowless_rendering_enabled = 1;
    settings.multi_threaded_message_loop = 0;

    // Create shared frame buffer for CEF and winit
    let frame_buffer = Arc::new(Mutex::new(None));
    let mut cef_app = OffscreenApp::new(frame_buffer.clone());

    assert_eq!(
        initialize(
            Some(args.as_main_args()),
            Some(&settings),
            Some(&mut cef_app),
            std::ptr::null_mut()
        ),
        1
    );

    // Start winit event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut winit_app = WinitApp::new();
    winit_app.frame_buffer = frame_buffer;
    winit_app.cef_app = Some(cef_app);
    winit_app.create_cef_browser();

    event_loop.run_app(&mut winit_app).unwrap();

    shutdown();
}
