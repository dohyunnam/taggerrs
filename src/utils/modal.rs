pub fn settings_modal (
    ui: &mut egui::Ui,
    gallery_media_box_size: &mut f32,
    gallery_media_boxes_per_row: &mut u32,
) {
    ui.label("Settings");
    ui.add(egui::Slider::new(gallery_media_box_size, 100.0..=500.0).text("Gallery Media Box Size"));
    ui.add(egui::Slider::new(gallery_media_boxes_per_row, 1..=6).text("Boxes per gallery row"));
}