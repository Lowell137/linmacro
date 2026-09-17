pub mod app;
pub mod engine;
pub mod model;
pub mod virtual_device;

use app::LinMacroApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([720.0, 480.0])
            .with_title("LinMacro - Linux Makro Yöneticisi"),
        ..Default::default()
    };

    eframe::run_native(
        "LinMacro",
        options,
        Box::new(|_cc| Ok(Box::new(LinMacroApp::new()))),
    )
}
