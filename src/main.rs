pub mod app;
pub mod engine;
pub mod ipc;
pub mod model;
pub mod virtual_device;

use app::LinMacroApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let flag = &args[1];
        if flag == "--toggle" || flag == "toggle" {
            ipc::send_ipc_command("toggle");
            return Ok(());
        }
        if flag == "--start" || flag == "start" {
            ipc::send_ipc_command("start");
            return Ok(());
        }
        if flag == "--stop" || flag == "stop" {
            ipc::send_ipc_command("stop");
            return Ok(());
        }
    }

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
