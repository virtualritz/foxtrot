use glm::Vec2;
use nalgebra_glm as glm;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, MouseScrollDelta, WindowEvent},
    keyboard::{Key, ModifiersState},
};

use crate::{backdrop::Backdrop, camera::Camera, model::Model};
use triangulate::mesh::Mesh;

pub struct App<'a> {
    start_time: std::time::SystemTime,

    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    loader: Option<std::thread::JoinHandle<Mesh>>,
    model: Option<Model>,
    backdrop: Backdrop,
    camera: Camera,

    depth: (wgpu::Texture, wgpu::TextureView),
    size: PhysicalSize<u32>,

    modifiers: ModifiersState,

    first_frame: bool,
}

pub enum Reply {
    Continue,
    Redraw,
    Quit,
}

impl<'a> App<'a> {
    pub fn new(
        start_time: std::time::SystemTime,
        size: PhysicalSize<u32>,
        adapter: wgpu::Adapter,
        surface: wgpu::Surface<'a>,
        device: wgpu::Device,
        queue: wgpu::Queue,
        loader: std::thread::JoinHandle<Mesh>,
    ) -> Self {
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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth = Self::rebuild_depth_(size, &device);
        let backdrop = Backdrop::new(&device, surface_format);

        Self {
            start_time,

            depth,
            backdrop,
            config,
            loader: Some(loader),
            model: None,
            camera: Camera::new(size.width as f32, size.height as f32),
            surface,
            device,
            queue,
            size,

            modifiers: ModifiersState::empty(),

            first_frame: true,
        }
    }

    pub fn device_event(&mut self, e: DeviceEvent) {
        if let DeviceEvent::MouseWheel {
            delta: MouseScrollDelta::PixelDelta(p),
        } = e
        {
            self.camera.mouse_scroll(p.y as f32);
        }
    }

    pub fn window_event(&mut self, e: WindowEvent) -> Reply {
        match e {
            WindowEvent::Resized(size) => {
                self.resize(size);
                Reply::Redraw
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // Scale factor changes don't directly affect the size anymore in winit 0.30
                Reply::Continue
            }
            WindowEvent::CloseRequested => Reply::Quit,
            WindowEvent::ModifiersChanged(m) => {
                self.modifiers = m.state();
                Reply::Continue
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if self.modifiers.super_key() && event.logical_key == Key::Character("q".into()) {
                    Reply::Quit
                } else {
                    Reply::Continue
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                use ElementState::*;
                match state {
                    Pressed => self.camera.mouse_pressed(button),
                    Released => self.camera.mouse_released(button),
                }
                Reply::Continue
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.camera
                    .mouse_move(Vec2::new(position.x as f32, position.y as f32));
                Reply::Redraw
            }
            WindowEvent::MouseWheel { delta, .. } => {
                if let MouseScrollDelta::LineDelta(_, verti) = delta {
                    self.camera.mouse_scroll(verti * 10.0);
                }
                Reply::Redraw
            }
            _ => Reply::Continue,
        }
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width > 0 && size.height > 0 {
            self.size = size;
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth = Self::rebuild_depth_(size, &self.device);
            self.camera.set_size(size.width as f32, size.height as f32);
        }
    }

    fn rebuild_depth_(
        size: PhysicalSize<u32>,
        device: &wgpu::Device,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some("depth tex"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let tex = device.create_texture(&desc);
        let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        (tex, view)
    }

    // Redraw the GUI, returning true if the model was not drawn (which means
    // that the parent loop should keep calling redraw to force model load)
    pub fn redraw(&mut self) -> bool {
        let output = self
            .surface
            .get_current_texture()
            .expect("Failed to acquire next surface texture");
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        self.backdrop.draw(&view, &self.depth.1, &mut encoder);
        if let Some(model) = &self.model {
            model.draw(
                &self.camera,
                &self.queue,
                &view,
                &self.depth.1,
                &mut encoder,
            );
        }
        let drew_model = self.model.is_some();
        self.queue.submit(Some(encoder.finish()));
        output.present();

        if drew_model && self.first_frame {
            let end = std::time::SystemTime::now();
            let dt = end.duration_since(self.start_time).expect("dt < 0??");
            println!("First redraw at {:?}", dt);
            self.first_frame = false;
        }

        // This is very awkward, but WebGPU doesn't actually do the GPU work
        // until after a queue is submitted, so we don't wait to wait for
        // the model until the _second_ frame.
        if !self.first_frame && self.model.is_none() {
            let mesh = self
                .loader
                .take()
                .unwrap()
                .join()
                .expect("Failed to load mesh");
            let model = Model::new(
                &self.device,
                self.config.format,
                &mesh.verts,
                &mesh.triangles,
            );
            self.model = Some(model);
            self.camera.fit_verts(&mesh.verts);
            self.first_frame = true;
        } else {
            self.first_frame = false;
        }

        !drew_model
    }
}
