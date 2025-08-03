use eframe::NativeOptions;
use rusty_runways_gui::gui::RustyRunwaysGui;

fn main() {
    let options = NativeOptions::default();

    eframe::run_native(
        "RustyRunways",
        options,
        Box::new(|_cc| Ok(Box::new(RustyRunwaysGui::default()))),
    )
    .expect("failed to start eframe");
}
