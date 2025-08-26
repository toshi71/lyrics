mod app;
mod music;
mod player;
mod playlist;
mod settings;
mod ui;

use app::{MyApp, setup_custom_fonts};

fn load_icon() -> Option<eframe::egui::IconData> {
    let icon_bytes = include_bytes!("../icon.png");
    let image = image::load_from_memory(icon_bytes).ok()?;
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();
    
    Some(eframe::egui::IconData {
        rgba: rgba_image.into_raw(),
        width: width as u32,
        height: height as u32,
    })
}


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_title("FLAC Music Player")
            .with_icon(load_icon().unwrap_or_default()),
        ..Default::default()
    };
    
    eframe::run_native(
        "FLAC Music Player",
        options,
        Box::new(|cc| {
            let app = MyApp::new();
            setup_custom_fonts(&cc.egui_ctx, &app.settings);
            Ok(Box::new(app))
        }),
    )
}
