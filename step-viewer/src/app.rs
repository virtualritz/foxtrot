use std::path::PathBuf;
use std::sync::Arc;
use three_d::*;

/// Main application state for the STEP viewer
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct StepViewerApp {
    /// Path to the currently loaded STEP file
    #[serde(skip)]
    file_path: Option<PathBuf>,

    /// Loaded STEP model data
    #[serde(skip)]
    step_data: Option<StepData>,

    /// 3D renderer (not serialized)
    #[serde(skip)]
    renderer: Option<crate::renderer::StepRenderer>,

    /// three-d headless context
    #[serde(skip)]
    headless_context: Option<Arc<Context>>,

    /// UI state
    ui_state: UiState,

    /// Error message to display
    #[serde(skip)]
    error_message: Option<String>,

    /// Camera controls
    camera_distance: f32,
    camera_rotation: (f32, f32),
}

/// Stores loaded STEP file data
struct StepData {
    /// Raw STEP entities
    entities: Vec<StepEntity>,

    /// File metadata
    metadata: StepMetadata,
}

/// Represents a single STEP entity
#[derive(Clone)]
struct StepEntity {
    id: usize,
    entity_type: String,
    record_name: String,
}

/// STEP file metadata
#[derive(Default)]
struct StepMetadata {
    file_description: String,
    file_name: String,
    schema_name: String,
    entity_count: usize,
}

/// UI state tracking
#[derive(serde::Deserialize, serde::Serialize)]
struct UiState {
    show_entity_list: bool,
    show_metadata: bool,
    background_color: [f32; 3],
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            show_entity_list: true,
            show_metadata: true,
            background_color: [0.2, 0.2, 0.2],
        }
    }
}

impl Default for StepViewerApp {
    fn default() -> Self {
        Self {
            file_path: None,
            step_data: None,
            renderer: None,
            headless_context: None,
            ui_state: UiState::default(),
            error_message: None,
            camera_distance: 10.0,
            camera_rotation: (45.0, 30.0),
        }
    }
}

