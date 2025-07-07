pub fn sidebar_paths(
    ui: &mut egui::Ui, 
    ctx: &egui::Context,
    paths: &mut Vec<String>, 
    input_path_usestate: &mut String, 
    currently_active_path: &mut Option<String>, 
    current_path_filepaths: &mut Option<Vec<String>>
) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(input_path_usestate).desired_width(100.0));
        if ui.button("+").clicked() {
            if !input_path_usestate.is_empty() && !paths.contains(input_path_usestate) {
                paths.push(input_path_usestate.clone());
                input_path_usestate.clear();
            }
        }
    });

    let mut paths_to_remove = Vec::new();

    for path in paths.iter() {
        ui.vertical(|ui| {
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    if ui.label(path.as_str()).clicked() {
                        ctx.forget_all_images();

                        *currently_active_path = Some(path.clone());
                        *current_path_filepaths = None;
                    };
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("X").clicked() {
                            paths_to_remove.push(path.clone());
                        }
                    });
                });
            })
        });
    }
    paths.retain(|p| !paths_to_remove.contains(p));
}

pub fn sidebar_tag_manager(ui: &mut egui::Ui) {
    ui.label("Tag Manager");
    if ui.button("Option 1").clicked() {
        // Handle option 1
    }
    if ui.button("Option 2").clicked() {
        // Handle option 2
    }
}