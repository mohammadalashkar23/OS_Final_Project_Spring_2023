use std::time::SystemTime;
use std::time::Duration;
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
use std::cell::RefCell;
use std::rc::Rc;
use duplicates::paths::get_descendants;
use duplicates::duplicates::get_duplicates_hashed;
use std::fs::File;
use std::io::Write;
use sysinfo::{NetworkExt, NetworksExt, ProcessExt, System, SystemExt};
//use core::error::Error;
use core::fmt::Error;
use std::fs::OpenOptions;
//use eframe::Error;
//use native_dialog::Error;
use std::process::Command;
fn is_directory_empty<P: AsRef<std::path::Path>>(dir_path: P) -> bool {
    let walker = WalkDir::new(&dir_path).into_iter();

    // Check if there are no entries in the directory
    walker.filter_map(|entry| entry.ok()).next().is_none()
}
fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}
fn is_compressed_file(entry: &DirEntry) -> bool {
    let compressed_extensions = [".zip", ".gz", ".tar",".rar"];
    entry
        .path()
        .to_str()
        .map(|name| compressed_extensions.iter().any(|&ext| name.ends_with(ext)))
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
#[derive(Clone)]
struct PieChart {
    name: String,
    sectors: Vec<Sector>,
}
#[derive(Clone)]
struct MyApp {
    path: String,
    scan_clicked: bool, 
    scanning_path: String,
    pie_chart: PieChart,
    show_pie_chart: bool,
    small_directories: Vec<String>,
    small_directories2: Vec<f64>,
    radius: f64,
    hidden: bool, 
    compressed: bool, 
    sorted: bool,
}
impl PieChart {
    //creates empty pie chart, which will eventually be updated w/ proper radius
    pub fn new_empty() -> Self {
        Self {
            name: String::new(),
            sectors: Vec::new(),
        }
    }
    
    pub fn new<S: AsRef<str>, L: AsRef<str>, P: AsRef<str>>(name: S, data: &[(f64, L, P)], radius: f64) -> Self {
        let sum: f64 = data.iter().map(|(f, _, _)| f).sum();

        let slices: Vec<_> = data.iter().map(|(f, n, d)| (f / sum, f, n, d)).collect();

        let step = TAU / FULL_CIRCLE_VERTICES;

        let mut offset = 0.0_f64;

        let sectors = slices
            .iter()
            .map(|(p, f, n, d)| {
                let vertices = (FULL_CIRCLE_VERTICES * p).round() as usize;

                let start = TAU * offset;
                let end = TAU * (offset + p);

                let sector = Sector::new(n, start, end, vertices, step, d, radius, **f);

                offset += p;

                sector
            })
            .collect();

        Self {
            name: name.as_ref().to_string(),
            sectors,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> String{
        let sectors = self.sectors.clone();
         let desired_size = egui::vec2(100.0, 100.0); // Set your desired size here
    //let desired_size_usize = egui::vec2(desired_size.x as usize, desired_size.y as usize);

        //copy current context for click checking
        let ctx = ui.ctx().clone();
        let mut temp_str = String::new();
        Plot::new(&self.name)
            .width(1290.0)
            .height(530.0)
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
                        temp_str = format!("{}", String::from(sector.path.clone()));
                    }
                    if highlight {
                        let p = plot_ui.pointer_coordinate().unwrap();
                        let mut p1 = p.clone(); 
                        p1.y = p1.y-0.05;
                        // TODO proper zoom
                        let text = RichText::new(&name).size(15.0).heading();
                        let text1 = RichText::new(&(sector.size).to_string()).size(15.0).heading();
                        plot_ui.text(Text::new(p, text).name(&name).anchor(Align2::LEFT_BOTTOM));
                        plot_ui.text(Text::new(p1, text1).name(&(sector.size).to_string()).anchor(Align2::LEFT_BOTTOM));
                    }
                }
            });
            temp_str
    }
}

#[derive(Clone)]
struct Sector {
    name: String,
    start: f64,
    end: f64,
    points: Vec<[f64; 2]>,
    path: String, 
    size: f64,
}

impl Sector {
    pub fn new<S: AsRef<str>, P: AsRef<str>>(name: S, start: f64, end: f64, vertices: usize, step: f64, path: P, radius: f64, size: f64) -> Self {
        let mut points = vec![];

        if end - TAU != start {
            points.push([0.0, 0.0]);
        }

        points.push([radius * start.sin(), radius * start.cos()]);

        for v in 1..vertices {
            let t = start + step * v as f64;
            points.push([radius * t.sin(), radius * t.cos()]);
        }

        points.push([radius * end.sin(), radius * end.cos()]);

        Self {
            name: name.as_ref().to_string(),
            start,
            end,
            points,
            path: path.as_ref().to_string(),
            size, 
        }
    }

