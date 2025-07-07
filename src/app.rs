#[path = "utils/static_page.rs"] mod static_page;
#[path = "utils/sidebar_modules.rs"] mod sidebar_modules;
#[path = "utils/centralpanel_modules.rs"] mod centralpanel_modules;
#[path = "utils/modal.rs"] mod modal;
#[path = "utils/settings_loader.rs"] mod settings_loader;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]

pub struct TaggerrsTemplate {
    paths: Vec<String>,
    currently_active_menu: String,
    currently_active_path: Option<String>,
    gallery_media_box_size: f32,
    gallery_media_boxes_per_row: u32,

    #[serde(skip)]
    input_path_usestate: String,
    #[serde(skip)]
    current_path_filepaths: Option<Vec<String>>,
    #[serde(skip)]
    settings_modal_open: bool,
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
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
                );
            } else if self.currently_active_menu == "Tag Manager" {
                sidebar_modules::sidebar_tag_manager(ui);
            }

        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if !self.currently_active_path.is_none() {
                centralpanel_modules::file_gallery(
                    ui,
                    &self.currently_active_path,
                    &mut self.current_path_filepaths,
                    &self.gallery_media_box_size,
                    &self.gallery_media_boxes_per_row,
                );
            } else {
                static_page::default_window(ui);
            }

            static_page::footer(ui);
        });
    }
}