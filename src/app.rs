#[path = "utils/static_page.rs"] mod static_page;
#[path = "utils/sidebar_modules.rs"] mod sidebar_modules;
#[path = "utils/centralpanel_modules.rs"] mod centralpanel_modules;
#[path = "utils/modal.rs"] mod modal;
#[path = "utils/settings_loader.rs"] mod settings_loader;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use egui_file_dialog::FileDialog;

#[derive(Clone)]
pub struct ImageData {
    pub bytes: Vec<u8>,
    pub loading: bool,
}

#[derive(Clone)]
pub enum DirectoryScanState {
    Scanning,
    Complete(Vec<String>),
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]

pub struct TaggerrsTemplate {
    paths: Vec<String>,
    currently_active_menu: String,
    gallery_media_box_size: f32,
    gallery_media_boxes_per_row: u32,

    #[serde(skip)]
    currently_active_path: Option<String>,
    #[serde(skip)]
    input_path_usestate: String,
    #[serde(skip)]
    current_path_filepaths: Option<Vec<String>>,
    #[serde(skip)]
    settings_modal_open: bool,
    #[serde(skip)]
    image_cache: Arc<Mutex<HashMap<String, ImageData>>>,
    #[serde(skip)]
    runtime: Arc<tokio::runtime::Runtime>,
    #[serde(skip)]
    directory_scan_state: Arc<Mutex<HashMap<String, DirectoryScanState>>>,
    #[serde(skip)]
    file_dialog: FileDialog,
}

impl Default for TaggerrsTemplate {
    fn default() -> Self {
        Self {
            paths: vec![],
            currently_active_menu: "Paths".to_string(),
            currently_active_path: None,
            input_path_usestate: "".to_string(),
            current_path_filepaths: None,
            settings_modal_open: false,
            gallery_media_box_size: 200.0,
            gallery_media_boxes_per_row: 2,
            image_cache: Arc::new(Mutex::new(HashMap::new())),
            runtime: Arc::new(tokio::runtime::Runtime::new().unwrap()),
            directory_scan_state: Arc::new(Mutex::new(HashMap::new())),
            file_dialog: FileDialog::new()
                .default_size([600.0, 400.0])
                .show_new_folder_button(true)
                .show_search(true),
        }
    }
}

impl TaggerrsTemplate {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TaggerrsTemplate {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update the file dialog first
        self.file_dialog.update(ctx);
        
        // Check if user picked a folder
        if let Some(picked_path) = self.file_dialog.take_picked() {
            let path_str = picked_path.to_string_lossy().to_string();
            if !self.paths.contains(&path_str) {
                self.paths.push(path_str.clone());
            }
            self.currently_active_path = Some(path_str.clone());
            self.current_path_filepaths = None;
            
            // Reset directory scan state for new path
            if let Ok(mut state_map) = self.directory_scan_state.try_lock() {
                state_map.remove(&path_str);
            }
        }
        
        // Only request repaint when needed to prevent excessive CPU usage
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Settings").clicked() {
                        self.settings_modal_open = true;
                    }
                });
            });
        });

        if self.settings_modal_open {
            egui::Window::new("Settings")
                .open(&mut self.settings_modal_open)
                .default_pos(egui::pos2(300.0, 200.0))
                .show(ctx, |ui| {
                    modal::settings_modal(
                        ui,
                        &mut self.gallery_media_box_size,
                        &mut self.gallery_media_boxes_per_row,
                    );
                }
            );
        }

        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.label("Paths").clicked() {
                    self.currently_active_menu = "Paths".to_string();
                } 
                ui.separator();
                if ui.label("Tag Manager").clicked() {
                    self.currently_active_menu = "Tag Manager".to_string();
                }
            });
            ui.separator();
            if self.currently_active_menu == "Paths" {
                sidebar_modules::sidebar_paths(
                    ui,
                    ctx,
                    &mut self.paths,
                    &mut self.input_path_usestate,
                    &mut self.currently_active_path,
                    &mut self.current_path_filepaths,
                    &self.directory_scan_state,
                    &mut self.file_dialog,
                );
            } else if self.currently_active_menu == "Tag Manager" {
                sidebar_modules::sidebar_tag_manager(ui);
            }

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.currently_active_path.is_none() {
                centralpanel_modules::file_gallery(
                    ui,
                    ctx,
                    &self.currently_active_path,
                    &mut self.current_path_filepaths,
                    &self.gallery_media_box_size,
                    &self.gallery_media_boxes_per_row,
                    &self.image_cache,
                    &self.runtime,
                    &self.directory_scan_state,
                );
            } else {
                static_page::default_window(ui);
            }

            static_page::footer(ui);
        });
    }
}