    pub fn contains(&self, &PlotPoint { x, y }: &PlotPoint) -> bool {
        let r = y.hypot(x);
        let mut theta = x.atan2(y);

        if theta < 0.0 {
            theta += TAU;
        }
        //gets last point in self.points array (which is on circumference), calcs distance from 0,0 to that point to get radius.
        let radius = (self.points.last().unwrap()[0]).hypot(self.points.last().unwrap()[1]);
        r < radius && theta > self.start && theta < self.end
    }
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            path: "/home".to_owned(),
            scan_clicked: true,
            scanning_path: "/home".to_owned(),
            pie_chart: PieChart::new_empty(),
            show_pie_chart: true,
            // Initialize small_directories as an empty vector
            small_directories: Vec::new(),
            small_directories2: Vec::new(),
            radius: 0.0,
            hidden: false, 
            compressed: false, 
            sorted: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Disk Analyzer");
            //find max size of x and y axis, radius will be less than that.
            self.radius = ui.available_size().x.min(ui.available_size().y) as f64 / 1000.0;

            ui.horizontal(|ui| {
                if ui.button("Scan").clicked() {
                    self.scanning_path = self.path.clone(); 
                    self.scan_clicked = true; 
                    //self.show_pie_chart = true;
                    self.update_pie_chart_data(ui);
                }
                let path_label = ui.label("Path: ");
                ui.text_edit_singleline(&mut self.path)
                    .labelled_by(path_label.id);
                ui.label("filters");
                ui.checkbox(&mut self.hidden, "Hidden files");
                ui.checkbox(&mut self.compressed, "Compressed files");
                ui.checkbox(&mut self.sorted, "Sorted visual");
            });
            ui.horizontal(|ui| {
            if ui.button("Browse").clicked() {
                // Open a folder selection dialog using new_picker()
                if let Ok(folder) = FileDialog::new()
                    .add_filter("All Files", &["*"])
                    .show_open_single_dir()
                {
                    self.path = folder.expect("failed").to_str().unwrap_or_default().to_owned();
                    self.scanning_path = self.path.clone();
                }
                self.update_pie_chart_data(ui);
            }
            if ui.button("Up").clicked() {
                if self.path.is_empty() || self.path == "/home" {
                    self.path = "/home".to_string();
                }else{
                    let index = self.path.rfind('/'); 
                    self.path = self.path.clone().chars().take(index.unwrap_or(self.path.clone().len())).collect();     
                }
                self.scanning_path = self.path.clone();
                self.update_pie_chart_data(ui);
            }
            if ui.button("Report").clicked() {
                if let Err(e) = self.create_file() {
                    eprintln!("Error creating file: {:?}", e);
                } else {
                    println!("File created successfully!");
                }
            }
            if ui.button("Recommendations").clicked() {
                if let Err(e) = self.recommendations() {
                    eprintln!("Error creating file: {:?}", e);
                } else {
                    println!("File created successfully!");
                }
            }
             });
            if self.scan_clicked {
                let temp_str = self.pie_chart.show(ui);
                if temp_str != "" {
                    self.path = temp_str; 
                    self.scanning_path = self.path.clone();
                    self.update_pie_chart_data(ui);
                }
            }
             let row_height = 10.0;
let total_rows = 10;
   egui::ScrollArea::vertical().max_height(20.0).max_width(200.0).auto_shrink([false;2]).show_rows(ui, row_height, total_rows, |ui, row_range| {
   if !self.small_directories.is_empty(){let mut combined_labels: Vec<String> = Vec::new();

    // Combine the labels from small_directories and converted f64 values from small_directories2
    for (directory, num) in self.small_directories.iter().zip(&self.small_directories2) {
        let label = format!("{} - {}", directory, num);
        combined_labels.push(label);
    }

    for directory in &combined_labels {
                   ui.label(directory);
               }
    }
    else
    {
    ui.label("No small directories found.");
    }
});
 
        });
    }
}

