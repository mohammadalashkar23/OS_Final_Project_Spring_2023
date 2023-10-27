use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Label, ListBox, ScrolledWindow};
use walkdir::WalkDir;
use walkdir::DirEntry;
use std::fs::metadata;
use std::fs;

const APP_ID: &str = "org.gtk_rs.HelloWorld2";

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

fn main() {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let list_box = ListBox::new();
let path_label = Label::new(None);
    path_label.set_text("Current Directory:"); 
    let path_label_clone = path_label.clone(); 
    let path_entry = Entry::new();
    let path_entry_clone = path_entry.clone(); 
    let list_box_clone = list_box.clone();
    
    //path_entry.set_placeholder_text("Enter directory path");
    let button = Button::builder()
        .label("Scan")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    // Connect to "clicked" signal of `button`
     button.connect_clicked(move |_| {
    let binding = path_entry_clone.text();
    let directory_path = binding.as_str();
path_label_clone.set_text(&format!("Current Directory: {}", directory_path));
   

    for entry_result in WalkDir::new(&directory_path).into_iter().filter_entry(|e| !is_hidden(e)) {
        match entry_result {
            Ok(entry) => {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let metadata = fs::metadata(entry.path());
                match metadata {
                    Ok(metadata) => {
                        let size = metadata.len(); // Get the size of the file in bytes
                        let label_text = format!("{} - {} bytes", file_name, size);
                        let label = Label::new(Some(&label_text));
                        list_box_clone.append(&label);
                    }
                    Err(err) => {
                        eprintln!("Error reading metadata: {}", err);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error reading entry: {}", err);
            }
        }
    }
});


    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    let scrolled_window = ScrolledWindow::builder()
        //.hscrollbar_policy(PolicyType::Never) // Disable horizontal scrolling
        .min_content_width(700)
        .max_content_width(700)
        .min_content_height(250) // Adjust the minimum height if needed
    .max_content_height(250) // Adjust the maximum height if needed
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .child(&list_box)
        .build();
     gtk_box.append(&path_label);
    gtk_box.append(&path_entry);
    gtk_box.append(&button);
    gtk_box.append(&scrolled_window);

    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .default_width(400)
        .default_height(400)
        .child(&gtk_box)
        .build();

    window.present();
}

