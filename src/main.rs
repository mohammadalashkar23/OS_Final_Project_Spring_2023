//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui;
use walkdir:: WalkDir;
use native_dialog::FileDialog;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(10000.0, 10000.0)),
        ..Default::default()
    };
    eframe::run_native(
        "DISK ANALYZER",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<MyApp>::default()
        }),
    )
}

struct MyApp {
    path: String,
    scan_clicked: bool, 
    scanning_path: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            path: "/".to_owned(),
            scan_clicked: true,
            scanning_path: "/".to_owned(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disk Analyzer");
            ui.horizontal(|ui| {
                if ui.button("Scan").clicked() {
                    self.scanning_path = self.path.clone(); 
                    self.scan_clicked = true; 
                }
                let path_label = ui.label("Path: ");
                ui.text_edit_singleline(&mut self.path)
                    .labelled_by(path_label.id);
            });
            if ui.button("Browse").clicked() {
                // Open a folder selection dialog using new_picker()
                if let Ok(folder) = FileDialog::new()
                    .add_filter("All Files", &["*"])
                    .show_open_single_dir()
                {
                    self.path = folder.expect("failed").to_str().unwrap_or_default().to_owned();
                    self.scanning_path = self.path.clone();
                }
            }
            if ui.button("UP").clicked() {
                let index = self.path.rfind('/'); 
                self.path = self.path.clone().chars().take(index.unwrap_or(self.path.clone().len())).collect(); 
                if self.path.is_empty() {
                    self.path = "/".to_string();
                }
                self.scanning_path = self.path.clone();
            }
            if self.scan_clicked {
                self.scan_directory(ui); 
            }
        });
    }
}

impl MyApp {
     fn scan_directory(&self, ui: &mut egui::Ui) {
        for entry in WalkDir::new(&self.scanning_path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                ui.label(format!("File: {}", entry.path().display()));
            } else if entry.file_type().is_dir() {
                ui.label(format!("Directory: {}", entry.path().display()));
            }
        }
    }
}
