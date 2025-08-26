mod app;
mod music;
mod player;
mod playlist;
mod settings;
mod ui;

use app::{MyApp, setup_custom_fonts};


fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([320.0, 240.0])
            .with_title("FLAC Music Player"),
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
