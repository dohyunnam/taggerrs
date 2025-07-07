pub fn footer (ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Powered by ");
            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            ui.label(" and ");
            ui.hyperlink_to(
                "eframe",
                "https://github.com/emilk/egui/tree/master/crates/eframe",
            );
            ui.label(".");
        });
        egui::warn_if_debug_build(ui);
    });
}

pub fn default_window(ui: &mut egui::Ui) {
    ui.heading("Welcome to Taggerrs!");
    ui.label("This is a static page template for your application.");
    ui.add_space(20.0);
    ui.label("You can customize this page to suit your needs.");
    ui.add_space(20.0);
    ui.label("Feel free to explore the code and make changes as you see fit.");
}