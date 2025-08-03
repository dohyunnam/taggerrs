use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::app::{ImageData, DirectoryScanState};

pub fn file_gallery(
    ui: &mut egui::Ui, 
    ctx: &egui::Context,
    currently_active_path: &Option<String>, 
    current_path_filepaths: &mut Option<Vec<String>>,
    gallery_media_box_size: &f32,
    gallery_media_boxes_per_row: &u32,
    image_cache: &Arc<Mutex<HashMap<String, ImageData>>>,
    runtime: &Arc<tokio::runtime::Runtime>,
    directory_scan_state: &Arc<Mutex<HashMap<String, DirectoryScanState>>>,
) {
    if let Some(active_path) = currently_active_path {
        ui.label(format!("{}", active_path));
    }

    if let Some(path) = currently_active_path {
        // Check directory scan state without blocking
        let scan_state = {
            if let Ok(mut state_map) = directory_scan_state.try_lock() {
                match state_map.get(path) {
                    Some(DirectoryScanState::Complete(files)) => {
                        *current_path_filepaths = Some(files.clone());
                        Some(DirectoryScanState::Complete(files.clone()))
                    }
                    Some(DirectoryScanState::Scanning) => {
                        Some(DirectoryScanState::Scanning)
                    }
                    _ => {
                        // Start scanning if not already started
                        state_map.insert(path.clone(), DirectoryScanState::Scanning);
                        
                        // Spawn async task without blocking
                        let path_clone = path.clone();
                        let state_clone = directory_scan_state.clone();
                        let ctx_clone = ctx.clone();
                        
                        runtime.spawn(async move {
                            let files = scan_directory_async(&path_clone).await;
                            
                            // Update state without blocking the UI
                            {
                                let mut state_map = state_clone.lock().await;
                                state_map.insert(path_clone, DirectoryScanState::Complete(files));
                            }
                            
                            ctx_clone.request_repaint();
                        });
                        
                        Some(DirectoryScanState::Scanning)
                    }
                }
            } else {
                None
            }
        };

        match scan_state {
            Some(DirectoryScanState::Complete(_)) => {
                if let Some(files) = current_path_filepaths.as_ref() {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for chunk in files.chunks((*gallery_media_boxes_per_row).try_into().unwrap()) {
                            ui.horizontal(|ui| {
                                for image_path in chunk {
                                    display_image_async(
                                        ui, 
                                        ctx,
                                        image_path, 
                                        *gallery_media_box_size, 
                                        image_cache, 
                                        runtime
                                    );
                                }
                            });
                        }
                    });
                }
            }
            Some(DirectoryScanState::Scanning) => {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.label("Scanning directory...");
                });
            }
            _ => {
                ui.label("Ready to scan...");
            }
        }
    }
}

async fn scan_directory_async(path: &str) -> Vec<String> {
    match tokio::fs::read_dir(path).await {
        Ok(mut entries) => {
            let mut files = Vec::new();
            let mut count = 0;
            const MAX_FILES: usize = 1000; // Limit to prevent memory issues
            
            while let Ok(Some(entry)) = entries.next_entry().await {
                if count >= MAX_FILES {
                    break; // Prevent loading too many files at once
                }
                
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if matches!(ext_str.as_str(), "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "mp4" | "avi" | "mov" | "mkv" | "webm" | "m4v" | "flv") {
                            files.push(path.display().to_string());
                            count += 1;
                        }
                    }
                }
            }
            files
        }
        Err(_) => Vec::new(),
    }
}

