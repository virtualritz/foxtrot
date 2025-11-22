#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::env;
use std::path::PathBuf;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    // Check for command-line arguments
    let file_path = env::args().nth(1).map(PathBuf::from);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "STEP Viewer",
        native_options,
        Box::new(move |cc| {
            let mut app = step_viewer::StepViewerApp::new(cc);
            // Load file if provided via command line
            if let Some(path) = file_path {
                app.load_file_on_startup(path);
            }
            Ok(Box::new(app))
        }),
    )
}