impl MyApp {

fn recommendations(&self)->Result<(), std::io::Error> {
        let trash_path= dirs::home_dir()
        .map(|home| home.join(".local/share/Trash"))
        .expect("Could not determine home directory");
        fs::remove_file("recommendations.txt")?;
        let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("recommendations.txt")?;
        writeln!(file, "Directories to consider:")?;
        // Append data to the file
        if !is_directory_empty(&trash_path) {
            writeln!(file, "- Empty the trash")?;
        }
        let base_dir = dirs::home_dir().map(|home| home.join(""))
        .expect("Could not determine home directory");
        let one_month_ago = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60);
        for entry in WalkDir::new(&base_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            // Check if it is a directory
            if path.is_dir() {
                // Get metadata
                if let Ok(metadata) = fs::metadata(path) {
                    // Check access time
                    if let Ok(access_time) = metadata.accessed() {
                        if access_time < one_month_ago {
                            writeln!(file,"- Directory accessed more than a month ago: {:?}", path);
                            // Add logic to process or store the path as needed
                        }
                    }
                }
            }
        }
        let base_path = PathBuf::from("/home/hajj-ahmed/OS_Final_Project_Spring_2023/src");
        let descendants = get_descendants(&base_path);
        let duplicates = get_duplicates_hashed(&descendants);
        for row in duplicates.iter() {
        // Iterate over elements in each row
            for element in row.iter() {
                // Do something with the element
                println!("PathBuf: {:?}", element);
            }
        }
        Ok(())
    }
fn create_file(&self) -> Result<(), std::io::Error> {
       let mut file = File::create("disk_space.txt")?;
    let mut sys = System::new_all();
    sys.refresh_all();
// Get the current disk
       let output = Command::new("df")
        .arg("-h").arg("-T") // Human-readable output
        .output()
        .expect("Failed to execute command");

    // Check if the command succeeded
    if output.status.success() {
        // Write the output to the file
        file.write_all(&output.stdout)?;
    } else {
        // If the command failed, write the error message to the file
        let error_msg = format!("Error: {}", String::from_utf8_lossy(&output.stderr));
        file.write_all(error_msg.as_bytes())?;
    }
    Ok(())
       
    }
    fn update_pie_chart_data(&mut self,ui: &mut egui::Ui) {
        let mut total_size = 0.0;
        let mut file_data: Vec<(f64, String, String)> = Vec::new(); // Vector to store file name and size pairs
       if self.scanning_path=="others"
       {
        
       }
       else
       {
        for entry_result in WalkDir::new(&self.scanning_path).max_depth(1).into_iter() {
            match entry_result {
                Ok(entry) => {
                if ((entry.file_type().is_dir() && !is_hidden(&entry)) || (self.hidden && is_hidden(&entry)) || (self.compressed && is_compressed_file(&entry))) && entry.path() != Path::new(&self.scanning_path) {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let entry_path = entry.path().to_string_lossy().to_string();
                    let size = calculate_directory_size(entry.path().to_str().unwrap());
                    match size {
                        Ok(f) => {
                            // The f64 value is in the Ok variant
                            total_size = total_size + f;
                            file_data.push((f, file_name, entry_path));
                        }
                        Err(e) => {
                            // Handle the error
                            eprintln!("Error: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                // Handle the error
                eprintln!("Error reading entry: {:?}", e);
            }
            }
        }
        if self.sorted {
            file_data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        }
        let smallest_size = total_size/360.0; //smallest size a dir can be
        let mut clean_file_data: Vec<(f64, String, String)> = Vec::new(); // Vector to store file name and size pairs, only dirs > 1/360th total size
        let mut small_file_data: Vec<(f64, String, String)> = Vec::new(); // Vector to store file name and size pairs, only smallers dirs

        for i in 0..file_data.len(){
            if file_data[i].0 > smallest_size{
                clean_file_data.push((file_data[i].0, file_data[i].1.to_string(), file_data[i].2.to_string()));
            }
            else{
                small_file_data.push((file_data[i].0, file_data[i].1.to_string(), file_data[i].2.to_string()));
            }
           
        }
        let mut total_small=0.0; 
        for i in 0..small_file_data.len()
        {
        total_small=total_small + small_file_data[i].0; 
        }
        if total_small>0.0 
        { if total_small<smallest_size 
        { total_small =smallest_size; }
        clean_file_data.push((total_small, ("others").to_string(), ("others").to_string()));
        }

        //Draw box here:

        let mut radius = ui.available_size().x.min(ui.available_size().y) / 1000.0;
        self.pie_chart = PieChart::new("Pie Chart", &clean_file_data, self.radius.into());
        self.small_directories = small_file_data.iter().map(|(_, name, _)| name.clone()).collect();
         self.small_directories2 = small_file_data.iter().map(|(size, _, _)| size.clone()).collect();
    }
    }
}

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(10000.0, 10000.0)),
        ..Default::default()
    };
    eframe::run_native("DISK ANALYZER", options, Box::new(|ctx| Box::<MyApp>::default()))?;
    Ok(())
}
