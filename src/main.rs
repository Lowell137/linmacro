pub mod app;
pub mod engine;
pub mod model;
pub mod virtual_device;

use app::LinMacroApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 420.0])
            .with_min_inner_size([420.0, 380.0])
            .with_resizable(true)
            .with_title("LinMacro - Auto Clicker"),
        ..Default::default()
    };

    eframe::run_native(
        "LinMacro",
        options,
        Box::new(|_cc| Ok(Box::new(LinMacroApp::new()))),
    )
}
