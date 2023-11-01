use std::f64::consts::TAU;
use egui::plot::{Legend, Plot, PlotPoint, PlotPoints, Polygon, Text};
use egui::{Align2, RichText};
use eframe::egui;
use native_dialog::FileDialog;
use eframe::NativeOptions;
use egui_extras::{Size, StripBuilder};
use walkdir::WalkDir;
use walkdir::DirEntry;
use std::fs::metadata;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
const FULL_CIRCLE_VERTICES: f64 = 360.0;
const RADIUS: f64 = 0.9;

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}
fn calculate_directory_size(directory_path: &str) -> Result<f64, std::io::Error> {
    let path = Path::new(directory_path);
    if path.is_dir() {
        let mut total_size = 0.0;

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path: PathBuf = entry.path();
            let entry_path_str = entry_path.to_str().ok_or(std::io::Error::from(std::io::ErrorKind::InvalidData))?;

            if entry_path.is_file() {
                total_size += fs::metadata(&entry_path)?.len() as f64;
            } else if entry_path.is_dir() {
                total_size += calculate_directory_size(entry_path_str)?;
            }
        }

        Ok(total_size)
    }else if path.is_file(){
    //let entry = entry?;
            //let entry_path: PathBuf = path;
    let mut total_size = 0.0;
      total_size += fs::metadata(&path)?.len() as f64;
       Ok(total_size)
    } else {
        Ok(0.0) // Not a directory, return 0.0 size
    }
}

pub struct PieChart {
    name: String,
    sectors: Vec<Sector>,
}

impl PieChart {
    pub fn new<S: AsRef<str>, L: AsRef<str>>(name: S, data: &[(f64, L)]) -> Self {
        let sum: f64 = data.iter().map(|(f, _)| f).sum();

        let slices: Vec<_> = data.iter().map(|(f, n)| (f / sum, n)).collect();

        let step = TAU / FULL_CIRCLE_VERTICES;

        let mut offset = 0.0_f64;

        let sectors = slices
            .iter()
            .map(|(p, n)| {
                let vertices = (FULL_CIRCLE_VERTICES * p).round() as usize;

                let start = TAU * offset;
                let end = TAU * (offset + p);

                let sector = Sector::new(n, start, end, vertices, step);

                offset += p;

                sector
            })
            .collect();

        Self {
            name: name.as_ref().to_string(),
            sectors,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let sectors = self.sectors.clone();

        //copy current context for click checking
        let ctx = ui.ctx().clone();

        Plot::new(&self.name)
            .label_formatter(|_: &str, _: &PlotPoint| String::default())
            .show_background(false)
            .legend(Legend::default())
            .show_axes([false; 2])
            .clamp_grid(true)
            .allow_boxed_zoom(false)
            .allow_drag(false)
            .allow_zoom(false)
            .allow_scroll(false)
            .data_aspect(1.0)
            // .set_margin_fraction([0.7; 2].into()) // this won't prevent the plot from moving
            // `include_*` will lock it into place
            .include_x(-2.0)
            .include_x(2.0)
            .include_y(-2.0)
            .include_y(2.0)
            .show(ui, |plot_ui| {
                for sector in sectors.into_iter() {
                    let highlight = plot_ui.pointer_coordinate().map(|p| sector.contains(&p)).unwrap_or_default();

                    let Sector { name, points, .. } = sector;

                    plot_ui.polygon(Polygon::new(PlotPoints::new(points)).name(&name).highlight(highlight));

                    //check for click, uses closure (aka fxn) to check if mouse was released
                    if highlight && ctx.input(|input| input.pointer.any_released()) {
                        let temp_str = format!("{}{}", String::from("/"), String::from(name.clone()));
                        println!("Sector {} was clicked", temp_str);
                        //self.path, self.scanning_path, change paths
                        //self.path or self.path = "/".to_string();
                        //Call scanning fxn here: self.update_pie_chart_data(ui);
                        //self.update_pie_chart_data(ui);
                    }

                    if highlight {
                        let p = plot_ui.pointer_coordinate().unwrap();
                        // TODO proper zoom
                        let text = RichText::new(&name).size(15.0).heading();
                        plot_ui.text(Text::new(p, text).name(&name).anchor(Align2::LEFT_BOTTOM));
                    }
                }
            });
    }
}

#[derive(Clone)]
struct Sector {
    name: String,
    start: f64,
    end: f64,
    points: Vec<[f64; 2]>,
}

impl Sector {
    pub fn new<S: AsRef<str>>(name: S, start: f64, end: f64, vertices: usize, step: f64) -> Self {
        let mut points = vec![];

        if end - TAU != start {
            points.push([0.0, 0.0]);
        }

        points.push([RADIUS * start.sin(), RADIUS * start.cos()]);

        for v in 1..vertices {
            let t = start + step * v as f64;
            points.push([RADIUS * t.sin(), RADIUS * t.cos()]);
        }

        points.push([RADIUS * end.sin(), RADIUS * end.cos()]);

        Self {
            name: name.as_ref().to_string(),
            start,
            end,
            points,
        }
    }

