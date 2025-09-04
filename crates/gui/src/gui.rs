use eframe::egui::{
    self, CornerRadius, Id, Pos2, Rect, ScrollArea, Sense, SidePanel, TopBottomPanel, Vec2, Window,
};
use rand::Rng;
use rusty_runways_core::config::WorldConfig;
use rusty_runways_core::utils::airplanes::models::AirplaneModel;
use rusty_runways_core::{Game, utils::airplanes::models::AirplaneStatus};

use crate::transforms::{map_transforms, world_to_screen};

enum Screen {
    MainMenu,
    InGame,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ClickItem {
    Airport(usize),
    Plane(usize),
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
    // config loader
    config_path: String,
    preview_cfg: Option<WorldConfig>,
    preview_open: bool,

    // In Game
    game: Option<Game>,

    // game log
    log: Vec<String>,
    scroll_log: bool,

    // save/load dialogs
    save_dialog: bool,
    load_dialog: bool,
    save_input: String,
    load_input: String,

    // In Game selection
    hovered_airport: Option<usize>,
    selected_airport: Option<usize>,
    hovered_airplane: Option<usize>,
    selected_airplane: Option<usize>,

    overlap_menu_open: bool,
    overlap_menu_items: Vec<ClickItem>,
    overlap_menu_pos: egui::Pos2,

    // selections for controls
    airport_order_selection: Option<usize>,
    airport_plane_selection: Option<usize>,
    plane_order_selection: Option<usize>,
    plane_destination: Option<usize>,
    // multi-select for orders
    airport_order_multi: std::collections::BTreeSet<usize>,
    plane_order_multi: std::collections::BTreeSet<usize>,
    // order filters (plane window)
    plane_filter_dest: Option<usize>,
    plane_filter_min_w: f32,
    plane_filter_max_w: f32,
    // buy plane dialog
    buy_dialog: bool,
    buy_model: Option<AirplaneModel>,
    buy_airport: Option<usize>,

    // Additional windows
    airport_panel: bool,
    plane_panel: bool,
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
            config_path: String::new(),
            preview_cfg: None,
            preview_open: false,
            log: Vec::new(),
            scroll_log: false,
            save_dialog: false,
            load_dialog: false,
            save_input: String::new(),
            load_input: String::new(),
            hovered_airport: None,
            selected_airport: None,
            hovered_airplane: None,
            selected_airplane: None,
            overlap_menu_open: false,
            overlap_menu_items: Vec::new(),
            overlap_menu_pos: Pos2::ZERO,
            airport_order_selection: None,
            airport_plane_selection: None,
            plane_order_selection: None,
            plane_destination: None,
            airport_order_multi: Default::default(),
            plane_order_multi: Default::default(),
            plane_filter_dest: None,
            plane_filter_min_w: 0.0,
            plane_filter_max_w: 1_000_000.0,
            buy_dialog: false,
            buy_model: None,
            buy_airport: None,
            airport_panel: false,
            plane_panel: false,
        }
    }
}

impl eframe::App for RustyRunwaysGui {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        match self.screen {
            Screen::MainMenu => self.ui_main_menu(ctx),
            Screen::InGame => self.ui_game(ctx),
        }
    }
}