impl StepViewerApp {
    /// Create a new STEP viewer application
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        // We'll initialize the renderer when we load a file
        app
    }

    /// Load a STEP file from the given path
    fn load_step_file(&mut self, path: PathBuf) {
        log::info!("Loading STEP file: {:?}", path);

        match Self::parse_step_file(&path) {
            Ok(data) => {
                self.file_path = Some(path);
                self.step_data = Some(data);
                self.error_message = None;
                self.create_placeholder_geometry();
                log::info!("STEP file loaded successfully");
            }
            Err(e) => {
                self.error_message = Some(format!("Error loading STEP file: {}", e));
                log::error!("Failed to load STEP file: {}", e);
            }
        }
    }

    /// Create placeholder geometry for visualization
    /// In a real implementation, this would parse STEP geometry
    fn create_placeholder_geometry(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.clear_meshes();

            // Create a simple cube as placeholder
            let positions: Vec<f32> = vec![
                // Front face
                -1.0, -1.0,  1.0,
                 1.0, -1.0,  1.0,
                 1.0,  1.0,  1.0,
                -1.0,  1.0,  1.0,
                // Back face
                -1.0, -1.0, -1.0,
                -1.0,  1.0, -1.0,
                 1.0,  1.0, -1.0,
                 1.0, -1.0, -1.0,
            ];

            let indices: Vec<u32> = vec![
                0, 1, 2,  0, 2, 3,  // Front
                4, 5, 6,  4, 6, 7,  // Back
                0, 4, 7,  0, 7, 1,  // Bottom
                3, 2, 6,  3, 6, 5,  // Top
                0, 3, 5,  0, 5, 4,  // Left
                1, 7, 6,  1, 6, 2,  // Right
            ];

            renderer.add_mesh(&positions, &indices, Srgba::new(100, 150, 200, 255));
        }
    }

    /// Parse a STEP file and extract data
    fn parse_step_file(path: &PathBuf) -> Result<StepData, String> {
        // Read the STEP file
        let file_content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Parse using ruststep
        let exchange = ruststep::parser::parse(&file_content)
            .map_err(|e| format!("Failed to parse STEP file: {:?}", e))?;

        // Extract metadata from header (which is a Vec<Record>)
        let file_description = exchange
            .header
            .iter()
            .find(|r| r.name == "FILE_DESCRIPTION")
            .map(|r| format!("{:?}", r.parameter))
            .unwrap_or_else(|| "No description".to_string());

        let file_name = exchange
            .header
            .iter()
            .find(|r| r.name == "FILE_NAME")
            .map(|r| format!("{:?}", r.parameter))
            .unwrap_or_else(|| path.file_name().unwrap_or_default().to_string_lossy().to_string());

        let schema_name = exchange
            .header
            .iter()
            .find(|r| r.name == "FILE_SCHEMA")
            .map(|r| format!("{:?}", r.parameter))
            .unwrap_or_else(|| "Unknown schema".to_string());

        // Count total entities across all data sections
        let entity_count: usize = exchange.data.iter().map(|section| section.entities.len()).sum();

        let metadata = StepMetadata {
            file_description,
            file_name,
            schema_name,
            entity_count,
        };

        // Extract entities from all data sections
        let entities: Vec<StepEntity> = exchange
            .data
            .iter()
            .flat_map(|section| section.entities.iter())
            .enumerate()
            .map(|(idx, entity)| StepEntity {
                id: idx,
                entity_type: "entity".to_string(),
                record_name: format!("{:?}", entity),
            })
            .collect();

        log::info!(
            "Parsed {} entities from STEP file",
            metadata.entity_count
        );

        Ok(StepData {
            entities,
            metadata,
        })
    }

    /// Show the file menu
    fn show_menu(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui.button("Open STEP file...").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("STEP files", &["stp", "step"])
                                .pick_file()
                            {
                                self.load_step_file(path);
                            }
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }

                    #[cfg(target_arch = "wasm32")]
                    {
                        ui.label("File operations not supported in web version");
                    }
                });

                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.ui_state.show_entity_list, "Entity List");
                    ui.checkbox(&mut self.ui_state.show_metadata, "Metadata");
                });

                ui.add_space(16.0);
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
    }

    /// Show the entity list panel
    fn show_entity_list(&mut self, ctx: &egui::Context) {
        if !self.ui_state.show_entity_list {
            return;
        }

        egui::SidePanel::left("entity_list")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Entities");
                ui.separator();

                if let Some(data) = &self.step_data {
                    ui.label(format!("Total: {}", data.entities.len()));
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for entity in &data.entities {
                            ui.horizontal(|ui| {
                                ui.label(format!("#{}: {}", entity.id, entity.record_name));
                            });
                        }
                    });
                } else {
                    ui.label("No file loaded");
                    ui.separator();
                    ui.label("Use File → Open STEP file... to load a model");
                }
            });
    }

    /// Show the metadata panel
    fn show_metadata(&mut self, ctx: &egui::Context) {
        if !self.ui_state.show_metadata {
            return;
        }

        egui::SidePanel::right("metadata")
            .default_width(350.0)
            .show(ctx, |ui| {
                ui.heading("File Information");
                ui.separator();

                if let Some(data) = &self.step_data {
                    egui::Grid::new("metadata_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            ui.label("File Name:");
                            ui.label(&data.metadata.file_name);
                            ui.end_row();

                            ui.label("Schema:");
                            ui.label(&data.metadata.schema_name);
                            ui.end_row();

                            ui.label("Entity Count:");
                            ui.label(format!("{}", data.metadata.entity_count));
                            ui.end_row();
                        });

                    ui.separator();
                    ui.label("Description:");
                    ui.label(&data.metadata.file_description);

                    ui.separator();
                    ui.heading("View Settings");
                    ui.color_edit_button_rgb(&mut self.ui_state.background_color);
                } else {
                    ui.label("No file loaded");
                }
            });
    }

    /// Show the 3D viewport
    fn show_viewport(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, response) = ui.allocate_exact_size(
                ui.available_size(),
                egui::Sense::click_and_drag(),
            );

            // Handle camera controls
            if response.dragged() {
                let delta = response.drag_delta();
                self.camera_rotation.0 += delta.x * 0.5;
                self.camera_rotation.1 += delta.y * 0.5;
                self.camera_rotation.1 = self.camera_rotation.1.clamp(-89.0, 89.0);
            }

            if let Some(scroll) = ui.input(|i| {
                if i.smooth_scroll_delta.y.abs() > 0.01 {
                    Some(i.smooth_scroll_delta.y)
                } else {
                    None
                }
            }) {
                self.camera_distance *= (1.0 - scroll * 0.001).max(0.1).min(10.0);
            }

            // Draw the background
            let painter = ui.painter();
            painter.rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(
                    (self.ui_state.background_color[0] * 255.0) as u8,
                    (self.ui_state.background_color[1] * 255.0) as u8,
                    (self.ui_state.background_color[2] * 255.0) as u8,
                ),
            );

            if self.step_data.is_none() {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "No STEP file loaded\n\nUse File → Open STEP file... to begin",
                    egui::FontId::proportional(20.0),
                    egui::Color32::GRAY,
                );
            } else if self.renderer.is_some() {
                // Show 3D rendering info
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!(
                        "3D Model Loaded\n\nCamera Distance: {:.1}\nRotation: ({:.0}°, {:.0}°)\n\nDrag to rotate, scroll to zoom",
                        self.camera_distance,
                        self.camera_rotation.0,
                        self.camera_rotation.1
                    ),
                    egui::FontId::proportional(16.0),
                    egui::Color32::LIGHT_GRAY,
                );

                // Draw a simple wireframe cube to show it's working
                let center = rect.center();
                let size = 100.0;
                let color = egui::Color32::from_rgb(100, 150, 200);

                // Simple 2D projection of 3D cube
                let angle_x = self.camera_rotation.0.to_radians();
                let angle_y = self.camera_rotation.1.to_radians();

                let cos_x = angle_x.cos();
                let sin_x = angle_x.sin();
                let cos_y = angle_y.cos();
                let sin_y = angle_y.sin();

                let project = |x: f32, y: f32, z: f32| -> egui::Pos2 {
                    // Rotate around Y axis
                    let x1 = x * cos_x - z * sin_x;
                    let z1 = x * sin_x + z * cos_x;

                    // Rotate around X axis
                    let y1 = y * cos_y - z1 * sin_y;
                    let z2 = y * sin_y + z1 * cos_y;

                    // Perspective projection
                    let scale = size / (4.0 + z2 * 0.5);
                    egui::pos2(
                        center.x + x1 * scale,
                        center.y + y1 * scale,
                    )
                };

                // Draw cube edges
                let corners = [
                    [-1.0, -1.0, -1.0],
                    [ 1.0, -1.0, -1.0],
                    [ 1.0,  1.0, -1.0],
                    [-1.0,  1.0, -1.0],
                    [-1.0, -1.0,  1.0],
                    [ 1.0, -1.0,  1.0],
                    [ 1.0,  1.0,  1.0],
                    [-1.0,  1.0,  1.0],
                ];

                let projected: Vec<egui::Pos2> = corners
                    .iter()
                    .map(|c| project(c[0], c[1], c[2]))
                    .collect();

                // Back face
                for i in 0..4 {
                    painter.line_segment(
                        [projected[i], projected[(i + 1) % 4]],
                        egui::Stroke::new(1.0, color),
                    );
                }

                // Front face
                for i in 4..8 {
                    painter.line_segment(
                        [projected[i], projected[4 + (i - 4 + 1) % 4]],
                        egui::Stroke::new(2.0, color),
                    );
                }

                // Connecting edges
                for i in 0..4 {
                    painter.line_segment(
                        [projected[i], projected[i + 4]],
                        egui::Stroke::new(1.0, color),
                    );
                }
            } else {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "3D renderer not initialized",
                    egui::FontId::proportional(16.0),
                    egui::Color32::RED,
                );
            }
        });
    }

    /// Show error message if present
    fn show_error(&mut self, ctx: &egui::Context) {
        if let Some(error) = &self.error_message.clone() {
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(error);
                    if ui.button("OK").clicked() {
                        self.error_message = None;
                    }
                });
        }
    }
}

impl eframe::App for StepViewerApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_menu(ctx);
        self.show_entity_list(ctx);
        self.show_metadata(ctx);
        self.show_viewport(ctx);
        self.show_error(ctx);
    }
}
