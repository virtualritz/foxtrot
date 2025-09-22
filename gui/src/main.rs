use std::sync::Arc;
use std::time::SystemTime;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent as WinitWindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub(crate) mod app;
pub(crate) mod backdrop;
pub(crate) mod camera;
pub(crate) mod model;

use crate::app::App;
use triangulate::mesh::Mesh;

struct GuiApp {
    start: SystemTime,
    loader: Option<std::thread::JoinHandle<Mesh>>,
    window: Option<Arc<Window>>,
    app: Option<App<'static>>,
}

impl GuiApp {
    fn new(start: SystemTime, loader: std::thread::JoinHandle<Mesh>) -> Self {
        Self {
            start,
            loader: Some(loader),
            window: None,
            app: None,
        }
    }
}

impl ApplicationHandler for GuiApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = std::sync::Arc::new(
                event_loop
                    .create_window(winit::window::WindowAttributes::default().with_title("Foxtrot"))
                    .expect("Failed to create window"),
            );

            // Initialize wgpu and create app
            let size = window.inner_size();
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
            let surface = instance
                .create_surface(window.clone())
                .expect("Failed to create surface");

            let adapter =
                pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                }))
                .expect("Failed to find an appropriate adapter");

            let (device, queue) =
                pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                    trace: Default::default(),
                }))
                .expect("Failed to create device");

            let app = App::new(
                self.start,
                size,
                adapter,
                surface,
                device,
                queue,
                self.loader.take().unwrap(),
            );

            self.app = Some(app);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WinitWindowEvent,
    ) {
        use app::Reply;

        if let Some(app) = &mut self.app
            && let Some(window) = &self.window
        {
            match app.window_event(event) {
                Reply::Continue => (),
                Reply::Quit => event_loop.exit(),
                Reply::Redraw => {
                    if app.redraw() {
                        window.request_redraw();
                    }
                }
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let Some(app) = &mut self.app {
            app.device_event(event);
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(app) = &mut self.app
            && let Some(window) = &self.window
            && app.redraw()
        {
            window.request_redraw();
        }
    }
}

fn main() {
    let start = SystemTime::now();
    env_logger::init();

    let matches = clap::App::new("gui")
        .author("Matt Keeter <matt@formlabs.com>")
        .about("Renders a STEP file")
        .arg(
            clap::Arg::with_name("input")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let input = matches
        .value_of("input")
        .expect("Could not get input file")
        .to_owned();

    // Load and triangulate the STEP file first
    println!("Loading mesh!");
    use step::step_file::StepFile;
    use triangulate::triangulate::triangulate;

    let data = std::fs::read(input).expect("Could not open file");
    let flat = StepFile::strip_flatten(&data);
    let step = StepFile::parse(&flat);
    let (mesh, _stats) = triangulate(&step);

    // Check if the mesh is empty
    if mesh.verts.is_empty() || mesh.triangles.is_empty() {
        eprintln!("Error: The STEP file produced an empty mesh (no vertices or triangles).");
        eprintln!("This may indicate an unsupported geometry type or parsing issue.");
        std::process::exit(1);
    }

    println!(
        "Mesh loaded: {} vertices, {} triangles",
        mesh.verts.len(),
        mesh.triangles.len()
    );

    // Start the mesh in a thread for the GUI to pick up later
    let loader = std::thread::spawn(move || mesh);

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = GuiApp::new(start, loader);
    let _ = event_loop.run_app(&mut app);
}
