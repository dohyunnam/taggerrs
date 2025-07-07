pub fn file_gallery(
    ui: &mut egui::Ui, 
    currently_active_path: &Option<String>, 
    current_path_filepaths: &mut Option<Vec<String>>,
    gallery_media_box_size: &f32
) {
    if let Some(active_path) = currently_active_path {
        ui.label(format!("{}", active_path));
    }

    if let Some(path) = currently_active_path {
        let entries = std::fs::read_dir(path);
        
        if let Ok(entries) = entries {
            let files: Vec<String> = entries.filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                path.is_file() && path.extension().map_or(false, |ext| {
                ext == "png" || ext == "jpg" || ext == "jpeg"
                })
            })
            .map(|entry| entry.path().display().to_string())
            .collect();

            *current_path_filepaths = Some(files);

            if let Some(files) = current_path_filepaths.as_ref() {
                let images_per_row = 3;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for chunk in files.chunks(images_per_row) {
                        ui.horizontal(|ui| {
                            for image_path in chunk {
                                if let Ok(bytes) = std::fs::read(image_path) {
                                    ui.group(|ui| {
                                        ui.vertical(|ui| {
                                            ui.set_width(*gallery_media_box_size);
                                            ui.set_height(*gallery_media_box_size);
                                            ui.add(
                                                egui::Image::from_bytes("image", bytes)
                                            );
                                            ui.label(image_path);
                                        });
                                    });
                                }
                            }
                        });
                    }
                });
            }
        } else {
            *current_path_filepaths = None;
        }
    }
}