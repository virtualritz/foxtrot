use three_d::*;

/// 3D renderer for STEP models using three-d
pub struct StepRenderer {
    context: Context,
    camera: Camera,
    target_size: (u32, u32),
    meshes: Vec<Gm<Mesh, ColorMaterial>>,
    control: CameraControl,
}

impl StepRenderer {
    /// Create a new renderer
    pub fn new(context: Context) -> Self {
        let camera = Camera::new_perspective(
            Viewport::new_at_origo(800, 600),
            vec3(5.0, 5.0, 5.0),
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, 1.0),
            degrees(45.0),
            0.1,
            1000.0,
        );

        let control = CameraControl::new(0.5, 0.1, 0.1);

        Self {
            context,
            camera,
            target_size: (800, 600),
            meshes: Vec::new(),
            control,
        }
    }

    /// Get the context
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Resize the viewport
    pub fn resize(&mut self, width: u32, height: u32) {
        self.target_size = (width, height);
        self.camera.set_viewport(Viewport::new_at_origo(width, height));
    }

    /// Update camera based on mouse events
    pub fn handle_events(&mut self, events: &mut [Event]) -> bool {
        self.control.handle_events(&mut self.camera, events)
    }

    /// Add a mesh to the scene
    pub fn add_mesh(&mut self, positions: &[f32], indices: &[u32], color: Srgba) {
        if positions.is_empty() || indices.is_empty() {
            return;
        }

        // Create mesh
        let mut cpu_mesh = CpuMesh::default();
        cpu_mesh.positions = Positions::F32(
            positions
                .chunks(3)
                .map(|p| vec3(p[0], p[1], p[2]))
                .collect(),
        );
        cpu_mesh.indices = Indices::U32(indices.to_vec());

        // Compute normals
        cpu_mesh.compute_normals();

        // Create GPU mesh
        let mesh = Gm::new(
            Mesh::new(&self.context, &cpu_mesh),
            ColorMaterial {
                color,
                ..Default::default()
            },
        );

        self.meshes.push(mesh);
    }

    /// Clear all meshes
    pub fn clear_meshes(&mut self) {
        self.meshes.clear();
    }

    /// Get mesh count for display
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Get camera position for display
    pub fn camera_position(&self) -> Vec3 {
        self.camera.position()
    }

    /// Get camera target for display
    pub fn camera_target(&self) -> Vec3 {
        self.camera.target()
    }
}

/// Simple camera control
pub struct CameraControl {
    rotate_speed: f32,
    zoom_speed: f32,
    pan_speed: f32,
    dragging: bool,
    last_mouse_pos: Option<(f32, f32)>,
}

impl CameraControl {
    pub fn new(rotate_speed: f32, zoom_speed: f32, pan_speed: f32) -> Self {
        Self {
            rotate_speed,
            zoom_speed,
            pan_speed,
            dragging: false,
            last_mouse_pos: None,
        }
    }

    pub fn handle_events(&mut self, camera: &mut Camera, events: &mut [Event]) -> bool {
        let mut changed = false;

        for event in events.iter_mut() {
            match event {
                Event::MousePress {
                    button: MouseButton::Left,
                    position,
                    ..
                } => {
                    self.dragging = true;
                    self.last_mouse_pos = Some((position.x as f32, position.y as f32));
                }
                Event::MouseRelease {
                    button: MouseButton::Left,
                    ..
                } => {
                    self.dragging = false;
                    self.last_mouse_pos = None;
                }
                Event::MouseMotion { position, .. } => {
                    if self.dragging {
                        if let Some(last_pos) = self.last_mouse_pos {
                            let delta_x = (position.x as f32 - last_pos.0) * self.rotate_speed;
                            let delta_y = (position.y as f32 - last_pos.1) * self.rotate_speed;
                            let target = camera.target();

                            camera.rotate_around_with_fixed_up(
                                target,
                                delta_x,
                                delta_y,
                            );
                            changed = true;
                        }
                        self.last_mouse_pos = Some((position.x as f32, position.y as f32));
                    }
                }
                Event::MouseWheel { delta, .. } => {
                    let target = camera.target();
                    camera.zoom_towards(target, delta.1 * self.zoom_speed, 0.1, 100.0);
                    changed = true;
                }
                _ => {}
            }
        }

        changed
    }
}
