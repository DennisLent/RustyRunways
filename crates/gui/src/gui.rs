use eframe::egui::{self, CornerRadius, Id, Pos2, Rect, ScrollArea, Sense, Vec2, Window};
use rand::Rng;
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
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
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
            Window::new("Save Game").open(&mut open).show(ctx, |ui| {
                ui.label("Save name:");
                ui.text_edit_singleline(&mut self.save_input);
                if ui.button("Confirm").clicked() {
                    if let Some(game) = &self.game {
                        match game.save_game(&self.save_input) {
                            Ok(_) => self.log.push(format!("Saved game '{}'.", self.save_input)),
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
            Window::new("Load Game").open(&mut open).show(ctx, |ui| {
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
                        painter.rect_filled(
                            rect,
                            CornerRadius::same(0),
                            ui.visuals().extreme_bg_color,
                        );

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
                            let resp =
                                ui.interact(hit_rect, Id::new(("airport", idx)), Sense::hover());
                            let hovered = resp.hovered();
                            resp.on_hover_text(format!(
                                "{}\nFuel ${:.2}/L",
                                airport.name, airport.fuel_price
                            ));

                            if hovered {
                                self.hovered_airport = Some(idx);
                                painter.circle_stroke(
                                    screen_pos,
                                    6.0,
                                    (2.0, egui::Color32::LIGHT_BLUE),
                                );
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

                        // Fleet overview
                        ui.heading("Fleet Overview");
                        ScrollArea::vertical()
                            .id_salt("Fleet Overview")
                            .max_height(total_height - 400.0)
                            .show(ui, |ui| {
                                for plane in self.game.as_ref().unwrap().planes() {
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

                                    if ui
                                        .button(format!(
                                            "{} | {:?} | {}",
                                            plane.id, plane.model, status
                                        ))
                                        .clicked()
                                    {
                                        self.selected_airplane = Some(plane.id);
                                        self.plane_panel = true;
                                    }
                                }
                            });
                        ui.separator();

                        // Airport overview
                        ui.heading("Airports");
                        ScrollArea::vertical()
                            .id_salt("Airport Overview")
                            .max_height(total_height - 400.0)
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
                        .min_width(left_w)
                        .max_width(total_width - 100.0)
                        .min_height(left_h)
                        .max_height(total_height - 100.0)
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

                            ui.separator();
                            ui.heading("Load Order");
                            if airport_clone.orders.is_empty() {
                                ui.label("No orders to load");
                            } else if planes_here.is_empty() {
                                ui.label("No planes at airport");
                            } else {
                                egui::ComboBox::from_label("Order")
                                    .selected_text(
                                        self.airport_order_selection
                                            .map(|o| o.to_string())
                                            .unwrap_or_else(|| "Select".into()),
                                    )
                                    .show_ui(ui, |ui| {
                                        for order in &airport_clone.orders {
                                            ui.selectable_value(
                                                &mut self.airport_order_selection,
                                                Some(order.id),
                                                order.id.to_string(),
                                            );
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
                                if ui.button("Load").clicked() {
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
                        let orders_at_airport: Vec<usize> = {
                            let g = self.game.as_ref().unwrap();
                            g.map
                                .airports
                                .iter()
                                .find(|(_, c)| *c == plane_clone.location)
                                .map(|(a, _)| a.orders.iter().map(|o| o.id).collect())
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
                                        for (airport, coord) in
                                            self.game.as_ref().unwrap().airports()
                                        {
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
                                            Ok(_) => {
                                                self.log.push(format!("Plane {} refueling", pid))
                                            }
                                            Err(e) => {
                                                self.log.push(format!("Refuel failed: {}", e))
                                            }
                                        }
                                        self.scroll_log = true;
                                    }
                                    if ui.button("Unload All").clicked() {
                                        match self.game.as_mut().unwrap().unload_all(pid) {
                                            Ok(_) => {
                                                self.log.push(format!("Plane {} unloading", pid))
                                            }
                                            Err(e) => {
                                                self.log.push(format!("Unload failed: {}", e))
                                            }
                                        }
                                        self.scroll_log = true;
                                    }
                                    if ui.button("Maintenance").clicked() {
                                        match self
                                            .game
                                            .as_mut()
                                            .unwrap()
                                            .maintenance_on_airplane(pid)
                                        {
                                            Ok(_) => self.log.push(format!(
                                                "Plane {} maintenance scheduled",
                                                pid
                                            )),
                                            Err(e) => {
                                                self.log.push(format!("Maintenance failed: {}", e))
                                            }
                                        }
                                        self.scroll_log = true;
                                    }
                                });
                                if !orders_at_airport.is_empty() {
                                    egui::ComboBox::from_label("Order")
                                        .selected_text(
                                            self.plane_order_selection
                                                .map(|o| o.to_string())
                                                .unwrap_or_else(|| "Select".into()),
                                        )
                                        .show_ui(ui, |ui| {
                                            for o in &orders_at_airport {
                                                ui.selectable_value(
                                                    &mut self.plane_order_selection,
                                                    Some(*o),
                                                    o.to_string(),
                                                );
                                            }
                                        });
                                    if ui.button("Load").clicked() {
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
                                }

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
                                            Ok(_) => self.log.push(format!(
                                                "Plane {} departing to {}",
                                                pid, dest
                                            )),
                                            Err(e) => {
                                                self.log.push(format!("Depart failed: {}", e))
                                            }
                                        }
                                        self.scroll_log = true;
                                    }
                                }
                            });
                    }
                }
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Game logs
            ui.collapsing("Game Log", |ui| {
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
        });
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
    }

    #[test]
    fn handle_click_item_plane() {
        let mut gui = RustyRunwaysGui::default();
        gui.handle_click_item(ClickItem::Plane(7));
        assert_eq!(gui.selected_airplane, Some(7));
        assert!(gui.plane_panel);
    }

    #[test]
    fn default_starts_on_main_menu() {
        let gui = RustyRunwaysGui::default();
        assert!(matches!(gui.screen, Screen::MainMenu));
        assert!(gui.game.is_none());
    }
}