    pub fn contains(&self, &PlotPoint { x, y }: &PlotPoint) -> bool {
        let r = y.hypot(x);
        let mut theta = x.atan2(y);

        if theta < 0.0 {
            theta += TAU;
        }

        r < RADIUS && theta > self.start && theta < self.end
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(10000.0, 10000.0)),
        ..Default::default()
    };
    eframe::run_native("Pie charts", options, Box::new(|ctx| Box::<MyApp>::default()))?;
    Ok(())
}

struct MyApp {
    path: String,
    scan_clicked: bool, 
    scanning_path: String,
    pie_chart: PieChart, //added
    show_pie_chart: bool,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            path: "/".to_owned(),
            scan_clicked: true,
            scanning_path: "/".to_owned(),
             pie_chart: PieChart::new("My Pie Chart", &[(0.3, "Slice A"), (0.2, "Slice B"), (0.5, "Slice C")]),
              show_pie_chart: false,
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
                    self.update_pie_chart_data(ui);
                    self.show_pie_chart = true;
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
            if ui.button("Up").clicked() {
                let index = self.path.rfind('/'); 
                self.path = self.path.clone().chars().take(index.unwrap_or(self.path.clone().len())).collect(); 
                if self.path.is_empty() {
                    self.path = "/".to_string();
                }
                self.scanning_path = self.path.clone();
            }
            if self.scan_clicked {
                //self.scan_directory(ui); 
               
            }
             if self.scan_clicked && self.show_pie_chart {
             self.pie_chart.show(ui);
              }

        });
    }
}

impl MyApp {
     /*fn scan_directory(&self, ui: &mut egui::Ui) {
        for entry in WalkDir::new(&self.scanning_path).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                ui.label(format!("File: {}", entry.path().display()));
            } else if entry.file_type().is_dir() {
                ui.label(format!("Directory: {}", entry.path().display()));
            }
        }
    }*/
     fn update_pie_chart_data(&mut self,ui: &mut egui::Ui) {
        // Replace this with your own data
        let new_data = &[(0.3, "Slice A"), (0.2, "Slice B"), (0.5, "Slice C")];
        let data: Vec<_> = (0..8).map(|i| (0.125, format!("{}: 12.5%", i + 1))).collect();

    let directory_path = "/home/mohammad/"; // Replace with your directory path
    let mut file_data: Vec<(f64, String)> = Vec::new(); // Vector to store file name and size pairs

    for entry_result in WalkDir::new(&self.scanning_path).max_depth(1).into_iter().filter_entry(|e| !is_hidden(e)) {
        match entry_result {
            Ok(entry) => {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let metadata = fs::metadata(entry.path());
                let size = calculate_directory_size(entry.path().to_str().unwrap());
                match size {
                                Ok(f) => {
                                    // The f64 value is in the Ok variant
                                   file_data.push((f, file_name));
                                }
                                Err(e) => {
                                    // Handle the error
                                    eprintln!("Error: {:?}", e);
                                }
                            }
                
                /*match metadata {
                    Ok(metadata) => {
                        //let size = metadata.len(); // Get the size of the file in bytes
                        //file_data.push((file_name, size)); // Add the pair to the vector
                        //let size = metadata.len() as f64; // Convert size to f64
                         let size =walk_dir(Path::new(directory_path));
                        file_data.push((size, file_name));
                    }
                    Err(err) => {
                        eprintln!("Error reading metadata: {}", err);
                    }
                }*/
            }
            Err(err) => {
                eprintln!("Error reading entry: {}", err);
            }
        }
    }
        self.pie_chart = PieChart::new("My Pie Chart", &file_data);
    }
}