fn display_image_async(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    image_path: &str,
    box_size: f32,
    image_cache: &Arc<Mutex<HashMap<String, ImageData>>>,
    runtime: &Arc<tokio::runtime::Runtime>,
) {
    let cache_clone = image_cache.clone();
    let ctx_clone = ctx.clone();
    let path_clone = image_path.to_string();
    
    // Check if image is in cache without blocking
    let cached_image = if let Ok(cache) = cache_clone.try_lock() {
        cache.get(&path_clone).cloned()
    } else {
        None
    };
    
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.set_width(box_size);
            ui.set_height(box_size);
            
            match cached_image {
                Some(image_data) if !image_data.loading => {
                    // Image is loaded, display it
                    if is_video_file(&path_clone) {
                        // For videos, show thumbnail if available, otherwise show placeholder
                        ui.add(
                            egui::Image::from_bytes(path_clone.clone(), image_data.bytes)
                                .max_width(box_size - 10.0)
                                .max_height(box_size - 10.0)
                        );
                        ui.label("🎬 Video");
                    } else {
                        ui.add(
                            egui::Image::from_bytes(path_clone.clone(), image_data.bytes)
                                .max_width(box_size - 10.0)
                                .max_height(box_size - 10.0)
                        );
                    }
                }
                Some(_) => {
                    // Loading placeholder
                    ui.spinner();
                    ui.label("Loading...");
                }
                None => {
                    // Not loaded yet, start loading
                    ui.spinner();
                    ui.label("Loading...");
                    
                    // Start loading without blocking
                    let cache_clone2 = cache_clone.clone();
                    let path_clone2 = path_clone.clone();
                    
                    // Limit concurrent image loading to prevent overwhelming the system
                    runtime.spawn(async move {
                        // Check if another task is already loading this image
                        {
                            let mut cache = cache_clone2.lock().await;
                            if cache.contains_key(&path_clone2) {
                                return; // Already being loaded
                            }
                            
                            // Limit total cache size to prevent memory issues
                            if cache.len() > 200 {
                                // Remove some old entries
                                let keys_to_remove: Vec<String> = cache.keys().take(50).cloned().collect();
                                for key in keys_to_remove {
                                    cache.remove(&key);
                                }
                            }
                            
                            cache.insert(path_clone2.clone(), ImageData {
                                bytes: Vec::new(),
                                loading: true,
                            });
                        }
                        
                        // Load image/video thumbnail asynchronously
                        match load_image_or_thumbnail_async(&path_clone2).await {
                            Ok(bytes) => {
                                // Check if the bytes are reasonable size (under 10MB)
                                if bytes.len() < 10 * 1024 * 1024 {
                                    let mut cache = cache_clone2.lock().await;
                                    cache.insert(path_clone2, ImageData {
                                        bytes,
                                        loading: false,
                                    });
                                    ctx_clone.request_repaint();
                                } else {
                                    // File too large, remove from cache
                                    let mut cache = cache_clone2.lock().await;
                                    cache.remove(&path_clone2);
                                }
                            }
                            Err(_) => {
                                // Handle loading error by removing from cache
                                let mut cache = cache_clone2.lock().await;
                                cache.remove(&path_clone2);
                            }
                        }
                    });
                }
            }
            
            // Show filename
            if let Some(filename) = std::path::Path::new(&path_clone).file_name() {
                ui.label(filename.to_string_lossy());
            }
        });
    });
}

async fn load_image_or_thumbnail_async(path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if is_video_file(path) {
        // Generate video thumbnail
        generate_video_thumbnail_async(path).await
    } else {
        // Load image directly
        tokio::fs::read(path).await.map_err(|e| e.into())
    }
}

async fn generate_video_thumbnail_async(video_path: &str) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // For now, create a simple placeholder image for videos
    // In a real implementation, you'd use ffmpeg or similar to generate actual thumbnails
    let filename = std::path::Path::new(video_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
        
    let placeholder_svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"200\" height=\"150\" viewBox=\"0 0 200 150\">\
        <rect width=\"200\" height=\"150\" fill=\"#333333\"/>\
        <circle cx=\"100\" cy=\"75\" r=\"20\" fill=\"#ffffff\"/>\
        <polygon points=\"95,65 95,85 110,75\" fill=\"#333333\"/>\
        <text x=\"100\" y=\"120\" text-anchor=\"middle\" fill=\"#ffffff\" font-family=\"Arial\" font-size=\"12\">Video</text>\
        <text x=\"100\" y=\"135\" text-anchor=\"middle\" fill=\"#aaaaaa\" font-family=\"Arial\" font-size=\"10\">{}</text>\
        </svg>", 
        filename
    );
    
    Ok(placeholder_svg.into_bytes())
}

fn is_video_file(path: &str) -> bool {
    if let Some(ext) = std::path::Path::new(path).extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        matches!(ext_str.as_str(), "mp4" | "avi" | "mov" | "mkv" | "webm" | "m4v" | "flv")
    } else {
        false
    }
}