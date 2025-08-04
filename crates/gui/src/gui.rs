use eframe::egui::{self, CornerRadius, Grid, Id, Rect, ScrollArea, Sense, Vec2, Window};
use rand::Rng;
use rusty_runways_core::{Game, utils::airplanes::models::AirplaneStatus};

use crate::transforms::{map_transforms, world_to_screen};

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
            hovered_airport: None,
            selected_airport: None,
            hovered_airplane: None,
            selected_airplane: None,
            airport_panel: false,
            plane_panel: false,
            stats_panel: false,
        }
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

                // right column for loading game
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
            })
        });
    }

    // in-game screen
    fn ui_game(&mut self, ctx: &eframe::egui::Context) {
        // header
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("RustyRunways");
                ui.separator();
                ui.label(format!("${:.0}", self.game.as_ref().unwrap().get_cash()));
                ui.separator();
                ui.label(format!("{}", self.game.as_ref().unwrap().get_time()));
                ui.separator();
                ui.label(format!(
                    "{} planes",
                    self.game.as_ref().unwrap().player.fleet_size
                ));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Save").clicked() {
                        // TODO: Add saving functionality
                    }
                    if ui.button("Load").clicked() {
                        self.screen = Screen::MainMenu;
                    }
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            // split horizontally:
            // map (left) 70%
            // stats (right) 30%
            let total_width = ui.available_width();
            let total_height = ui.available_height();
            let left_w = total_width * 0.7;
            let left_h = total_height * 0.7;
            ui.horizontal(|ui| {
                // world map
                ui.vertical(|ui| {
                    ui.set_width(left_w);
                    ui.set_height(left_h);
                    ui.group(|ui| {
                        ui.heading("World Map");

                        let rect_size = Vec2::new(left_w, left_h);
                        let (rect, _response) = ui.allocate_exact_size(rect_size, Sense::hover());
                        let painter = ui.painter().with_clip_rect(rect);

                        // get structs
                        let airports = self.game.as_ref().unwrap().airports();
                        let airplanes = self.game.as_ref().unwrap().planes();

                        // calculate transforms
                        let transform = map_transforms(airports, rect, 8.0);

                        // background
                        painter.rect_filled(
                            rect,
                            CornerRadius::same(0),
                            ui.visuals().extreme_bg_color,
                        );

                        // airports
                        for (idx, (airport, coord)) in airports.iter().enumerate() {
                            let screen_pos = world_to_screen(coord, transform);

                            // make a tiny rectangle around the circle:
                            let hit_rect = Rect::from_center_size(screen_pos, Vec2::splat(12.0));
                            let resp =
                                ui.interact(hit_rect, Id::new(("airport", idx)), Sense::click());

                            // highlight on hover
                            if resp.hovered() {
                                painter.circle_stroke(
                                    screen_pos,
                                    6.0,
                                    (2.0, egui::Color32::LIGHT_BLUE),
                                );
                            }
                            // or normal
                            painter.circle_filled(screen_pos, 4.0, egui::Color32::BLUE);

                            // click => show panel
                            if resp.clicked() {
                                self.selected_airport = Some(idx);
                                self.airport_panel = true;
                            }
                        }

                        // planes
                        for plane in airplanes {
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

                        for plane in airplanes {
                            let p = world_to_screen(&plane.location, transform);
                            painter.circle_filled(p, 5.0, egui::Color32::WHITE);
                        }
                    });
                });

                // stats, quick actions & planes
                ui.vertical(|ui| {
                    ui.set_width(total_width - left_w);
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

                        ui.heading("Fleet Overview");
                        //Give 400 px to all other components, may need adjustment
                        ScrollArea::vertical()
                            .max_height(total_height - 400.0)
                            .show(ui, |ui| {
                                Grid::new("fleet_grid")
                                    .striped(true)
                                    .min_col_width(40.0)
                                    .show(ui, |ui| {
                                        ui.heading("ID");
                                        ui.heading("Model");
                                        ui.heading("Status");
                                        ui.end_row();

                                        for plane in self.game.as_ref().unwrap().planes() {
                                            ui.label(plane.id.to_string());
                                            ui.label(format!("{:?}", plane.model));

                                            let status = match &plane.status {
                                                AirplaneStatus::Parked => "Parked".into(),
                                                AirplaneStatus::Refueling => "Refueling".into(),
                                                AirplaneStatus::Loading => "Loading".into(),
                                                AirplaneStatus::Unloading => "Unloading".into(),
                                                AirplaneStatus::Maintenance => "Maintenance".into(),
                                                AirplaneStatus::InTransit {
                                                    hours_remaining,
                                                    ..
                                                } => {
                                                    format!("En-route ({}h left)", hours_remaining)
                                                }
                                            };

                                            ui.label(status);

                                            ui.end_row();
                                        }
                                    })
                            });
                        ui.separator();

                        // QUICK ACTIONS
                        ui.heading("Quick Actions");
                        if ui.button("Advance 1h").clicked() {
                            self.game.as_mut().unwrap().advance(1);
                        }
                        // TODO: Other quick actions
                    });
                });
            });

            // Airport window with information
            let total_width = ui.available_width();
            let total_height = ui.available_height();
            let left_w = total_width * 0.7;
            let left_h = total_height * 0.7;

            if let Some(idx) = self.selected_airport {
                if self.airport_panel {
                    let (airport, coord) = &self.game.as_ref().unwrap().map.airports[idx];
                    Window::new(format!("Airport: {}", airport.name))
                        .open(&mut self.airport_panel)
                        .min_width(left_w)
                        .max_width(total_width - 100.0)
                        .min_height(left_h) 
                        .max_height(total_height - 100.0)
                        .resizable(true)  
                        .show(ctx, |ui| {
                            ui.label(format!("ID: {}", airport.id));
                            ui.label(format!("Location: ({:.1}, {:.1})", coord.x, coord.y));
                            ui.label(format!("Runway: {:.0}m", airport.runway_length));
                            ui.label(format!("Fuel price: ${:.2}/L", airport.fuel_price));
                            ui.label(format!("Parking fee: ${:.2}/hr", airport.parking_fee));
                            ui.label(format!("Landing fee: ${:.2}/ton", airport.landing_fee));
                            ui.separator();
                            ui.heading("Outstanding Orders");
                            ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                                if airport.orders.is_empty() {
                                    ui.label("No pending orders.");
                                } else {
                                    let full_width = ui.available_width();
                                    for order in &airport.orders {
                                        ui.group(|group_ui| {
                                            
                                            group_ui.set_width(full_width);

                                            // Top line: “[ID] Name → Dest”
                                            let dest_name =
                                                &self.game.as_ref().unwrap().map.airports
                                                    [order.destination_id]
                                                    .0
                                                    .name;
                                            group_ui.horizontal(|ui| {
                                                ui.strong(format!(
                                                    "[{}] {:?}",
                                                    order.id, order.name
                                                ));
                                                ui.separator();
                                                ui.label("Dest:");
                                                ui.label(dest_name);
                                            });
                                            group_ui.add_space(4.0);

                                            // Each field on its own line
                                            group_ui
                                                .label(format!("Weight:   {:.1} kg", order.weight));
                                            group_ui
                                                .label(format!("Value:    ${:.2}", order.value));
                                            group_ui.label(format!("Deadline: {}", order.deadline));

                                            group_ui.add_space(4.0);
                                        });
                                        ui.add_space(4.0);
                                    }
                                }
                            });
                        });
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Logs
            ui.collapsing("Game Log", |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // TODO: Add logs
                });
            });
        });
    }
}
