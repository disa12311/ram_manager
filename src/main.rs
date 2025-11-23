mod ram_manager;
mod gui;

use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([900.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Advanced RAM Manager",
        options,
        Box::new(|_cc| Ok(Box::new(gui::RamManagerApp::default()))),
    )
}