impl RustyRunwaysGui {
    // main menu
    fn ui_main_menu(&mut self, ctx: &eframe::egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Title & subtitle
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.heading("RustyRunways");
                ui.small("Build your aviation empire · Manage logistics · Conquer the skies");
                ui.add_space(24.0);
            });

            ui.columns(2, |cols| {
                // Left column for new game
                cols[0].group(|ui| {
                    ui.heading("Start New Game");
                    ui.add_space(12.0);

                    ui.label("Game Seed");
                    ui.text_edit_singleline(&mut self.seed_str);
                    ui.add_space(6.0);

                    ui.label("Number of Airports");
                    ui.text_edit_singleline(&mut self.airports_str);
                    ui.add_space(6.0);

                    ui.label("Starting Cash ($)");
                    ui.text_edit_singleline(&mut self.cash_str);

                    ui.add_space(12.0);

                    // Game from inputs
                    if ui
                        .add_sized([150.0, 30.0], egui::Button::new("Launch Game"))
                        .clicked()
                    {
                        self.error = None;

                        // try to parse each field
                        // bail on failure
                        let seed = match self.seed_str.parse::<u64>() {
                            Ok(s) => s,
                            Err(e) => {
                                self.error = Some(format!("Invalid seed: {}", e));
                                return;
                            }
                        };
                        let airports = match self.airports_str.parse::<usize>() {
                            Ok(n) => n,
                            Err(e) => {
                                self.error = Some(format!("Invalid # of airports: {}", e));
                                return;
                            }
                        };
                        let cash = match self.cash_str.parse::<f32>() {
                            Ok(c) => c,
                            Err(e) => {
                                self.error = Some(format!("Invalid starting cash: {}", e));
                                return;
                            }
                        };

                        // everything parsed
                        let new_game = Game::new(seed, Some(airports), cash);
                        self.game = Some(new_game);
                        self.screen = Screen::InGame;
                    }
                });

                // middle column for loading game
                cols[1].group(|ui| {
                    ui.heading("Load Saved Game");
                    ui.add_space(12.0);

                    ui.label("Save Game Name");
                    ui.text_edit_singleline(&mut self.save_name);
                    ui.add_space(6.0);

                    ui.label("Recent Saves");
                    for name in &self.recent_saves {
                        ui.label(format!("• {}", name));
                    }

                    ui.add_space(12.0);
                    if ui
                        .add_sized([150.0, 30.0], egui::Button::new("Load Game"))
                        .clicked()
                    {
                        self.game = match Game::load_game(&self.save_name) {
                            Ok(game_instance) => {
                                self.screen = Screen::InGame;
                                Some(game_instance)
                            }
                            Err(e) => {
                                self.error = Some(format!("{}", e));
                                None
                            }
                        }
                    }
                });
            });

            ui.add_space(12.0);
            ui.group(|ui| {
                ui.heading("Start From Config");
                ui.add_space(12.0);
                ui.label("Config path (.yaml)");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.config_path);
                    if ui.button("Browse").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("YAML", &["yaml", "yml"])
                            .pick_file()
                        {
                            if let Some(p) = path.to_str() {
                                self.config_path = p.to_string();
                            }
                        }
                    }
                });
                ui.add_space(6.0);
                if ui.button("Preview").clicked() {
                    match std::fs::read_to_string(&self.config_path) {
                        Ok(text) => match serde_yaml::from_str::<WorldConfig>(&text) {
                            Ok(cfg) => {
                                self.preview_cfg = Some(cfg);
                                self.preview_open = true;
                                self.error = None;
                            }
                            Err(e) => self.error = Some(format!("YAML error: {}", e)),
                        },
                        Err(e) => self.error = Some(format!("Read error: {}", e)),
                    }
                }
                if ui.button("Start").clicked() {
                    match std::fs::read_to_string(&self.config_path) {
                        Ok(text) => match serde_yaml::from_str::<WorldConfig>(&text) {
                            Ok(cfg) => match Game::from_config(cfg) {
                                Ok(g) => {
                                    self.game = Some(g);
                                    self.screen = Screen::InGame;
                                    self.error = None;
                                }
                                Err(e) => self.error = Some(e.to_string()),
                            },
                            Err(e) => self.error = Some(format!("YAML error: {}", e)),
                        },
                        Err(e) => self.error = Some(format!("Read error: {}", e)),
                    }
                }
            });

            ui.vertical_centered(|ui| {
                ui.add_space(12.0);

                // Random game
                if ui
                    .add_sized([150.0, 30.0], egui::Button::new("Random Game"))
                    .clicked()
                {
                    let seed: u64 = rand::thread_rng().r#gen();

                    println!("[DEBUG]: started random game with: seed={}", seed);

                    self.game = Some(Game::new(seed, None, 1_000_000.0));
                    self.screen = Screen::InGame;
                }

                ui.add_space(24.0);

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::RED, err);
                }
            });
        });

        // Config preview window
        if self.preview_open {
            let mut open = true;
            Window::new("Config Preview")
                .open(&mut open)
                .resizable(true)
                .default_size(Vec2::new(640.0, 420.0))
                .show(ctx, |ui| {
                    if let Some(cfg) = &self.preview_cfg {
                        ui.label(format!(
                            "Seed: {:?} | Starting Cash: ${:.0} | Generate Orders: {}",
                            cfg.seed, cfg.starting_cash, cfg.generate_orders
                        ));
                        ui.separator();
                        ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                            for a in &cfg.airports {
                                ui.group(|ui| {
                                    ui.label(format!(
                                        "[{}] {} @ ({:.1}, {:.1})",
                                        a.id, a.name, a.location.x, a.location.y
                                    ));
                                    ui.label(format!(
                                        "Runway: {:.0}m | Fuel: ${:.2}/L | Landing: ${:.2}/t | Parking: ${:.2}/h",
                                        a.runway_length_m,
                                        a.fuel_price_per_l,
                                        a.landing_fee_per_ton,
                                        a.parking_fee_per_hour
                                    ));
                                });
                                ui.add_space(6.0);
                            }
                        });
                        ui.separator();
                        if ui.button("Start Game").clicked() {
                            if let Some(cfg2) = self.preview_cfg.clone() {
                                match Game::from_config(cfg2) {
                                    Ok(g) => {
                                        self.game = Some(g);
                                        self.screen = Screen::InGame;
                                        self.error = None;
                                        self.preview_open = false;
                                    }
                                    Err(e) => self.error = Some(e.to_string()),
                                }
                            }
                        }
                    }
                });
            self.preview_open = open;
        }
    }

    // in-game screen
    fn ui_game(&mut self, ctx: &eframe::egui::Context) {
        // keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                if let Some(g) = self.game.as_mut() {
                    g.advance(1);
                    self.log.push("Advanced 1h".to_string());
                    self.scroll_log = true;
                }
            }
            if i.key_pressed(egui::Key::Escape) {
                if self.plane_panel {
                    self.plane_panel = false;
                } else if self.airport_panel {
                    self.airport_panel = false;
                }
            }
        });

        // header
        TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("RustyRunways");
                ui.separator();
                ui.label(format!("${:.0}", self.game.as_ref().unwrap().get_cash()));
                ui.separator();
                ui.label(self.game.as_ref().unwrap().get_time().to_string());
                ui.separator();
                ui.label(format!(
                    "{} planes",
                    self.game.as_ref().unwrap().player.fleet_size
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save").clicked() {
                        self.save_dialog = true;
                        self.save_input.clear();
                    }
                    if ui.button("Load").clicked() {
                        self.load_dialog = true;
                        self.load_input.clear();
                    }
                    if ui.button("Menu").clicked() {
                        self.screen = Screen::MainMenu;
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        // Save dialog
        if self.save_dialog {
            let mut open = true;
            let mut close = false;
            Window::new("Save Game")
                .collapsible(false)
                .resizable(false)
                .default_size(Vec2::new(320.0, 140.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Save name:");
                    ui.text_edit_singleline(&mut self.save_input);
                    if ui.button("Confirm").clicked() {
                        if let Some(game) = &self.game {
                            match game.save_game(&self.save_input) {
                                Ok(_) => {
                                    self.log.push(format!("Saved game '{}'.", self.save_input))
                                }
                                Err(e) => self.log.push(format!("Save failed: {}", e)),
                            }
                            self.scroll_log = true;
                        }
                        close = true;
                    }
                });
            self.save_dialog = open && !close;
        }

        if self.load_dialog {
            let mut open = true;
            let mut close = false;
            Window::new("Load Game")
                .collapsible(false)
                .resizable(false)
                .default_size(Vec2::new(320.0, 140.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Load name:");
                    ui.text_edit_singleline(&mut self.load_input);
                    if ui.button("Confirm").clicked() {
                        match Game::load_game(&self.load_input) {
                            Ok(game_instance) => {
                                self.log.push(format!("Loaded game '{}'.", self.load_input));
                                self.game = Some(game_instance);
                            }
                            Err(e) => self.log.push(format!("Load failed: {}", e)),
                        }
                        self.scroll_log = true;
                        close = true;
                    }
                });
            self.load_dialog = open && !close;
        }

        // Right sidebar for stats/overviews
        SidePanel::right("sidebar")
            .resizable(true)
            .default_width(320.0)
            .show(ctx, |ui| {
                // stats, quick actions & planes
                ui.group(|ui| {
                    // STATS
                    ui.heading("Game Stats");
                    ui.label(format!(
                        "Income: ${:.2}\nExpenses: ${:.2}\nDeliveries: {}",
                        self.game.as_ref().unwrap().daily_income,
                        self.game.as_ref().unwrap().daily_expenses,
                        self.game.as_ref().unwrap().player.orders_delivered
                    ));
                    ui.separator();

                    // Fleet overview
                    ui.horizontal(|ui| {
                        ui.heading("Fleet Overview");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Buy new plane").clicked() {
                                self.buy_dialog = true;
                            }
                        });
                    });
                    let h = ui.available_height().min(300.0);
                    ScrollArea::vertical()
                        .id_salt("Fleet Overview")
                        .max_height(h)
                        .show(ui, |ui| {
                            let g = self.game.as_ref().unwrap();
                            let airports = g.airports().to_vec();
                            for plane in g.planes() {
                                let status = match &plane.status {
                                    AirplaneStatus::Parked => "Parked".into(),
                                    AirplaneStatus::Refueling => "Refueling".into(),
                                    AirplaneStatus::Loading => "Loading".into(),
                                    AirplaneStatus::Unloading => "Unloading".into(),
                                    AirplaneStatus::Maintenance => "Maintenance".into(),
                                    AirplaneStatus::InTransit {
                                        hours_remaining, ..
                                    } => {
                                        format!("En-route ({}h left)", hours_remaining)
                                    }
                                    AirplaneStatus::Broken => "Broken".into(),
                                };

                                let at_airport =
                                    !matches!(plane.status, AirplaneStatus::InTransit { .. });
                                let loc_text = if at_airport {
                                    airports
                                        .iter()
                                        .find(|(_, c)| *c == plane.location)
                                        .map(|(a, _)| a.name.clone())
                                        .unwrap_or_else(|| "Unknown".into())
                                } else {
                                    String::new()
                                };

                                let label = if at_airport {
                                    format!(
                                        "{} | {:?} | {} | at {}",
                                        plane.id, plane.model, status, loc_text
                                    )
                                } else {
                                    format!("{} | {:?} | {}", plane.id, plane.model, status)
                                };

                                if ui.button(label).clicked() {
                                    self.selected_airplane = Some(plane.id);
                                    self.plane_panel = true;
                                }
                            }
                        });
                    ui.separator();

                    // Airport overview
                    ui.heading("Airports");
                    let h2 = ui.available_height().min(300.0);
                    ScrollArea::vertical()
                        .id_salt("Airport Overview")
                        .max_height(h2)
                        .show(ui, |ui| {
                            for (idx, (airport, _)) in
                                self.game.as_ref().unwrap().airports().iter().enumerate()
                            {
                                if ui
                                    .button(format!("{} | {}", airport.id, airport.name))
                                    .clicked()
                                {
                                    self.selected_airport = Some(idx);
                                    self.airport_panel = true;
                                }
                            }
                        });
                    ui.separator();

                    // QUICK ACTIONS
                    ui.heading("Quick Actions");
                    if ui.button("Advance 1h").clicked() {
                        self.game.as_mut().unwrap().advance(1);
                        self.log.push("Advanced 1h".to_string());
                        self.scroll_log = true;
                    }
                });
            });

        // Buy plane dialog
        if self.buy_dialog {
            let mut open = true;
            let mut close = false;
            let airports_list = {
                let g = self.game.as_ref().unwrap();
                g.airports()
                    .iter()
                    .map(|(a, _)| (a.id, a.name.clone(), a.runway_length))
                    .collect::<Vec<_>>()
            };
            Window::new("Buy New Plane")
                .collapsible(false)
                .resizable(true)
                .default_size(Vec2::new(720.0, 520.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Select model:");
                    ui.separator();
                    let avail_w = ui.available_width();
                    ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                        let models = [
                            AirplaneModel::SparrowLight,
                            AirplaneModel::FalconJet,
                            AirplaneModel::CometRegional,
                            AirplaneModel::Atlas,
                            AirplaneModel::TitanHeavy,
                            AirplaneModel::Goliath,
                            AirplaneModel::Zephyr,
                            AirplaneModel::Lightning,
                        ];
                        for model in models.iter() {
                            let specs = model.specs();
                            let selected = self.buy_model == Some(model.clone());
                            ui.group(|ui| {
                                ui.set_width(avail_w);
                                ui.horizontal(|ui| {
                                    if ui.radio(selected, format!("{:?}", model)).clicked() {
                                        self.buy_model = Some(model.clone());
                                    }
                                    ui.separator();
                                    ui.label(format!("Price: ${:.0}", specs.purchase_price));
                                    ui.separator();
                                    ui.label(format!("Payload: {:.0}kg", specs.payload_capacity));
                                    ui.separator();
                                    ui.label(format!("Cruise: {:.0}km/h", specs.cruise_speed));
                                });
                                ui.label(format!(
                                    "Fuel: {:.0}L | Burn: {:.0} L/h | Oper cost: ${:.0}/h | Runway ≥ {:.0}m",
                                    specs.fuel_capacity, specs.fuel_consumption, specs.operating_cost, specs.min_runway_length
                                ));
                            });
                            ui.add_space(6.0);
                        }
                    });

                    ui.add_space(8.0);
                    egui::CollapsingHeader::new("Model specs table").default_open(false).show(ui, |ui| {
                        egui::Grid::new("models_table").striped(true).show(ui, |ui| {
                            ui.strong("Model");
                            ui.strong("Cruise");
                            ui.strong("Fuel");
                            ui.strong("Burn");
                            ui.strong("Oper/h");
                            ui.strong("Payload");
                            ui.strong("Price");
                            ui.strong("Runway");
                            ui.end_row();

                            let models = [
                                AirplaneModel::SparrowLight,
                                AirplaneModel::FalconJet,
                                AirplaneModel::CometRegional,
                                AirplaneModel::Atlas,
                                AirplaneModel::TitanHeavy,
                                AirplaneModel::Goliath,
                                AirplaneModel::Zephyr,
                                AirplaneModel::Lightning,
                            ];
                            for m in models {
                                let s = m.specs();
                                ui.label(format!("{:?}", m));
                                ui.label(format!("{:.0}", s.cruise_speed));
                                ui.label(format!("{:.0}", s.fuel_capacity));
                                ui.label(format!("{:.0}", s.fuel_consumption));
                                ui.label(format!("${:.0}", s.operating_cost));
                                ui.label(format!("{:.0}", s.payload_capacity));
                                ui.label(format!("${:.0}", s.purchase_price));
                                ui.label(format!("{:.0}m", s.min_runway_length));
                                ui.end_row();
                            }
                        });
                    });

                    // Balance indicator
                    if let Some(model) = &self.buy_model {
                        let price = model.specs().purchase_price;
                        let cash = self.game.as_ref().unwrap().get_cash();
                        let remaining = cash - price;
                        if cash >= price {
                            ui.colored_label(egui::Color32::LIGHT_GREEN, format!(
                                "Price ${:.0} | Cash ${:.0} | After purchase ${:.0}",
                                price, cash, remaining
                            ));
                        } else {
                            ui.colored_label(egui::Color32::RED, format!(
                                "Price ${:.0} | Cash ${:.0} | Short by ${:.0}",
                                price, cash, -remaining
                            ));
                        }
                    }

                    ui.separator();
                    ui.label("Starting airport:");
                    let selected_airport_text = self
                        .buy_airport
                        .and_then(|id| airports_list.iter().find(|(i, _, _)| *i == id).map(|(_, n, _)| n.clone()))
                        .unwrap_or_else(|| "Select".into());
                    egui::ComboBox::from_label("Airport")
                        .selected_text(selected_airport_text)
                        .show_ui(ui, |ui| {
                            for (id, name, runway) in &airports_list {
                                let label = if let Some(model) = &self.buy_model {
                                    let min_runway = model.specs().min_runway_length;
                                    let ok = *runway >= min_runway;
                                    if ok {
                                        format!("{} (runway {:.0}m · OK)", name, runway)
                                    } else {
                                        format!("{} (runway {:.0}m · too short)", name, runway)
                                    }
                                } else {
                                    format!("{} (runway {:.0}m)", name, runway)
                                };
                                ui.selectable_value(&mut self.buy_airport, Some(*id), label);
                            }
                        });

                    // Runway hint for selection
                    if let (Some(model), Some(ap_id)) = (&self.buy_model, self.buy_airport) {
                        if let Some((_, _, runway)) = airports_list.iter().find(|(i, _, _)| *i == ap_id) {
                            let need = model.specs().min_runway_length;
                            if *runway >= need {
                                ui.colored_label(egui::Color32::LIGHT_GREEN, format!(
                                    "Runway OK: need ≥ {:.0}m, airport has {:.0}m",
                                    need, runway
                                ));
                            } else {
                                ui.colored_label(egui::Color32::RED, format!(
                                    "Runway too short: need ≥ {:.0}m, airport has {:.0}m",
                                    need, runway
                                ));
                            }
                        }
                    }

                    ui.add_space(8.0);
                    let mut can_buy = false;
                    if let (Some(model), Some(ap_id)) = (&self.buy_model, self.buy_airport) {
                        let price_ok = self.game.as_ref().unwrap().get_cash() >= model.specs().purchase_price;
                        let runway_ok = airports_list
                            .iter()
                            .find(|(i, _, _)| *i == ap_id)
                            .map(|(_, _, r)| *r >= model.specs().min_runway_length)
                            .unwrap_or(false);
                        can_buy = price_ok && runway_ok;
                    }
                    if ui.add_enabled(can_buy, egui::Button::new("Confirm Purchase")).clicked() {
                        if let (Some(model), Some(ap_id)) = (self.buy_model.clone(), self.buy_airport) {
                            let model_name = format!("{:?}", model);
                            match self.game.as_mut().unwrap().buy_plane(&model_name, ap_id) {
                                Ok(_) => self.log.push(format!("Purchased {:?} at airport {}", model, ap_id)),
                                Err(e) => self.log.push(format!("Purchase failed: {}", e)),
                            }
                            self.scroll_log = true;
                            close = true;
                        }
                    }
                });
            self.buy_dialog = open && !close;
        }

        // Bottom log panel spanning full width
        TopBottomPanel::bottom("log_panel")
            .resizable(true)
            .default_height(160.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                ui.heading("Game Log");
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for entry in &self.log {
                        ui.label(entry);
                    }
                    if self.scroll_log {
                        ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        self.scroll_log = false;
                    }
                });
            });

        // Main content: world map fills remaining space
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.heading("World Map");

                let rect_size = ui.available_size();
                let (rect, _response) = ui.allocate_exact_size(rect_size, Sense::hover());
                let painter = ui.painter().with_clip_rect(rect);

                // get structs
                let airports = {
                    let g = self.game.as_ref().unwrap();
                    g.airports().to_vec()
                };
                let airplanes = {
                    let g = self.game.as_ref().unwrap();
                    g.planes().clone()
                };

                // calculate transforms
                let transform = map_transforms(&airports, rect, 8.0);

                // background
                painter.rect_filled(rect, CornerRadius::same(0), ui.visuals().extreme_bg_color);

                if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                    let mut hits = Vec::new();
                    for (idx, (_ap, coord)) in airports.iter().enumerate() {
                        let screen = world_to_screen(coord, transform);
                        if screen.distance(pos) < 6.0 {
                            hits.push(ClickItem::Airport(idx));
                        }
                    }
                    for plane in airplanes.iter() {
                        let screen = world_to_screen(&plane.location, transform);
                        if screen.distance(pos) < 6.0 {
                            hits.push(ClickItem::Plane(plane.id));
                        }
                    }
                    let primary_clicked = ui.input(|i| i.pointer.primary_clicked());
                    if primary_clicked && !hits.is_empty() {
                        if hits.len() == 1 {
                            self.handle_click_item(hits[0]);
                        } else {
                            self.overlap_menu_open = true;
                            self.overlap_menu_items = hits;
                            self.overlap_menu_pos = pos;
                        }
                    }
                }

                // airports
                for (idx, (airport, coord)) in airports.iter().enumerate() {
                    let screen_pos = world_to_screen(coord, transform);

                    let hit_rect = Rect::from_center_size(screen_pos, Vec2::splat(12.0));
                    let resp = ui.interact(hit_rect, Id::new(("airport", idx)), Sense::hover());
                    let hovered = resp.hovered();
                    resp.on_hover_text(format!(
                        "{}\nFuel ${:.2}/L",
                        airport.name, airport.fuel_price
                    ));

                    if hovered {
                        self.hovered_airport = Some(idx);
                        painter.circle_stroke(screen_pos, 6.0, (2.0, egui::Color32::LIGHT_BLUE));
                    }
                    painter.circle_filled(screen_pos, 4.0, egui::Color32::BLUE);
                }

                // planes
                for plane in &airplanes {
                    if let AirplaneStatus::InTransit {
                        hours_remaining: _,
                        destination,
                        origin,
                        total_hours: _,
                    } = plane.status
                    {
                        let pos0 = world_to_screen(&origin, transform);
                        let pos1 = world_to_screen(&airports[destination].1, transform);
                        painter.line_segment([pos0, pos1], (1.0, egui::Color32::YELLOW));
                    }
                }
                for (idx, plane) in airplanes.iter().enumerate() {
                    let p = world_to_screen(&plane.location, transform);
                    let rect = Rect::from_center_size(p, Vec2::splat(12.0));
                    let resp = ui.interact(rect, Id::new(("plane", idx)), Sense::hover());
                    let hovered = resp.hovered();
                    resp.on_hover_text(format!(
                        "Plane {}\nFuel {:.0}/{:.0}L\nPayload {:.0}/{:.0}kg",
                        plane.id,
                        plane.current_fuel,
                        plane.specs.fuel_capacity,
                        plane.current_payload,
                        plane.specs.payload_capacity
                    ));
                    if hovered {
                        self.hovered_airplane = Some(plane.id);
                        painter.circle_stroke(p, 6.0, (2.0, egui::Color32::LIGHT_GREEN));
                    }
                    painter.circle_filled(p, 5.0, egui::Color32::WHITE);
                }
            });

            if self.overlap_menu_open {
                let popup_id = egui::Id::new("overlap_menu");
                if let Some(resp) = egui::Popup::new(
                    popup_id,
                    ui.ctx().clone(),
                    self.overlap_menu_pos,
                    ui.layer_id(),
                )
                .show(|ui| {
                    ui.vertical(|ui| {
                        for item in self.overlap_menu_items.clone() {
                            let label = match item {
                                ClickItem::Airport(i) => format!("Airport {}", i),
                                ClickItem::Plane(p) => format!("Plane {}", p),
                            };
                            if ui.button(label).clicked() {
                                self.handle_click_item(item);
                                self.overlap_menu_open = false;
                            }
                        }
                    });
                }) {
                    if ui.input(|i| i.pointer.any_click()) {
                        if let Some(pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                            if !resp.response.rect.contains(pos) {
                                self.overlap_menu_open = false;
                            }
                        } else {
                            self.overlap_menu_open = false;
                        }
                    }
                }
            }
        });

        // Airport window with information

        if let Some(idx) = self.selected_airport {
            if self.airport_panel {
                let (airport_clone, coord) = {
                    let g = self.game.as_ref().unwrap();
                    let (a, c) = &g.map.airports[idx];
                    (a.clone(), *c)
                };
                let planes_here: Vec<usize> = {
                    let g = self.game.as_ref().unwrap();
                    g.planes()
                        .iter()
                        .filter(|p| p.location == coord)
                        .map(|p| p.id)
                        .collect()
                };
                Window::new(format!("Airport: {}", airport_clone.name))
                    .open(&mut self.airport_panel)
                    .collapsible(false)
                    .min_width(600.0)
                    .min_height(360.0)
                    .default_size(Vec2::new(720.0, 520.0))
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.label(format!("ID: {}", airport_clone.id));
                        ui.label(format!("Location: ({:.1}, {:.1})", coord.x, coord.y));
                        ui.label(format!("Runway: {:.0}m", airport_clone.runway_length));
                        ui.label(format!("Fuel price: ${:.2}/L", airport_clone.fuel_price));
                        ui.label(format!("Parking fee: ${:.2}/hr", airport_clone.parking_fee));
                        ui.label(format!(
                            "Landing fee: ${:.2}/ton",
                            airport_clone.landing_fee
                        ));
                        ui.separator();
                        ui.heading("Outstanding Orders");
                        ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                            if airport_clone.orders.is_empty() {
                                ui.label("No pending orders.");
                            } else {
                                let full_width = ui.available_width();
                                for order in &airport_clone.orders {
                                    ui.group(|group_ui| {
                                        group_ui.set_width(full_width);
                                        let dest_name = &self.game.as_ref().unwrap().map.airports
                                            [order.destination_id]
                                            .0
                                            .name;
                                        group_ui.horizontal(|ui| {
                                            ui.strong(format!("[{}] {:?}", order.id, order.name));
                                            ui.separator();
                                            ui.label("Dest:");
                                            ui.label(dest_name);
                                        });
                                        group_ui.add_space(4.0);
                                        group_ui.label(format!("Weight:   {:.1} kg", order.weight));
                                        group_ui.label(format!("Value:    ${:.2}", order.value));
                                        group_ui.label(format!("Deadline: {}", order.deadline));
                                        group_ui.add_space(4.0);
                                    });
                                    ui.add_space(4.0);
                                }
                            }
                        });

                        ui.separator();
                        ui.heading("Load Order(s)");
                        if airport_clone.orders.is_empty() {
                            ui.label("No orders to load");
                        } else if planes_here.is_empty() {
                            ui.label("No planes at airport");
                        } else {
                            // Single-select with detailed labels
                            let selected_text = if let Some(sel) = self.airport_order_selection {
                                if let Some(o) = airport_clone.orders.iter().find(|o| o.id == sel) {
                                    let dest_name = &self.game.as_ref().unwrap().map.airports
                                        [o.destination_id]
                                        .0
                                        .name;
                                    format!(
                                        "[{}] {:?} | wt {:.1}kg | dest {} | dl {} | ${:.2}",
                                        o.id, o.name, o.weight, dest_name, o.deadline, o.value
                                    )
                                } else {
                                    "Select".into()
                                }
                            } else {
                                "Select".into()
                            };
                            egui::ComboBox::from_label("Order (single)")
                                .selected_text(selected_text)
                                .show_ui(ui, |ui| {
                                    for o in &airport_clone.orders {
                                        let dest_name = &self.game.as_ref().unwrap().map.airports
                                            [o.destination_id]
                                            .0
                                            .name;
                                        let label = format!(
                                            "[{}] {:?} | wt {:.1}kg | dest {} | dl {} | ${:.2}",
                                            o.id, o.name, o.weight, dest_name, o.deadline, o.value
                                        );
                                        ui.selectable_value(
                                            &mut self.airport_order_selection,
                                            Some(o.id),
                                            label,
                                        );
                                    }
                                });
                            // Multi-select list
                            ui.separator();
                            ui.label("Select multiple orders:");
                            let full_w = ui.available_width();
                            ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                                for order in &airport_clone.orders {
                                    let mut checked = self.airport_order_multi.contains(&order.id);
                                    ui.horizontal(|ui| {
                                        ui.set_width(full_w);
                                        if ui
                                            .checkbox(
                                                &mut checked,
                                                format!(
                                                    "[{}] {:?} ({:.1}kg)",
                                                    order.id, order.name, order.weight
                                                ),
                                            )
                                            .changed()
                                        {
                                            if checked {
                                                self.airport_order_multi.insert(order.id);
                                            } else {
                                                self.airport_order_multi.remove(&order.id);
                                            }
                                        }
                                    });
                                }
                            });
                            egui::ComboBox::from_label("Plane")
                                .selected_text(
                                    self.airport_plane_selection
                                        .map(|p| p.to_string())
                                        .unwrap_or_else(|| "Select".into()),
                                )
                                .show_ui(ui, |ui| {
                                    for plane_id in &planes_here {
                                        ui.selectable_value(
                                            &mut self.airport_plane_selection,
                                            Some(*plane_id),
                                            plane_id.to_string(),
                                        );
                                    }
                                });
                            ui.horizontal(|ui| {
                                if ui.button("Load (single)").clicked() {
                                    if let (Some(o), Some(p)) =
                                        (self.airport_order_selection, self.airport_plane_selection)
                                    {
                                        match self.game.as_mut().unwrap().load_order(o, p) {
                                            Ok(_) => self
                                                .log
                                                .push(format!("Loaded order {} on plane {}", o, p)),
                                            Err(e) => self.log.push(format!("Load failed: {}", e)),
                                        }
                                        self.scroll_log = true;
                                    }
                                }
                                if ui.button("Load Selected").clicked() {
                                    if let Some(p) = self.airport_plane_selection {
                                        let selected: Vec<usize> =
                                            self.airport_order_multi.iter().cloned().collect();
                                        for o in selected {
                                            match self.game.as_mut().unwrap().load_order(o, p) {
                                                Ok(_) => self.log.push(format!(
                                                    "Loaded order {} on plane {}",
                                                    o, p
                                                )),
                                                Err(e) => {
                                                    self.log.push(format!("Load failed: {}", e))
                                                }
                                            }
                                        }
                                        self.scroll_log = true;
                                        self.airport_order_multi.clear();
                                    }
                                }
                            });
                        }
                    });
            }
        }

        if let Some(pid) = self.selected_airplane {
            if self.plane_panel {
                if let Some(plane_clone) = self
                    .game
                    .as_ref()
                    .unwrap()
                    .planes()
                    .iter()
                    .find(|p| p.id == pid)
                    .cloned()
                {
                    let orders_at_airport = {
                        let g = self.game.as_ref().unwrap();
                        g.map
                            .airports
                            .iter()
                            .find(|(_, c)| *c == plane_clone.location)
                            .map(|(a, _)| a.orders.clone())
                            .unwrap_or_default()
                    };
                    let airports_list = {
                        let g = self.game.as_ref().unwrap();
                        g.airports()
                            .iter()
                            .map(|(a, _)| (a.id, a.name.clone()))
                            .collect::<Vec<_>>()
                    };

                    Window::new(format!("Plane {}", pid))
                        .open(&mut self.plane_panel)
                        .collapsible(false)
                        .default_size(Vec2::new(440.0, 520.0))
                        .resizable(true)
                        .show(ctx, |ui| {
                            ui.label(format!("Model: {:?}", plane_clone.model));
                            ui.label(format!(
                                "Fuel: {:.0}/{:.0}L",
                                plane_clone.current_fuel, plane_clone.specs.fuel_capacity
                            ));
                            ui.label(format!(
                                "Payload: {:.0}/{:.0}kg",
                                plane_clone.current_payload, plane_clone.specs.payload_capacity
                            ));
                            ui.separator();
                            ui.heading("Manifest");
                            ScrollArea::vertical()
                                .max_height(200.0)
                                .id_salt("manifest")
                                .show(ui, |ui| {
                                    if plane_clone.manifest.is_empty() {
                                        ui.label("No cargo");
                                    } else {
                                        for order in &plane_clone.manifest {
                                            ui.label(format!(
                                                "[{}] {:?} wt {:.1} val ${:.2} dl {}",
                                                order.id,
                                                order.name,
                                                order.weight,
                                                order.value,
                                                order.deadline
                                            ));
                                        }
                                    }
                                });

                            ui.separator();
                            ui.heading("Reachable Airports");
                            ScrollArea::vertical()
                                .max_height(200.0)
                                .id_salt("airports")
                                .show(ui, |ui| {
                                    for (airport, coord) in self.game.as_ref().unwrap().airports() {
                                        let can_fly: bool =
                                            plane_clone.can_fly_to(airport, coord).is_ok();

                                        ui.label(format!(
                                            "[{} | {}]: {}",
                                            airport.id, airport.name, can_fly
                                        ));
                                    }
                                });

                            ui.separator();
                            ui.horizontal(|ui| {
                                if ui.button("Refuel").clicked() {
                                    match self.game.as_mut().unwrap().refuel_plane(pid) {
                                        Ok(_) => self.log.push(format!("Plane {} refueling", pid)),
                                        Err(e) => self.log.push(format!("Refuel failed: {}", e)),
                                    }
                                    self.scroll_log = true;
                                }
                                if ui.button("Unload All").clicked() {
                                    match self.game.as_mut().unwrap().unload_all(pid) {
                                        Ok(_) => self.log.push(format!("Plane {} unloading", pid)),
                                        Err(e) => self.log.push(format!("Unload failed: {}", e)),
                                    }
                                    self.scroll_log = true;
                                }
                                if ui.button("Maintenance").clicked() {
                                    match self.game.as_mut().unwrap().maintenance_on_airplane(pid) {
                                        Ok(_) => self
                                            .log
                                            .push(format!("Plane {} maintenance scheduled", pid)),
                                        Err(e) => {
                                            self.log.push(format!("Maintenance failed: {}", e))
                                        }
                                    }
                                    self.scroll_log = true;
                                }
                            });
                            if !orders_at_airport.is_empty() {
                                // Filters
                                ui.separator();
                                ui.heading("Filter Orders");
                                ui.horizontal(|ui| {
                                    // Destination filter
                                    let selected_dest = self
                                        .plane_filter_dest
                                        .and_then(|id| {
                                            airports_list
                                                .iter()
                                                .find(|(i, _)| *i == id)
                                                .map(|(_, n)| n.clone())
                                        })
                                        .unwrap_or_else(|| "All".into());
                                    egui::ComboBox::from_label("Destination")
                                        .selected_text(selected_dest)
                                        .show_ui(ui, |ui| {
                                            if ui
                                                .selectable_label(
                                                    self.plane_filter_dest.is_none(),
                                                    "All",
                                                )
                                                .clicked()
                                            {
                                                self.plane_filter_dest = None;
                                            }
                                            for (id, name) in &airports_list {
                                                ui.selectable_value(
                                                    &mut self.plane_filter_dest,
                                                    Some(*id),
                                                    name.clone(),
                                                );
                                            }
                                        });
                                    // Weight filter
                                    ui.label("Min wt");
                                    ui.add(
                                        egui::DragValue::new(&mut self.plane_filter_min_w)
                                            .speed(10.0),
                                    );
                                    ui.label("Max wt");
                                    ui.add(
                                        egui::DragValue::new(&mut self.plane_filter_max_w)
                                            .speed(10.0),
                                    );
                                    if ui.button("Reset").clicked() {
                                        self.plane_filter_dest = None;
                                        self.plane_filter_min_w = 0.0;
                                        self.plane_filter_max_w = 1_000_000.0;
                                    }
                                });

                                let filtered_orders: Vec<
                                    &rusty_runways_core::utils::orders::order::Order,
                                > = orders_at_airport
                                    .iter()
                                    .filter(|o| {
                                        let dest_ok = match self.plane_filter_dest {
                                            Some(d) => o.destination_id == d,
                                            None => true,
                                        };
                                        let w = o.weight;
                                        let w_ok = w >= self.plane_filter_min_w
                                            && w <= self.plane_filter_max_w;
                                        dest_ok && w_ok
                                    })
                                    .collect();

                                // single-select with detailed labels
                                let selected_text = if let Some(sel) = self.plane_order_selection {
                                    if let Some(o) = filtered_orders.iter().find(|o| o.id == sel) {
                                        let dest_name = &self.game.as_ref().unwrap().map.airports
                                            [o.destination_id]
                                            .0
                                            .name;
                                        format!(
                                            "[{}] {:?} | wt {:.1}kg | dest {} | dl {} | ${:.2}",
                                            o.id, o.name, o.weight, dest_name, o.deadline, o.value
                                        )
                                    } else {
                                        "Select".into()
                                    }
                                } else {
                                    "Select".into()
                                };
                                egui::ComboBox::from_label("Order (single)")
                                    .selected_text(selected_text)
                                    .show_ui(ui, |ui| {
                                        for o in &filtered_orders {
                                            let dest_name =
                                                &self.game.as_ref().unwrap().map.airports
                                                    [o.destination_id]
                                                    .0
                                                    .name;
                                            let label = format!(
                                                "[{}] {:?} | wt {:.1}kg | dest {} | dl {} | ${:.2}",
                                                o.id,
                                                o.name,
                                                o.weight,
                                                dest_name,
                                                o.deadline,
                                                o.value
                                            );
                                            ui.selectable_value(
                                                &mut self.plane_order_selection,
                                                Some(o.id),
                                                label,
                                            );
                                        }
                                    });
                                // multi-select with detailed labels
                                ui.separator();
                                ui.label("Select multiple orders:");
                                ScrollArea::vertical()
                                    .max_height(140.0)
                                    .id_salt("plane_orders_multi")
                                    .show(ui, |ui| {
                                        for o in &filtered_orders {
                                            let mut checked =
                                                self.plane_order_multi.contains(&o.id);
                                            let dest_name =
                                                &self.game.as_ref().unwrap().map.airports
                                                    [o.destination_id]
                                                    .0
                                                    .name;
                                            let label = format!(
                                                "[{}] {:?} | wt {:.1}kg | dest {} | dl {} | ${:.2}",
                                                o.id,
                                                o.name,
                                                o.weight,
                                                dest_name,
                                                o.deadline,
                                                o.value
                                            );
                                            if ui.checkbox(&mut checked, label).changed() {
                                                if checked {
                                                    self.plane_order_multi.insert(o.id);
                                                } else {
                                                    self.plane_order_multi.remove(&o.id);
                                                }
                                            }
                                        }
                                    });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    if ui.button("Load (single)").clicked() {
                                        if let Some(o) = self.plane_order_selection {
                                            match self.game.as_mut().unwrap().load_order(o, pid) {
                                                Ok(_) => self.log.push(format!(
                                                    "Loaded order {} on plane {}",
                                                    o, pid
                                                )),
                                                Err(e) => {
                                                    self.log.push(format!("Load failed: {}", e))
                                                }
                                            }
                                            self.scroll_log = true;
                                        }
                                    }
                                    if ui.button("Load Selected").clicked() {
                                        let selected: Vec<usize> =
                                            self.plane_order_multi.iter().cloned().collect();
                                        for o in selected {
                                            match self.game.as_mut().unwrap().load_order(o, pid) {
                                                Ok(_) => self.log.push(format!(
                                                    "Loaded order {} on plane {}",
                                                    o, pid
                                                )),
                                                Err(e) => {
                                                    self.log.push(format!("Load failed: {}", e))
                                                }
                                            }
                                        }
                                        self.scroll_log = true;
                                        self.plane_order_multi.clear();
                                    }
                                });
                            }

                            ui.add_space(8.0);
                            egui::ComboBox::from_label("Destination")
                                .selected_text(
                                    self.plane_destination
                                        .and_then(|id| {
                                            airports_list
                                                .iter()
                                                .find(|(i, _)| *i == id)
                                                .map(|(_, n)| n.clone())
                                        })
                                        .unwrap_or_else(|| "Select".into()),
                                )
                                .show_ui(ui, |ui| {
                                    for (id, name) in &airports_list {
                                        ui.selectable_value(
                                            &mut self.plane_destination,
                                            Some(*id),
                                            name.clone(),
                                        );
                                    }
                                });
                            if ui.button("Depart").clicked() {
                                if let Some(dest) = self.plane_destination {
                                    match self.game.as_mut().unwrap().depart_plane(pid, dest) {
                                        Ok(_) => self
                                            .log
                                            .push(format!("Plane {} departing to {}", pid, dest)),
                                        Err(e) => self.log.push(format!("Depart failed: {}", e)),
                                    }
                                    self.scroll_log = true;
                                }
                            }
                        });
                }
            }
        }
    }

    fn handle_click_item(&mut self, item: ClickItem) {
        match item {
            ClickItem::Airport(idx) => {
                self.selected_airport = Some(idx);
                self.airport_panel = true;
            }
            ClickItem::Plane(id) => {
                self.selected_airplane = Some(id);
                self.plane_panel = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ClickItem, RustyRunwaysGui, Screen};

    #[test]
    fn handle_click_item_airport() {
        let mut gui = RustyRunwaysGui::default();
        gui.handle_click_item(ClickItem::Airport(2));
        assert_eq!(gui.selected_airport, Some(2));
        assert!(gui.airport_panel);
        assert!(gui.selected_airplane.is_none());
        assert!(!gui.plane_panel);
    }

    #[test]
    fn handle_click_item_plane() {
        let mut gui = RustyRunwaysGui::default();
        gui.handle_click_item(ClickItem::Plane(7));
        assert_eq!(gui.selected_airplane, Some(7));
        assert!(gui.plane_panel);
        assert!(gui.selected_airport.is_none());
        assert!(!gui.airport_panel);
    }

    #[test]
    fn default_starts_on_main_menu() {
        let gui = RustyRunwaysGui::default();
        assert!(matches!(gui.screen, Screen::MainMenu));
        assert!(gui.game.is_none());
    }

    #[test]
    fn default_inputs_are_seeded() {
        let gui = RustyRunwaysGui::default();
        assert_eq!(gui.seed_str, "1");
        assert_eq!(gui.airports_str, "5");
        assert_eq!(gui.cash_str, "1000000");
    }
}
