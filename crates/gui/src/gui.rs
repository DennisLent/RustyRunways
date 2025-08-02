use eframe::egui;
use rusty_runways_core::Game;

enum Screen {
    MainMenu,
    InGame,
}

pub struct RustyRunwaysGui {
    // global
    screen: Screen,

    // Main menu
    seed_str: String,
    airports_str: String,
    cash_str: String,
    save_name: String,
    recent_saves: Vec<String>,
    error: Option<String>,

    // In Game
    game: Option<Game>,
    game_error: Option<String>,

    // In Game selection
    hovered_airport: Option<usize>,
    selected_airport: Option<usize>,
    hovered_airplane: Option<usize>,
    selected_airplane: Option<usize>,

    // Additional windows
    airport_panel: bool,
    plane_panel: bool,
    stats_panel: bool,
}

impl Default for RustyRunwaysGui {
    
    fn default() -> Self {
        RustyRunwaysGui { 
            screen: Screen::MainMenu, 
            seed_str: "1".into(), 
            airports_str: "5".into(), 
            cash_str: "1000000".into(), 
            save_name: "None".into(), 
            recent_saves: Vec::new(), 
            error: None, 
            game: None, 
            game_error: None, 
            hovered_airport: None, 
            selected_airport: None, 
            hovered_airplane: None, 
            selected_airplane: None, 
            airport_panel: false, 
            plane_panel: false, 
            stats_panel: false }
    }
}

impl eframe::App for RustyRunwaysGui {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        match self.screen {
            Screen::MainMenu => self.ui_main_menu(ctx),
            Screen::InGame => self.ui_game(ctx),
        }
    }
}

impl RustyRunwaysGui {
    fn ui_main_menu(&mut self, ctx: &eframe::egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RustyRunways");
            ui.horizontal(|ui| {
                ui.label("Seed:");
                ui.text_edit_singleline(&mut self.seed_str);
                ui.label("# Airports:");
                ui.text_edit_singleline(&mut self.airports_str);
                ui.label("Cash:");
                ui.text_edit_singleline(&mut self.cash_str);
            });

            if let Some(err) = &self.game_error {
                ui.colored_label(egui::Color32::RED, err);
            }
        });
    }

    fn ui_game(&mut self, ctx: &eframe::egui::Context) {}
}
