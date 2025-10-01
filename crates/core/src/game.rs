#![allow(unexpected_cfgs)]

use crate::config::{
    AirplaneCatalogStrategy, AirplaneModelConfig, DEFAULT_FUEL_INTERVAL_HOURS,
    DEFAULT_RESTOCK_CYCLE_HOURS, FuelGameplay, GameplayConfig, ManualOrderConfig, WorldConfig,
};
use crate::events::{Event, GameTime, ScheduledEvent};
use crate::player::Player;
use crate::statistics::DailyStats;
use crate::utils::airplanes::airplane::Airplane;
use crate::utils::airplanes::models::{AirplaneModel, AirplaneSpecs, AirplaneStatus};
use crate::utils::airport::Airport;
use crate::utils::coordinate::Coordinate;
use crate::utils::errors::GameError;
use crate::utils::map::Map;
use crate::utils::orders::{
    DemandGenerationParams, OrderGenerationParams, PassengerGenerationParams,
    order::{Order, OrderPayload},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use rusty_runways_commands::Command::*;
use rusty_runways_commands::{Command, parse_command};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::path::{Path, PathBuf};
use std::{fs, io};
use strum::IntoEnumIterator;

const REPORT_INTERVAL: u64 = 24;
const DEFAULT_RESTOCK_CYCLE: u64 = DEFAULT_RESTOCK_CYCLE_HOURS;
const DEFAULT_FUEL_INTERVAL: u64 = DEFAULT_FUEL_INTERVAL_HOURS;

fn default_rng() -> StdRng {
    StdRng::seed_from_u64(0)
}

fn default_model_catalog() -> HashMap<String, AirplaneSpecs> {
    let mut m = HashMap::new();
    for model in AirplaneModel::iter() {
        m.insert(format!("{:?}", model), model.specs());
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AirportConfig, FuelGameplay, GameplayConfig, Location, ManualOrderConfig, OrderTuning,
        OrdersGameplay, PassengerTuning, WorldConfig,
    };
    use crate::utils::orders::CargoType;
    use tempfile::tempdir;

    fn base_gameplay_config() -> GameplayConfig {
        GameplayConfig {
            restock_cycle_hours: 168,
            fuel_interval_hours: 6,
            orders: OrdersGameplay {
                regenerate: true,
                generate_initial: true,
                tuning: OrderTuning::default(),
                passengers: PassengerTuning::default(),
            },
            fuel: FuelGameplay::default(),
        }
    }

    fn manual_airport(
        id: usize,
        name: &str,
        location: Option<Location>,
        orders: Vec<ManualOrderConfig>,
    ) -> AirportConfig {
        AirportConfig {
            id,
            name: name.to_string(),
            location,
            runway_length_m: Some(3_200.0),
            fuel_price_per_l: Some(1.4),
            landing_fee_per_ton: Some(4.3),
            parking_fee_per_hour: Some(12.0),
            orders,
        }
    }

    #[test]
    fn world_event_branches_update_prices() {
        let mut game = Game::new(9, Some(3), 100_000.0);
        let base = game.map.airports[0].0.fuel_price;
        game.schedule(
            game.time,
            Event::WorldEvent {
                airport: Some(0),
                factor: 1.2,
                duration: 1,
            },
        );
        assert!(game.tick_event());
        assert!((game.map.airports[0].0.fuel_price - base * 1.2).abs() < 1e-3);

        game.events.clear();
        game.events.push(ScheduledEvent {
            time: game.time + 1,
            event: Event::WorldEventEnd {
                airport: Some(0),
                factor: 1.2,
            },
        });
        game.time += 1;
        assert!(game.tick_event());
        game.events.clear();
        assert!((game.map.airports[0].0.fuel_price - base).abs() < 1e-3);
    }

    #[cfg(feature = "ui_prints")]
    #[test]
    fn list_airports_handles_passenger_orders() {
        let mut game = Game::new(10, Some(3), 250_000.0);
        game.map.airports[0].0.orders.push(Order {
            id: 123,
            payload: OrderPayload::Passengers { count: 14 },
            value: 5_500.0,
            deadline: 18,
            origin_id: 0,
            destination_id: 1,
        });
        game.list_airports(true);
    }

    #[test]
    fn refresh_specs_restores_passenger_capacity() {
        let mut game = Game::new(11, Some(3), 300_000.0);
        game.airplanes[0].specs.passenger_capacity = 0;
        game.player.fleet[0].specs.passenger_capacity = 0;
        game.refresh_airplane_specs();
        let expected = game.airplanes[0].model.specs().passenger_capacity;
        assert_eq!(game.airplanes[0].specs.passenger_capacity, expected);
        assert_eq!(game.player.fleet[0].specs.passenger_capacity, expected);
    }

    #[test]
    fn gameplay_settings_rejects_zero_restock_cycle() {
        let mut cfg = base_gameplay_config();
        cfg.restock_cycle_hours = 0;
        let err = gameplay_settings(&cfg).unwrap_err();
        assert!(err.contains("restock_cycle_hours"));
    }

    #[test]
    fn from_config_handles_manual_passenger_and_cargo_orders() {
        let mut gameplay = base_gameplay_config();
        gameplay.orders.regenerate = false;
        gameplay.orders.generate_initial = false;

        let airports = vec![
            manual_airport(
                0,
                "HUB",
                Some(Location {
                    x: 1_200.0,
                    y: 900.0,
                }),
                vec![ManualOrderConfig::Passengers {
                    passengers: 24,
                    value: 8_400.0,
                    deadline_hours: 18,
                    destination_id: 1,
                }],
            ),
            manual_airport(
                1,
                "AAX",
                None,
                vec![ManualOrderConfig::Cargo {
                    cargo: CargoType::Food,
                    weight: 320.0,
                    value: 6_200.0,
                    deadline_hours: 26,
                    destination_id: 0,
                }],
            ),
        ];

        let cfg = WorldConfig {
            seed: Some(77),
            starting_cash: 550_000.0,
            airports,
            num_airports: None,
            gameplay,
            airplanes: None,
        };

        let game = Game::from_config(cfg).expect("config should be accepted");

        assert_eq!(game.map.num_airports, 2);
        let (_, generated_coord) = &game.map.airports[1];
        assert!(generated_coord.x != 0.0 || generated_coord.y != 0.0);

        let passenger_order = game.map.airports[0]
            .0
            .orders
            .iter()
            .find(|o| matches!(o.payload, OrderPayload::Passengers { .. }))
            .expect("passenger order should exist");
        assert_eq!(passenger_order.origin_id, 0);

        let cargo_order = game.map.airports[1]
            .0
            .orders
            .iter()
            .find(|o| matches!(o.payload, OrderPayload::Cargo { .. }))
            .expect("cargo order should exist");
        assert_eq!(cargo_order.origin_id, 1);
    }

    #[test]
    fn from_config_rejects_negative_cargo_value() {
        let mut gameplay = base_gameplay_config();
        gameplay.orders.regenerate = false;
        gameplay.orders.generate_initial = false;

        let airports = vec![
            manual_airport(
                0,
                "HUB",
                Some(Location { x: 100.0, y: 200.0 }),
                vec![ManualOrderConfig::Cargo {
                    cargo: CargoType::Machines,
                    weight: 400.0,
                    value: -10.0,
                    deadline_hours: 12,
                    destination_id: 1,
                }],
            ),
            manual_airport(1, "AAY", Some(Location { x: 400.0, y: 800.0 }), Vec::new()),
        ];

        let cfg = WorldConfig {
            seed: Some(5),
            starting_cash: 400_000.0,
            airports,
            num_airports: None,
            gameplay,
            airplanes: None,
        };

        let err = Game::from_config(cfg).unwrap_err();
        if let GameError::InvalidConfig { msg } = err {
            assert!(msg.contains("negative value"));
        } else {
            panic!("Expected invalid config error");
        }
    }

    #[test]
    fn save_and_load_game_roundtrip() {
        let tmp = tempdir().expect("tempdir");
        let original = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("chdir");

        let mut game = Game::new(21, Some(2), 123_456.0);
        game.player.cash = 222_222.0;
        game.save_game("roundtrip").expect("save to succeed");

        let loaded = Game::load_game("roundtrip").expect("load to succeed");
        assert_eq!(loaded.player.cash, 222_222.0);

        std::env::set_current_dir(&original).expect("restore cwd");
    }

    #[test]
    fn drain_log_and_reset_runtime_clear_state() {
        let mut game = Game::new(4, Some(2), 50_000.0);
        game.log.push("alpha".into());
        assert_eq!(game.drain_log(), vec!["alpha".to_string()]);
        assert!(game.log.is_empty());

        game.log.push("beta".into());
        game.reset_runtime();
        assert!(game.log.is_empty());
    }

    #[test]
    fn observe_reports_plane_payloads() {
        let mut game = Game::new(12, Some(2), 90_000.0);
        let dest_coord = game.map.airports[1].1;
        let origin_coord = game.map.airports[0].1;
        game.airplanes[0].current_passengers = 8;
        game.airplanes[0].current_payload = 450.0;
        game.airplanes[0].status = AirplaneStatus::InTransit {
            hours_remaining: 2,
            destination: 1,
            origin: origin_coord,
            total_hours: 3,
        };

        let obs = game.observe();
        assert_eq!(obs.planes.len(), game.airplanes.len());
        let plane = &obs.planes[0];
        assert_eq!(plane.payload.passenger_current, 8);
        assert_eq!(plane.destination, Some(1));
        assert_eq!(plane.hours_remaining, Some(2));
        assert!(
            obs.airports
                .iter()
                .any(|a| a.id == 1 && (a.x - dest_coord.x).abs() < 1e-3)
        );
    }

    #[test]
    fn tick_event_handles_global_world_event() {
        let mut game = Game::new(33, Some(3), 120_000.0);
        let baseline: Vec<f32> = game
            .map
            .airports
            .iter()
            .map(|(ap, _)| ap.fuel_price)
            .collect();
        game.events.clear();
        game.schedule(
            game.time,
            Event::WorldEvent {
                airport: None,
                factor: 1.4,
                duration: 6,
            },
        );

        assert!(game.tick_event());
        for ((airport, _), before) in game.map.airports.iter().zip(baseline.iter()) {
            assert!((airport.fuel_price - before * 1.4).abs() < 1e-3);
        }

        game.schedule(
            game.time + 6,
            Event::WorldEventEnd {
                airport: None,
                factor: 1.4,
            },
        );
        game.time += 6;
        assert!(game.tick_event());
    }

    #[test]
    fn tick_event_updates_daily_stats_and_pricing() {
        let mut game = Game::new(7, Some(2), 80_000.0);
        game.events.clear();
        let base = game.map.airports[0].0.fuel_price;
        game.daily_income = 600.0;
        game.daily_expenses = 200.0;
        game.schedule(game.time, Event::DailyStats);
        game.schedule(game.time, Event::DynamicPricing);

        assert!(game.tick_event());
        assert!(game.tick_event());

        assert_eq!(game.stats.len(), 1);
        assert!(game.daily_income.abs() < f32::EPSILON);
        assert!(
            game.map
                .airports
                .iter()
                .any(|(ap, _)| (ap.fuel_price - base).abs() > 1e-3)
        );
    }

    #[test]
    fn maintenance_check_marks_and_repairs_planes() {
        let mut game = Game::new(5, Some(2), 70_000.0);
        game.events.clear();
        game.airplanes[0].hours_since_maintenance = 2_000;
        game.airplanes[0].status = AirplaneStatus::Parked;
        game.schedule(game.time, Event::MaintenanceCheck);

        assert!(game.tick_event());
        assert!(game.airplanes[0].needs_maintenance);
        assert!(matches!(game.airplanes[0].status, AirplaneStatus::Broken));

        game.advance(10);
        assert!(!game.airplanes[0].needs_maintenance);
        assert!(matches!(game.airplanes[0].status, AirplaneStatus::Parked));
    }

    #[cfg(feature = "ui_prints")]
    #[test]
    fn list_airport_valid_and_invalid() {
        let mut game = Game::new(16, Some(2), 65_000.0);
        game.map.airports[0].0.orders.push(Order {
            id: 999,
            payload: OrderPayload::Passengers { count: 10 },
            value: 3_500.0,
            deadline: 12,
            origin_id: 0,
            destination_id: 1,
        });
        game.show_cash();
        game.show_time();
        game.show_stats();

        game.list_airports(true);
        game.list_airport(0, true).expect("airport exists");

        let err = game.list_airport(10, false).unwrap_err();
        assert!(matches!(err, GameError::AirportIdInvalid { .. }));
    }
}

fn default_restock_cycle() -> GameTime {
    DEFAULT_RESTOCK_CYCLE
}

fn default_fuel_interval() -> GameTime {
    DEFAULT_FUEL_INTERVAL
}

fn default_regenerate_orders() -> bool {
    true
}

fn default_arrival_times() -> HashMap<usize, GameTime> {
    HashMap::new()
}

fn deserialize_arrival_times<'de, D>(deserializer: D) -> Result<HashMap<usize, GameTime>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ArrivalTimesSerde {
        Map(HashMap<usize, GameTime>),
        MapString(HashMap<String, GameTime>),
        Vec(Vec<GameTime>),
    }

    let parsed = ArrivalTimesSerde::deserialize(deserializer)?;
    Ok(match parsed {
        ArrivalTimesSerde::Map(map) => map,
        ArrivalTimesSerde::MapString(map) => map
            .into_iter()
            .filter_map(|(k, v)| k.parse::<usize>().ok().map(|idx| (idx, v)))
            .collect(),
        ArrivalTimesSerde::Vec(vec) => vec.into_iter().enumerate().collect::<HashMap<_, _>>(),
    })
}

fn default_fuel_settings() -> FuelGameplay {
    FuelGameplay::default()
}

fn gameplay_settings(
    cfg: &GameplayConfig,
) -> Result<
    (
        DemandGenerationParams,
        GameTime,
        GameTime,
        bool,
        bool,
        FuelGameplay,
    ),
    String,
> {
    let tuning = &cfg.orders.tuning;
    let passenger_tuning = &cfg.orders.passengers;
    if tuning.max_deadline_hours == 0 {
        return Err("orders.max_deadline_hours must be at least 1".into());
    }
    if tuning.min_weight <= 0.0 {
        return Err("orders.min_weight must be greater than 0".into());
    }
    if tuning.max_weight < tuning.min_weight {
        return Err("orders.max_weight must be >= orders.min_weight".into());
    }
    if passenger_tuning.max_deadline_hours == 0 {
        return Err("orders.passengers.max_deadline_hours must be at least 1".into());
    }
    if passenger_tuning.min_count == 0 {
        return Err("orders.passengers.min_count must be greater than 0".into());
    }
    if passenger_tuning.max_count < passenger_tuning.min_count {
        return Err("orders.passengers.max_count must be >= min_count".into());
    }
    if passenger_tuning.fare_per_km <= 0.0 {
        return Err("orders.passengers.fare_per_km must be greater than 0".into());
    }
    if cfg.restock_cycle_hours == 0 {
        return Err("restock_cycle_hours must be at least 1".into());
    }
    if cfg.fuel_interval_hours == 0 {
        return Err("fuel_interval_hours must be at least 1".into());
    }

    if cfg.fuel.elasticity <= 0.0 {
        return Err("fuel.elasticity must be greater than 0".into());
    }
    if cfg.fuel.elasticity >= 1.0 {
        return Err("fuel.elasticity must be less than 1".into());
    }
    if cfg.fuel.min_price_multiplier <= 0.0 {
        return Err("fuel.min_price_multiplier must be greater than 0".into());
    }
    if cfg.fuel.max_price_multiplier < cfg.fuel.min_price_multiplier {
        return Err("fuel.max_price_multiplier must be >= fuel.min_price_multiplier".into());
    }
    if cfg.fuel.max_price_multiplier <= 1.0 {
        return Err("fuel.max_price_multiplier must be greater than 1".into());
    }

    let order_params = OrderGenerationParams::from(tuning);
    let passenger_params = PassengerGenerationParams::from(passenger_tuning);
    let demand_params = DemandGenerationParams {
        cargo: order_params,
        passengers: passenger_params,
    };
    Ok((
        demand_params,
        cfg.restock_cycle_hours,
        cfg.fuel_interval_hours,
        cfg.orders.regenerate,
        cfg.orders.generate_initial,
        cfg.fuel.clone(),
    ))
}

/// Holds all mutable world state and drives the simulation via scheduled events.
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    /// Current simulation time (hours)
    pub time: GameTime,
    /// The world map of airports and coordinates
    pub map: Map,
    /// All airplanes in the world
    pub airplanes: Vec<Airplane>,
    /// Tracker for each plane's last arrival time
    #[serde(
        default = "default_arrival_times",
        deserialize_with = "deserialize_arrival_times"
    )]
    pub arrival_times: HashMap<usize, GameTime>,
    /// The player's company (cash, fleet, deliveries)
    pub player: Player,
    /// Future events, ordered by their `time` (earliest first)
    pub events: BinaryHeap<ScheduledEvent>,
    /// Income over each day
    pub daily_income: f32,
    /// Expenses over each day
    pub daily_expenses: f32,
    /// History of all stats
    pub stats: Vec<DailyStats>,
    /// Seed used to create the RNG for deterministic behaviour
    pub seed: u64,
    /// Frequency (in hours) for restocking airports
    #[serde(default = "default_restock_cycle")]
    pub restock_cycle: GameTime,
    /// Frequency (in hours) for dynamic fuel price adjustments
    #[serde(default = "default_fuel_interval")]
    pub fuel_interval: GameTime,
    /// Fuel pricing behavior parameters
    #[serde(default = "default_fuel_settings")]
    pub fuel_settings: FuelGameplay,
    /// Whether dynamic restocking is enabled for this save
    #[serde(default = "default_regenerate_orders")]
    pub regenerate_orders: bool,
    /// Game-local random number generator to avoid global RNG usage
    #[serde(skip, default = "default_rng")]
    rng: StdRng,
    /// Log of messages generated during play
    #[serde(skip, default)]
    log: Vec<String>,
    /// Available airplane catalog for purchases and starter selection.
    #[serde(default = "default_model_catalog")]
    model_catalog: HashMap<String, AirplaneSpecs>,
    /// If true, only models in `model_catalog` are allowed (replace mode)
    #[serde(default)]
    models_replace: bool,
}

#[derive(Serialize)]
pub struct Observation {
    pub time: u64,
    pub cash: f32,
    pub airports: Vec<AirportObs>,
    pub planes: Vec<PlaneObs>,
}

#[derive(Serialize)]
pub struct AirportObs {
    pub id: usize,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub fuel_price: f32,
    pub runway_length: f32,
    pub num_orders: usize,
}

#[derive(Serialize)]
pub struct PlaneObs {
    pub id: usize,
    pub model: String,
    pub x: f32,
    pub y: f32,
    pub status: String,
    pub fuel: FuelObs,
    pub payload: PayloadObs,
    pub destination: Option<usize>,
    pub hours_remaining: Option<u64>,
}

#[derive(Serialize)]
pub struct FuelObs {
    pub current: f32,
    pub capacity: f32,
}

#[derive(Serialize)]
pub struct PayloadObs {
    pub cargo_current: f32,
    pub cargo_capacity: f32,
    pub passenger_current: u32,
    pub passenger_capacity: u32,
}

impl Game {
    /// Initialize a new game with `num_airports`, seeded randomness, and player's starting cash.
    ///
    /// Parameters
    /// - `seed`: Random seed used for deterministic world generation.
    /// - `num_airports`: Number of airports to generate (use `None` for default).
    /// - `starting_cash`: Cash balance for the player at the start.
    ///
    /// Returns
    /// - `Game`: A fully initialized simulation ready to run.
    ///
    /// Example
    /// ```
    /// use rusty_runways_core::Game;
    /// let mut game = Game::new(123, Some(5), 650_000.0);
    /// assert_eq!(game.airports().len(), 5);
    /// game.advance(1);
    /// ```
    pub fn new(seed: u64, num_airports: Option<usize>, starting_cash: f32) -> Self {
        let mut map = Map::generate_from_seed(seed, num_airports);
        for (airport, _) in map.airports.iter_mut() {
            airport.ensure_base_fuel_price();
        }

        let player = Player::new(starting_cash, &map);
        let airplanes = player.fleet.clone();
        let arrival_times = airplanes
            .iter()
            .map(|plane| (plane.id, 0))
            .collect::<HashMap<_, _>>();
        let events = BinaryHeap::new();

        let mut game = Game {
            time: 0,
            map,
            airplanes,
            player,
            events,
            arrival_times,
            daily_income: 0.0,
            daily_expenses: 0.0,
            stats: Vec::new(),
            seed,
            restock_cycle: DEFAULT_RESTOCK_CYCLE,
            fuel_interval: DEFAULT_FUEL_INTERVAL,
            fuel_settings: FuelGameplay::default(),
            regenerate_orders: true,
            rng: StdRng::seed_from_u64(seed),
            log: Vec::new(),
            model_catalog: default_model_catalog(),
            models_replace: false,
        };

        for (airport, _) in game.map.airports.iter_mut() {
            airport.ensure_base_fuel_price();
        }

        game.schedule(game.restock_cycle, Event::Restock);
        game.schedule(REPORT_INTERVAL, Event::DailyStats);
        game.schedule(game.fuel_interval, Event::DynamicPricing);
        game.schedule_world_event();
        game.schedule(1, Event::MaintenanceCheck);

        game
    }

    /// Initialize a game from a configuration (airports explicitly provided).
    ///
    /// Parameters
    /// - `cfg`: A [`WorldConfig`](crate::config::WorldConfig) with airports and gameplay parameters.
    ///
    /// Returns
    /// - `Ok(Game)` if the configuration is valid.
    /// - `Err(GameError)` if validation fails (e.g. duplicate IDs).
    ///
    /// Example
    /// ```
    /// use rusty_runways_core::{Game, config::{WorldConfig, GameplayConfig}};
    /// let cfg = WorldConfig {
    ///     seed: Some(1),
    ///     starting_cash: 650_000.0,
    ///     airports: vec![],
    ///     num_airports: Some(4),
    ///     gameplay: GameplayConfig::default(),
    ///     airplanes: None,
    /// };
    /// let game = Game::from_config(cfg).unwrap();
    /// assert_eq!(game.airports().len(), 4);
    /// ```
    pub fn from_config(cfg: WorldConfig) -> Result<Self, GameError> {
        let seed = cfg.seed.unwrap_or(0);
        let (
            demand_params,
            restock_cycle,
            fuel_interval,
            regenerate_orders,
            generate_initial_orders,
            fuel_settings,
        ) = gameplay_settings(&cfg.gameplay).map_err(|msg| GameError::InvalidConfig { msg })?;

        let have_explicit_airports = !cfg.airports.is_empty();
        if have_explicit_airports && cfg.num_airports.is_some() {
            return Err(GameError::InvalidConfig {
                msg: "num_airports cannot be provided when airports are explicitly listed".into(),
            });
        }
        if !have_explicit_airports && cfg.num_airports.is_none() {
            return Err(GameError::InvalidConfig {
                msg: "num_airports must be provided when airports list is empty".into(),
            });
        }

        let map = if have_explicit_airports {
            use std::collections::HashSet;

            let mut ids = HashSet::new();
            let mut names = HashSet::new();
            for a in &cfg.airports {
                if !ids.insert(a.id) {
                    return Err(GameError::InvalidConfig {
                        msg: format!("duplicate airport id {}", a.id),
                    });
                }
                let lower = a.name.to_lowercase();
                if !names.insert(lower) {
                    return Err(GameError::InvalidConfig {
                        msg: format!("duplicate airport name '{}'", a.name),
                    });
                }
            }

            let mut airports_vec = Vec::with_capacity(cfg.airports.len());
            let mut next_order_id = 0usize;
            let missing_coords: Vec<usize> = cfg
                .airports
                .iter()
                .enumerate()
                .filter_map(|(idx, a)| {
                    if a.location.is_some() {
                        None
                    } else {
                        Some(idx)
                    }
                })
                .collect();
            let generated_coords = if missing_coords.is_empty() {
                Vec::new()
            } else {
                Map::generate_clustered_coordinates(seed.wrapping_add(13), missing_coords.len())
            };

            for (idx, a) in cfg.airports.iter().enumerate() {
                if let Some(len) = a.runway_length_m {
                    if len <= 0.0 {
                        return Err(GameError::InvalidConfig {
                            msg: format!("airport {} runway_length must be > 0", a.id),
                        });
                    }
                }
                if let Some(price) = a.fuel_price_per_l {
                    if price <= 0.0 {
                        return Err(GameError::InvalidConfig {
                            msg: format!("airport {} fuel_price_per_l must be > 0", a.id),
                        });
                    }
                }
                if let Some(fee) = a.landing_fee_per_ton {
                    if fee < 0.0 {
                        return Err(GameError::InvalidConfig {
                            msg: format!("airport {} landing_fee_per_ton must be >= 0", a.id),
                        });
                    }
                }
                if let Some(fee) = a.parking_fee_per_hour {
                    if fee < 0.0 {
                        return Err(GameError::InvalidConfig {
                            msg: format!("airport {} parking_fee_per_hour must be >= 0", a.id),
                        });
                    }
                }
                if let Some(loc) = a.location {
                    if !(0.0..=10000.0).contains(&loc.x) || !(0.0..=10000.0).contains(&loc.y) {
                        return Err(GameError::InvalidConfig {
                            msg: format!(
                                "airport {} location ({:.2},{:.2}) out of bounds [0,10000]",
                                a.id, loc.x, loc.y
                            ),
                        });
                    }
                }
                if !regenerate_orders && a.orders.is_empty() {
                    return Err(GameError::InvalidConfig {
                        msg: format!(
                            "airport {} must define at least one order when regeneration is disabled",
                            a.id
                        ),
                    });
                }

                let default_airport = Airport::generate_random(seed, a.id);
                let coord = if let Some(loc) = a.location {
                    Coordinate::new(loc.x, loc.y)
                } else {
                    let generated_idx = missing_coords
                        .iter()
                        .position(|i| *i == idx)
                        .expect("generated coordinate index must exist");
                    generated_coords[generated_idx]
                };

                let runway_length = a.runway_length_m.unwrap_or(default_airport.runway_length);
                let fuel_price = a.fuel_price_per_l.unwrap_or(default_airport.fuel_price);
                let landing_fee = a.landing_fee_per_ton.unwrap_or(default_airport.landing_fee);
                let parking_fee = a
                    .parking_fee_per_hour
                    .unwrap_or(default_airport.parking_fee);

                let mut manual_orders = Vec::with_capacity(a.orders.len());
                for order_cfg in &a.orders {
                    match order_cfg {
                        ManualOrderConfig::Cargo {
                            cargo,
                            weight,
                            value,
                            deadline_hours,
                            destination_id,
                        } => {
                            if *deadline_hours == 0 {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has order with deadline_hours == 0",
                                        a.id
                                    ),
                                });
                            }
                            if *weight <= 0.0 {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has order with non-positive weight",
                                        a.id
                                    ),
                                });
                            }
                            if *value < 0.0 {
                                return Err(GameError::InvalidConfig {
                                    msg: format!("airport {} has order with negative value", a.id),
                                });
                            }
                            if *destination_id == a.id {
                                return Err(GameError::InvalidConfig {
                                    msg: format!("airport {} has order pointing to itself", a.id),
                                });
                            }
                            if !ids.contains(destination_id) {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has order with unknown destination {}",
                                        a.id, destination_id
                                    ),
                                });
                            }

                            manual_orders.push(Order {
                                id: next_order_id,
                                payload: OrderPayload::Cargo {
                                    cargo_type: *cargo,
                                    weight: *weight,
                                },
                                value: *value,
                                deadline: *deadline_hours,
                                origin_id: a.id,
                                destination_id: *destination_id,
                            });
                        }
                        ManualOrderConfig::Passengers {
                            passengers,
                            value,
                            deadline_hours,
                            destination_id,
                        } => {
                            if *deadline_hours == 0 {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has passenger order with deadline_hours == 0",
                                        a.id
                                    ),
                                });
                            }
                            if *passengers == 0 {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has passenger order with zero passengers",
                                        a.id
                                    ),
                                });
                            }
                            if *value < 0.0 {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has passenger order with negative value",
                                        a.id
                                    ),
                                });
                            }
                            if *destination_id == a.id {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has passenger order pointing to itself",
                                        a.id
                                    ),
                                });
                            }
                            if !ids.contains(destination_id) {
                                return Err(GameError::InvalidConfig {
                                    msg: format!(
                                        "airport {} has passenger order with unknown destination {}",
                                        a.id, destination_id
                                    ),
                                });
                            }

                            manual_orders.push(Order {
                                id: next_order_id,
                                payload: OrderPayload::Passengers { count: *passengers },
                                value: *value,
                                deadline: *deadline_hours,
                                origin_id: a.id,
                                destination_id: *destination_id,
                            });
                        }
                    }
                    next_order_id += 1;
                }

                let ap = Airport {
                    id: a.id,
                    name: a.name.clone(),
                    runway_length,
                    fuel_price,
                    base_fuel_price: fuel_price,
                    landing_fee,
                    parking_fee,
                    orders: manual_orders,
                    fuel_sold: 0.0,
                };
                airports_vec.push((ap, coord));
            }

            let mut built =
                Map::from_airports(seed, airports_vec, demand_params.clone(), next_order_id);
            if regenerate_orders && generate_initial_orders {
                built.restock_airports();
            }
            built
        } else {
            let num_airports = cfg.num_airports.unwrap();
            if num_airports == 0 {
                return Err(GameError::InvalidConfig {
                    msg: "num_airports must be greater than 0".into(),
                });
            }
            if !regenerate_orders {
                return Err(GameError::InvalidConfig {
                    msg: "orders.regenerate=false requires explicit airports with manual orders"
                        .into(),
                });
            }

            let mut generated = Map::generate_from_seed(seed, Some(num_airports));
            generated.demand_params = demand_params.clone();
            generated.clear_orders();
            if generate_initial_orders {
                generated.restock_airports();
            }
            generated
        };

        // Build model catalog based on config
        let mut models_replace = false;
        let mut catalog = default_model_catalog();
        if let Some(acfg) = &cfg.airplanes {
            if !acfg.models.is_empty() {
                if matches!(acfg.strategy, AirplaneCatalogStrategy::Replace) {
                    models_replace = true;
                    catalog.clear();
                }
                let mut seen = std::collections::HashSet::new();
                for m in &acfg.models {
                    validate_model_config(m)?;
                    let key = m.name.trim().to_lowercase();
                    if !seen.insert(key) {
                        return Err(GameError::InvalidConfig {
                            msg: format!("duplicate airplane model name '{}'", m.name),
                        });
                    }
                    catalog.insert(m.name.clone(), model_specs_from_config(m));
                }
            }
        }

        let player = Player::new_from_catalog(cfg.starting_cash, &map, &catalog);
        let airplanes = player.fleet.clone();
        let arrival_times = airplanes
            .iter()
            .map(|plane| (plane.id, 0))
            .collect::<HashMap<_, _>>();
        let events = BinaryHeap::new();

        let mut game = Game {
            time: 0,
            map,
            airplanes,
            player,
            events,
            arrival_times,
            daily_income: 0.0,
            daily_expenses: 0.0,
            stats: Vec::new(),
            seed,
            restock_cycle,
            fuel_interval,
            fuel_settings,
            regenerate_orders,
            rng: StdRng::seed_from_u64(seed),
            log: Vec::new(),
            model_catalog: catalog,
            models_replace,
        };

        for (airport, _) in game.map.airports.iter_mut() {
            airport.ensure_base_fuel_price();
        }

        if game.regenerate_orders {
            game.schedule(game.restock_cycle, Event::Restock);
        }
        game.schedule(REPORT_INTERVAL, Event::DailyStats);
        game.schedule(game.fuel_interval, Event::DynamicPricing);
        game.schedule_world_event();
        game.schedule(1, Event::MaintenanceCheck);

        Ok(game)
    }

    /// Return the seed used to initialize this game.
    ///
    /// Returns
    /// - `u64`: The configured deterministic seed.
    ///
    /// Example
    /// ```
    /// let game = rusty_runways_core::Game::new(7, Some(3), 1.0);
    /// assert_eq!(game.seed(), 7);
    /// ```
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Drain the internal log, returning all messages collected so far.
    ///
    /// Returns
    /// - `Vec<String>`: Log messages since last drain; subsequent calls return empty until new logs appear.
    ///
    /// Example
    /// ```
    /// let mut game = rusty_runways_core::Game::new(1, Some(3), 0.0);
    /// let _first = game.drain_log();
    /// // ... after some actions ...
    /// let _second = game.drain_log();
    /// ```
    pub fn drain_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.log)
    }

    /// Reinitialize runtime-only fields after deserializing.
    ///
    /// This resets the internal RNG and clears transient logs without touching game state.
    ///
    /// Example
    /// ```
    /// let mut game = rusty_runways_core::Game::new(1, Some(3), 0.0);
    /// game.reset_runtime();
    /// ```
    pub fn reset_runtime(&mut self) {
        self.rng = StdRng::seed_from_u64(self.seed);
        self.log.clear();
    }

    fn days_and_hours(&self, total_hours: GameTime) -> String {
        let days = total_hours / 24;
        let hours = total_hours % 24;

        match (days, hours) {
            (0, h) => format!("{}h", h),
            (d, 0) => format!("{}d", d),
            (d, h) => format!("{}d {}h", d, h),
        }
    }

    fn schedule_world_event(&mut self) {
        // event every 4 to 5 days
        let next_start = self.time + self.rng.gen_range(96..=120);

        // 1/8 chance it is global
        let is_global = self.rng.gen_bool(0.125);
        let airport = if is_global {
            None
        } else {
            Some(self.rng.gen_range(0..self.map.num_airports))
        };

        // price can spike or crash
        let factor = if self.rng.gen_bool(0.5) {
            self.rng.gen_range(1.2..=1.5)
        } else {
            self.rng.gen_range(0.5..=0.8)
        };

        // lasts 12 - 72 hours
        let duration = self.rng.gen_range(24..72);
        self.schedule(
            self.time + next_start,
            Event::WorldEvent {
                airport,
                factor,
                duration,
            },
        );
    }

    /// Write the entire game state to JSON to save.
    ///
    /// Parameters
    /// - `name`: Logical save name (without extension); stored under `save_games/`.
    ///
    /// Returns
    /// - `io::Result<()>`: Errors if directories/files cannot be created or written.
    ///
    /// Example
    /// ```no_run
    /// let game = rusty_runways_core::Game::new(1, Some(3), 0.0);
    /// game.save_game("my-save").unwrap();
    /// ```
    pub fn save_game(&self, name: &str) -> io::Result<()> {
        let save_dir = Path::new("save_games");
        fs::create_dir_all(save_dir)?;

        let mut path = PathBuf::from(save_dir);
        path.push(format!("{}.json", name));

        let file = fs::File::create(&path)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).map_err(io::Error::other)
    }

    fn refresh_airplane_specs(&mut self) {
        for plane in &mut self.airplanes {
            let specs = plane.model.specs();
            plane.specs = specs;
            if plane.current_passengers > specs.passenger_capacity {
                plane.current_passengers = specs.passenger_capacity;
            }
            if plane.current_payload > specs.payload_capacity {
                plane.current_payload = specs.payload_capacity;
            }
        }
        for plane in &mut self.player.fleet {
            let specs = plane.model.specs();
            plane.specs = specs;
            if plane.current_passengers > specs.passenger_capacity {
                plane.current_passengers = specs.passenger_capacity;
            }
            if plane.current_payload > specs.payload_capacity {
                plane.current_payload = specs.payload_capacity;
            }
        }
    }

    /// Load a game from JSON.
    ///
    /// Parameters
    /// - `name`: Logical save name (without extension) previously used in [`Game::save_game`].
    ///
    /// Returns
    /// - `Ok(Game)`: Loaded game.
    /// - `Err(io::Error)`: If the file does not exist or has invalid content.
    ///
    /// Example
    /// ```no_run
    /// let game = rusty_runways_core::Game::load_game("my-save").unwrap();
    /// ```
    pub fn load_game(name: &str) -> io::Result<Self> {
        let mut path = PathBuf::from("save_games");
        path.push(format!("{}.json", name));

        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Save file '{}' not found", path.display()),
            ));
        }

        let file = fs::File::open(&path)?;
        let reader = io::BufReader::new(file);
        let mut game: Game = serde_json::from_reader(reader).map_err(io::Error::other)?;
        game.refresh_airplane_specs();
        Ok(game)
    }

    /// Schedule `event` to occur at absolute simulation time `time`.
    ///
    /// Parameters
    /// - `time`: Absolute time (hours).
    /// - `event`: The event to enqueue.
    fn schedule(&mut self, time: GameTime, event: Event) {
        self.events.push(ScheduledEvent { time, event });
    }

    /// Show current player cash
    #[cfg(feature = "ui_prints")]
    pub fn show_cash(&self) {
        println!("${}", self.player.cash);
    }

    /// Show current time
    #[cfg(feature = "ui_prints")]
    pub fn show_time(&self) {
        println!("{}", self.days_and_hours(self.time));
    }

    /// Shows the lifetime stats
    #[cfg(feature = "ui_prints")]
    pub fn show_stats(&self) {
        let headers = ["Day", "Income", "Expense", "End Cash", "Fleet", "Delivered"];

        //get max width per column
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(self.stats.len());

        for s in &self.stats {
            let row = vec![
                s.day.to_string(),
                format!("{:.2}", s.income),
                format!("{:.2}", s.expenses),
                format!("{:.2}", s.net_cash),
                s.fleet_size.to_string(),
                s.total_deliveries.to_string(),
            ];

            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
            rows.push(row);
        }

        for (i, header) in headers.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            // left-align
            print!("{:<width$}", header, width = col_widths[i]);
        }
        println!();

        // Separator
        let total_width: usize = col_widths.iter().sum::<usize>() + (3 * (headers.len() - 1));
        println!("{}", "-".repeat(total_width));

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    print!(" | ");
                }

                // right-align
                print!("{:>width$}", cell, width = col_widths[i]);
            }
            println!();
        }
    }

    /// Process the next scheduled event; advance `self.time`. Returns false if no events remain.
    pub fn tick_event(&mut self) -> bool {
        if let Some(scheduled) = self.events.pop() {
            // advance time
            self.time = scheduled.time;

            match scheduled.event {
                // Restock every 14 days
                Event::Restock => {
                    if self.regenerate_orders {
                        self.map.restock_airports();
                        self.schedule(self.time + self.restock_cycle, Event::Restock);
                    }
                }

                // Finished loading, therefore we need to update the status
                Event::LoadingEvent { plane } => {
                    self.airplanes[plane].status = AirplaneStatus::Parked;
                }

                // Update the progress of the flight
                Event::FlightProgress { plane } => {
                    // buffer for events
                    let mut to_schedule: Vec<(GameTime, Event)> = Vec::new();

                    {
                        let airplane = &mut self.airplanes[plane];

                        if let AirplaneStatus::InTransit {
                            hours_remaining,
                            destination,
                            origin,
                            total_hours,
                        } = airplane.status
                        {
                            let dest_coord = self.map.airports[destination].1;
                            let hours_elapsed = total_hours - hours_remaining + 1;
                            let fraction = (hours_elapsed as f32) / (total_hours as f32);

                            airplane.location = Coordinate {
                                x: origin.x + (dest_coord.x - origin.x) * fraction,
                                y: origin.y + (dest_coord.y - origin.y) * fraction,
                            };

                            if hours_remaining > 1 {
                                airplane.status = AirplaneStatus::InTransit {
                                    hours_remaining: hours_remaining - 1,
                                    destination,
                                    origin,
                                    total_hours,
                                };

                                // still in transit
                                to_schedule.push((self.time + 1, Event::FlightProgress { plane }));
                            } else {
                                // landing
                                let (airport, _) = &self.map.airports[destination];
                                let landing_fee = airport.landing_fee(airplane);
                                self.player.cash -= landing_fee;
                                self.daily_expenses += landing_fee;

                                self.arrival_times.insert(plane, self.time);
                                airplane.location = self.map.airports[destination].1;

                                if airplane.needs_maintenance {
                                    airplane.status = AirplaneStatus::Broken;
                                    to_schedule.push((self.time + 8, Event::Maintenance { plane }));
                                } else {
                                    airplane.status = AirplaneStatus::Parked;
                                }
                            }
                        }
                    }

                    // Schedule new events
                    for (when, ev) in to_schedule {
                        self.schedule(when, ev);
                    }
                }

                Event::RefuelComplete { plane } => {
                    self.airplanes[plane].status = AirplaneStatus::Parked;
                }

                Event::DailyStats => {
                    let day = self.time / 24;
                    self.stats.push(DailyStats {
                        day,
                        income: self.daily_income,
                        expenses: self.daily_expenses,
                        net_cash: self.player.cash,
                        fleet_size: self.player.fleet_size,
                        total_deliveries: self.player.orders_delivered,
                    });

                    //reset
                    self.daily_income = 0.0;
                    self.daily_expenses = 0.0;

                    self.schedule(self.time + REPORT_INTERVAL, Event::DailyStats);
                }

                Event::DynamicPricing => {
                    // Adjust prices across the board
                    let settings = &self.fuel_settings;
                    for (airport, _) in self.map.airports.iter_mut() {
                        airport.adjust_fuel_price(
                            settings.elasticity,
                            settings.min_price_multiplier,
                            settings.max_price_multiplier,
                        );
                    }

                    // Schedule next
                    self.schedule(self.time + self.fuel_interval, Event::DynamicPricing);
                }

                Event::WorldEvent {
                    airport,
                    factor,
                    duration,
                } => {
                    match airport {
                        Some(airport_id) => {
                            self.map.airports[airport_id].0.fuel_price *= factor;
                            let name = &self.map.airports[airport_id].0.name;

                            let pct = (factor - 1.0) * 100.0;
                            println!(
                                "Fuel price spike of +{:.0}% at {} for {}h!",
                                pct, name, duration
                            );
                        }
                        None => {
                            for (airport, _) in &mut self.map.airports {
                                airport.fuel_price *= factor
                            }

                            let pct = (factor - 1.0) * 100.0;
                            println!("Global fuel price spike of +{:.0}% for {}h!", pct, duration);
                        }
                    }

                    let event_end = self.time + duration;
                    self.schedule(event_end, Event::WorldEventEnd { airport, factor });
                }

                // Reset world event
                Event::WorldEventEnd { airport, factor } => {
                    match airport {
                        Some(airport_id) => {
                            self.map.airports[airport_id].0.fuel_price /= factor;

                            let name = &self.map.airports[airport_id].0.name;
                            let pct = (factor - 1.0) * 100.0;
                            println!("Fuel price spike of +{:.0}% at {} has ended.", pct, name);
                        }
                        None => {
                            for (airport, _) in &mut self.map.airports {
                                airport.fuel_price /= factor
                            }
                            let pct = (factor - 1.0) * 100.0;
                            println!("Global fuel price spike of +{:.0}% has ended.", pct);
                        }
                    }

                    // schedule the next event
                    self.schedule_world_event();
                }

                Event::MaintenanceCheck => {
                    // Collect vec of planes that are broken:
                    let mut just_broke = Vec::new();

                    for (idx, airplane) in self.airplanes.iter_mut().enumerate() {
                        if airplane.status != AirplaneStatus::Maintenance {
                            airplane.add_hours_since_maintenance();
                            let p_fail = airplane.risk_of_failure();
                            if self.rng.gen_bool(p_fail as f64) {
                                airplane.needs_maintenance = true;
                                if matches!(
                                    airplane.status,
                                    AirplaneStatus::Parked
                                        | AirplaneStatus::Loading
                                        | AirplaneStatus::Unloading
                                        | AirplaneStatus::Refueling
                                ) {
                                    airplane.status = AirplaneStatus::Broken;
                                    just_broke.push(idx);
                                }
                            }
                        }
                    }

                    // Schedule broken events
                    for idx in just_broke {
                        self.schedule(self.time + 8, Event::Maintenance { plane: idx });
                    }

                    // next check
                    self.schedule(self.time + 1, Event::MaintenanceCheck);
                }

                Event::Maintenance { plane } => {
                    let airplane = &mut self.airplanes[plane];
                    airplane.status = AirplaneStatus::Parked;
                    airplane.hours_since_maintenance = 0;
                    airplane.needs_maintenance = false;
                }

                _ => {
                    println!("Not implemented!")
                }
            }

            true
        } else {
            false
        }
    }

    /// Run the simulation until `max_time` or until there are no more events.
    ///
    /// Parameters
    /// - `max_time`: Absolute target time (hours) to fast-forward to.
    ///
    /// Example
    /// ```
    /// let mut game = rusty_runways_core::Game::new(1, Some(3), 0.0);
    /// game.run_until(24);
    /// ```
    pub fn run_until(&mut self, max_time: GameTime) {
        while self.time < max_time && self.tick_event() {}

        //if no events, just jump to time step
        if self.time < max_time {
            self.time = max_time;
        }
    }

    /// Advance the simulation clock by `hours`, processing due events as you go.
    ///
    /// Parameters
    /// - `hours`: Number of hours to advance.
    ///
    /// Example
    /// ```
    /// let mut game = rusty_runways_core::Game::new(1, Some(3), 0.0);
    /// game.advance(6);
    /// ```
    pub fn advance(&mut self, hours: GameTime) {
        let target = self.time + hours;

        // Keep processing events in time order until we're past `target`
        while let Some(ev) = self.events.peek() {
            if ev.time <= target {
                self.tick_event();
            } else {
                break;
            }
        }

        // Finally bump the clock
        self.time = target;
    }

    /// Display a summary of all airports in the map, including their orders.
    /// If with_orders is true, show the orders alongside.
    #[cfg(feature = "ui_prints")]
    pub fn list_airports(&self, with_orders: bool) {
        println!("Airports ({} total):", self.map.num_airports);
        for (airport, coord) in &self.map.airports {
            println!(
                "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Fuel: ${:.2}/L | Parking: ${:.2}/hr | Landing Fee: ${:.2}/ton",
                airport.id,
                airport.name,
                coord.x,
                coord.y,
                airport.runway_length,
                airport.fuel_price,
                airport.parking_fee,
                airport.landing_fee,
            );
            if with_orders {
                if airport.orders.is_empty() {
                    println!("  No pending orders.");
                } else {
                    println!("  Orders:");
                    for order in &airport.orders {
                        let payload_info = match &order.payload {
                            OrderPayload::Cargo { cargo_type, weight } => {
                                format!("{:?} | weight: {:.1}kg", cargo_type, weight)
                            }
                            OrderPayload::Passengers { count } => {
                                format!("Passengers | count: {}", count)
                            }
                        };
                        println!(
                            "    [{}] {} -> {} | value: ${:.2} | deadline: {} | destination: {}",
                            order.id,
                            payload_info,
                            self.map.airports[order.destination_id].0.name,
                            order.value,
                            order.deadline,
                            order.destination_id
                        );
                    }
                }
            }
        }
    }

    /// Display a summary of a single airport in the map, including its orders.
    /// If with_orders is true, show the orders alongside.
    #[cfg(feature = "ui_prints")]
    pub fn list_airport(&self, airport_id: usize, with_orders: bool) -> Result<(), GameError> {
        if airport_id > (self.map.num_airports - 1) {
            return Err(GameError::AirportIdInvalid { id: airport_id });
        }

        let (airport, coord) = &self.map.airports[airport_id];
        println!(
            "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Fuel: ${:.2}/L | Parking: ${:.2}/hr | Landing Fee: ${:.2}/ton",
            airport.id,
            airport.name,
            coord.x,
            coord.y,
            airport.runway_length,
            airport.fuel_price,
            airport.parking_fee,
            airport.landing_fee,
        );
        if with_orders {
            if airport.orders.is_empty() {
                println!("  No pending orders.");
            } else {
                println!("  Orders:");
                for order in &airport.orders {
                    let payload_info = match &order.payload {
                        OrderPayload::Cargo { cargo_type, weight } => {
                            format!("{:?} | weight: {:.1}kg", cargo_type, weight)
                        }
                        OrderPayload::Passengers { count } => {
                            format!("Passengers | count: {}", count)
                        }
                    };
                    println!(
                        "    [{}] {} -> {} | value: ${:.2} | deadline: {} | destination: {}",
                        order.id,
                        payload_info,
                        self.map.airports[order.destination_id].0.name,
                        order.value,
                        self.days_and_hours(order.deadline),
                        order.destination_id
                    );
                }
            }
        }

        Ok(())
    }

    #[cfg(feature = "ui_prints")]
    fn find_associated_airport(&self, location: &Coordinate) -> Result<String, GameError> {
        let airport = match self.map.airports.iter().find(|(_, c)| c == location) {
            Some((airport, _)) => airport,
            _ => {
                return Err(GameError::AirportLocationInvalid {
                    location: *location,
                });
            }
        };

        Ok(airport.name.clone())
    }

    /// Locate a plane and the index of the airport where it is currently parked.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if no plane with `plane_id` exists or
    /// [`GameError::PlaneNotAtAirport`] if the plane is not located at any airport.
    fn plane_and_airport_idx(&self, plane_id: usize) -> Result<(usize, usize), GameError> {
        let plane_index = self
            .airplanes
            .iter()
            .position(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let location = self.airplanes[plane_index].location;
        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        Ok((plane_index, airport_idx))
    }

    /// Display a summary of all airplanes in the game.
    #[cfg(feature = "ui_prints")]
    pub fn list_airplanes(&self) -> Result<(), GameError> {
        println!("Airplanes ({} total):", self.airplanes.len());
        for plane in &self.airplanes {
            if let AirplaneStatus::InTransit {
                hours_remaining,
                destination,
                ..
            } = plane.status
            {
                let dest_name = &self.map.airports[destination].0.name;
                println!(
                    "ID: {} | {:?} en-route to airport {} | Location: ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: InTransit - arrival in {}",
                    plane.id,
                    plane.model,
                    dest_name,
                    plane.location.x,
                    plane.location.y,
                    plane.current_fuel,
                    plane.specs.fuel_capacity,
                    plane.current_payload,
                    plane.specs.payload_capacity,
                    self.days_and_hours(hours_remaining)
                );
            } else {
                let loc = &plane.location;
                let airport_name = self.find_associated_airport(loc)?;
                println!(
                    "ID: {} | {:?} at airport {} ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: {:?}",
                    plane.id,
                    plane.model,
                    airport_name,
                    loc.x,
                    loc.y,
                    plane.current_fuel,
                    plane.specs.fuel_capacity,
                    plane.current_payload,
                    plane.specs.payload_capacity,
                    plane.status,
                );
            }
        }

        Ok(())
    }

    /// Display a summary of a single airplane in the game.
    #[cfg(feature = "ui_prints")]
    pub fn list_airplane(&self, plane_id: usize) -> Result<(), GameError> {
        if plane_id > (self.airplanes.len() - 1) {
            return Err(GameError::PlaneIdInvalid { id: plane_id });
        }

        let plane = &self.airplanes[plane_id];

        if let AirplaneStatus::InTransit {
            hours_remaining,
            destination,
            ..
        } = plane.status
        {
            let dest_name = &self.map.airports[destination].0.name;
            println!(
                "ID: {} | {:?} en-route to airport {} | Location: ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: InTransit - arrival in {}",
                plane.id,
                plane.model,
                dest_name,
                plane.location.x,
                plane.location.y,
                plane.current_fuel,
                plane.specs.fuel_capacity,
                plane.current_payload,
                plane.specs.payload_capacity,
                self.days_and_hours(hours_remaining)
            );

            Ok(())
        } else {
            let loc = &plane.location;
            let airport_name = self.find_associated_airport(loc)?;
            println!(
                "ID: {} | {:?} at airport {} ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Cargo: {:.2}/{:.2}kg | Pax: {}/{} | Status: {:?}",
                plane.id,
                plane.model,
                airport_name,
                loc.x,
                loc.y,
                plane.current_fuel,
                plane.specs.fuel_capacity,
                plane.current_payload,
                plane.specs.payload_capacity,
                plane.current_passengers,
                plane.specs.passenger_capacity,
                plane.status,
            );
            if !plane.manifest.is_empty() {
                println!("  Manifest:");
                for order in plane.manifest.clone() {
                    let payload_info = match &order.payload {
                        OrderPayload::Cargo { cargo_type, weight } => {
                            format!("{:?} | weight: {:.1}kg", cargo_type, weight)
                        }
                        OrderPayload::Passengers { count } => {
                            format!("Passengers | count: {}", count)
                        }
                    };
                    println!(
                        "    [{}] {} -> {} | value: ${:.2} | deadline: {} | destination: {}",
                        order.id,
                        payload_info,
                        self.map.airports[order.destination_id].0.name,
                        order.value,
                        order.deadline,
                        order.destination_id
                    );
                }
            }

            Ok(())
        }
    }

    #[cfg(feature = "ui_prints")]
    pub fn show_distances(&self, plane_id: usize) -> Result<(), GameError> {
        if plane_id > (self.airplanes.len() - 1) {
            return Err(GameError::PlaneIdInvalid { id: plane_id });
        }

        let plane = &self.airplanes[plane_id];

        // If plane is in transit, dont't calc
        if let AirplaneStatus::InTransit { .. } = plane.status {
            println!("Plane currently in transit");
            Ok(())
        } else {
            for (airport, coordinate) in &self.map.airports {
                let distance = plane.distance_to(coordinate);

                let can_land = plane.can_fly_to(airport, coordinate).is_ok();

                println!(
                    "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Distance to: {:.2}km | Can land: {:?}",
                    airport.id,
                    airport.name,
                    coordinate.x,
                    coordinate.y,
                    airport.runway_length,
                    distance,
                    can_land
                );
            }
            Ok(())
        }
    }

    /// Buy an airplane is possible
    pub fn buy_plane(&mut self, model: &String, airport_id: usize) -> Result<(), GameError> {
        // Get copy of home coordinate
        let home_coord = {
            let (_airport, coord) = &self.map.airports[airport_id];
            *coord
        };

        // Borrow airport as mut
        let airport_ref = &mut self.map.airports[airport_id].0;

        // Try catalog (case-insensitive)
        if let Some((name, specs)) = self
            .model_catalog
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(model))
            .map(|(n, s)| (n.clone(), *s))
        {
            match self
                .player
                .buy_plane_with_specs(&name, airport_ref, &home_coord, specs)
            {
                Ok(_) => {
                    let (new_plane_id, buying_price) = {
                        let plane = self
                            .player
                            .fleet
                            .last()
                            .expect("player fleet must contain newly purchased plane");
                        (plane.id, plane.specs.purchase_price)
                    };
                    self.daily_expenses += buying_price;

                    self.airplanes = self.player.fleet.clone();
                    self.player.fleet = self.airplanes.clone();
                    self.player.fleet_size = self.player.fleet.len();
                    self.arrival_times.insert(new_plane_id, self.time);
                    return Ok(());
                }
                Err(e) => return Err(e),
            }
        }

        if self.models_replace {
            return Err(GameError::UnknownModel {
                input: model.clone(),
                suggestion: None,
            });
        }

        // Fallback to built-in models
        match self.player.buy_plane(model, airport_ref, &home_coord) {
            Ok(_) => {
                let (new_plane_id, buying_price) = {
                    let plane = self
                        .player
                        .fleet
                        .last()
                        .expect("player fleet must contain newly purchased plane");
                    (plane.id, plane.specs.purchase_price)
                };
                self.daily_expenses += buying_price;

                self.airplanes = self.player.fleet.clone();
                self.player.fleet = self.airplanes.clone();
                self.player.fleet_size = self.player.fleet.len();
                self.arrival_times.insert(new_plane_id, self.time);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Sell an airplane currently owned by the player.
    pub fn sell_plane(&mut self, plane_id: usize) -> Result<f32, GameError> {
        let plane_index = self
            .airplanes
            .iter()
            .position(|plane| plane.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let plane_snapshot = self.airplanes[plane_index].clone();
        if !plane_snapshot.manifest.is_empty() {
            return Err(GameError::InvalidCommand {
                msg: format!(
                    "Plane {} cannot be sold while carrying {} orders",
                    plane_id,
                    plane_snapshot.manifest.len()
                ),
            });
        }
        if plane_snapshot.status != AirplaneStatus::Parked {
            return Err(GameError::PlaneNotReady {
                plane_state: plane_snapshot.status,
            });
        }

        let (sold_plane, refund) = self.player.sell_plane(plane_id)?;
        debug_assert_eq!(sold_plane.id, plane_id);

        self.airplanes.remove(plane_index);
        self.arrival_times.remove(&plane_id);

        self.player.fleet = self.airplanes.clone();
        self.player.fleet_size = self.player.fleet.len();

        self.daily_income += refund;

        Ok(refund)
    }

    /// Load an order onto a plane if capacity and state allow it.
    ///
    /// Parameters
    /// - `order_id`: Order at the current airport to load.
    /// - `plane_id`: Plane ID (must be parked at the airport).
    ///
    /// Returns
    /// - `Ok(())` on success.
    /// - `Err(GameError)`: If the plane doesn't exist, isn't parked, or capacity constraints fail.
    pub fn load_order(&mut self, order_id: usize, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;
        let plane = &mut self.airplanes[plane_idx];
        let airport = &mut self.map.airports[airport_idx].0;

        airport.load_order(order_id, plane)?;
        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload all orders from the plane.
    ///
    /// Parameters
    /// - `plane_id`: Plane ID.
    ///
    /// Returns
    /// - `Ok(())` on success.
    /// - `Err(GameError)`: If the plane doesn't exist or isn't parked at an airport.
    pub fn unload_all(&mut self, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;

        let airport = &mut self.map.airports[airport_idx].0;
        let plane = &mut self.airplanes[plane_idx];
        let mut deliveries = plane.unload_all();

        // Check deliveries
        for delivery in deliveries.drain(..) {
            // reached the destination and before deadline
            if delivery.destination_id == airport.id {
                if delivery.deadline != 0 {
                    println!("Successfully delivered order {}", delivery.id);
                    self.player.cash += delivery.value;
                    self.daily_income += delivery.value;
                    self.player.record_delivery();
                } else {
                    println!("Order {}: Deadline expired", delivery.id)
                }
            }
            // not the destination so it goes into the stock at the airport
            else {
                println!(
                    "Order {} being stored at airport {}",
                    delivery.id, airport.id
                );
                airport.orders.push(delivery);
            }
        }

        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload a list of orders from a plane.
    ///
    /// Parameters
    /// - `order_id`: A list of order IDs to unload.
    /// - `plane_id`: Plane ID.
    ///
    /// Returns
    /// - `Ok(())` on success.
    /// - `Err(GameError)`: If the plane doesn't exist or isn't parked at an airport.
    pub fn unload_orders(
        &mut self,
        order_id: Vec<usize>,
        plane_id: usize,
    ) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;

        let airport = &mut self.map.airports[airport_idx].0;
        let plane = &mut self.airplanes[plane_idx];

        for order in order_id {
            let delivery = plane.unload_order(order)?;

            if delivery.destination_id == airport.id {
                if delivery.deadline != 0 {
                    println!("Successfully delivered order {}", delivery.id);
                    self.player.cash += delivery.value;
                    self.daily_income += delivery.value;
                    self.player.record_delivery();
                } else {
                    println!("Order {}: Deadline expired", delivery.id)
                }
            }
            // not the destination so it goes into the stock at the airport
            else {
                println!(
                    "Order {} being stored at airport {}",
                    delivery.id, airport.id
                );
                airport.orders.push(delivery);
            }
        }
        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload a specific order from a plane.
    ///
    /// Parameters
    /// - `order_id`: Order ID to unload.
    /// - `plane_id`: Plane ID.
    ///
    /// Returns
    /// - `Ok(())` on success.
    /// - `Err(GameError)`: If the plane doesn't exist or isn't parked.
    pub fn unload_order(&mut self, order_id: usize, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;

        let airport = &mut self.map.airports[airport_idx].0;
        let plane = &mut self.airplanes[plane_idx];

        let delivery = plane.unload_order(order_id)?;

        if delivery.destination_id == airport.id {
            if delivery.deadline != 0 {
                println!("Successfully delivered order {}", delivery.id);
                self.player.cash += delivery.value;
                self.daily_income += delivery.value;
                self.player.record_delivery();
            } else {
                println!("Order {}: Deadline expired", delivery.id)
            }
        }
        // not the destination so it goes into the stock at the airport
        else {
            println!(
                "Order {} being stored at airport {}",
                delivery.id, airport.id
            );
            airport.orders.push(delivery);
        }

        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Depart a plane to another airport.
    ///
    /// Parameters
    /// - `plane_id`: Plane ID to dispatch.
    /// - `destination_id`: Destination airport ID.
    ///
    /// Returns
    /// - `Ok(())` on success.
    /// - `Err(GameError)`: Invalid IDs, not parked, insufficient fuel, or runway issues.
    pub fn depart_plane(
        &mut self,
        plane_id: usize,
        destination_id: usize,
    ) -> Result<(), GameError> {
        let (plane_idx, origin_idx) = self.plane_and_airport_idx(plane_id)?;
        let plane = &mut self.airplanes[plane_idx];

        // Guard rail: only depart when parked
        if !matches!(plane.status, AirplaneStatus::Parked) {
            return Err(GameError::PlaneNotReady {
                plane_state: plane.status.clone(),
            });
        }
        let (dest_airport, dest_coords) = &self
            .map
            .airports
            .iter()
            .find(|(a, _)| a.id == destination_id)
            .ok_or(GameError::AirportIdInvalid { id: destination_id })?;

        // consume fuel & get flight_hours
        // check before if we can get there, else we don't charge
        let flight_hours = plane.consume_flight_fuel(dest_airport, dest_coords)?;
        let origin_coord = plane.location;

        // charge parking
        let parked_since = *self.arrival_times.get(&plane_id).unwrap_or(&self.time);
        let parked_hours = (self.time - parked_since) as f32;
        let parking_fee = self.map.airports[origin_idx].0.parking_fee * parked_hours;
        self.player.cash -= parking_fee;
        self.daily_expenses += parking_fee;

        // set the status (no location change here!)
        plane.status = AirplaneStatus::InTransit {
            hours_remaining: flight_hours,
            destination: destination_id,
            origin: origin_coord,
            total_hours: flight_hours,
        };

        // kick off the first hourly tick
        self.schedule(self.time + 1, Event::FlightProgress { plane: plane_id });

        Ok(())
    }

    /// Refuel a plane and charge the player. Only works if the airplane is not in transit.
    ///
    /// Parameters
    /// - `plane_id`: Plane to refuel.
    ///
    /// Returns
    /// - `Ok(()))` on success.
    /// - `Err(GameError)`: If plane is invalid, not parked, or funds are insufficient.
    pub fn refuel_plane(&mut self, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;
        let plane = &mut self.airplanes[plane_idx];

        // fuel airplane and log liters for dynamic pricing
        let fueling_fee = self.map.airports[airport_idx].0.fueling_fee(plane);
        if self.player.cash < fueling_fee {
            return Err(GameError::InsufficientFunds {
                have: self.player.cash,
                need: fueling_fee,
            });
        }
        self.map.airports[airport_idx].0.fuel_supply(plane);
        plane.refuel();

        // charge the player
        self.player.cash -= fueling_fee;
        self.daily_expenses += fueling_fee;

        // schedule fueling event
        self.schedule(self.time + 1, Event::RefuelComplete { plane: plane_id });

        Ok(())
    }

    /// Perform maintenance on airplane
    pub fn maintenance_on_airplane(&mut self, plane_id: usize) -> Result<(), GameError> {
        let airplane = &mut self.airplanes[plane_id];

        // cannot perform maintenance when not at an airport
        if matches!(
            airplane.status,
            AirplaneStatus::InTransit {
                hours_remaining: _,
                destination: _,
                origin: _,
                total_hours: _
            }
        ) {
            return Err(GameError::PlaneNotAtAirport { plane_id });
        }

        airplane.maintenance();
        self.schedule(self.time + 1, Event::Maintenance { plane: plane_id });
        Ok(())
    }

    pub fn execute_str(&mut self, line: &str) -> Result<(), GameError> {
        let cmd =
            parse_command(line).map_err(|e| GameError::InvalidCommand { msg: e.to_string() })?;
        self.execute(cmd)
    }

    pub fn execute(&mut self, cmd: Command) -> Result<(), GameError> {
        match cmd {
            ShowAirports { .. }
            | ShowAirport { .. }
            | ShowAirplanes
            | ShowAirplane { .. }
            | ShowDistances { .. }
            | ShowCash
            | ShowTime
            | ShowStats
            | ShowModels
            | LoadConfig { .. }
            | Exit => Ok(()),
            BuyPlane { model, airport } => self.buy_plane(&model, airport),
            SellPlane { plane } => {
                self.sell_plane(plane)?;
                Ok(())
            }
            LoadOrder { order, plane } => self.load_order(order, plane),
            LoadOrders { orders, plane } => {
                for o in orders {
                    self.load_order(o, plane)?;
                }
                Ok(())
            }
            UnloadOrder { order, plane } => self.unload_order(order, plane),
            UnloadOrders { orders, plane } => {
                for o in orders {
                    self.unload_order(o, plane)?;
                }
                Ok(())
            }
            UnloadAll { plane } => self.unload_all(plane),
            Refuel { plane } => self.refuel_plane(plane),
            DepartPlane { plane, dest } => self.depart_plane(plane, dest),
            HoldPlane { .. } => Ok(()),
            Advance { hours } => {
                self.advance(hours);
                Ok(())
            }
            SaveGame { name } => self
                .save_game(&name)
                .map_err(|e| GameError::InvalidCommand { msg: e.to_string() }),
            LoadGame { name } => {
                *self = Game::load_game(&name)
                    .map_err(|e| GameError::InvalidCommand { msg: e.to_string() })?;
                Ok(())
            }
            Maintenance { plane_id } => self.maintenance_on_airplane(plane_id),
        }
    }

    pub fn observe(&self) -> Observation {
        let airports = self
            .map
            .airports
            .iter()
            .map(|(airport, coord)| AirportObs {
                id: airport.id,
                name: airport.name.clone(),
                x: coord.x,
                y: coord.y,
                fuel_price: airport.fuel_price,
                runway_length: airport.runway_length,
                num_orders: airport.orders.len(),
            })
            .collect();

        let planes = self
            .airplanes
            .iter()
            .map(|plane| {
                let (destination, hours_remaining) = match plane.status {
                    AirplaneStatus::InTransit {
                        destination,
                        hours_remaining,
                        ..
                    } => (Some(destination), Some(hours_remaining)),
                    _ => (None, None),
                };
                PlaneObs {
                    id: plane.id,
                    model: format!("{:?}", plane.model),
                    x: plane.location.x,
                    y: plane.location.y,
                    status: format!("{:?}", plane.status),
                    fuel: FuelObs {
                        current: plane.current_fuel,
                        capacity: plane.specs.fuel_capacity,
                    },
                    payload: PayloadObs {
                        cargo_current: plane.current_payload,
                        cargo_capacity: plane.specs.payload_capacity,
                        passenger_current: plane.current_passengers,
                        passenger_capacity: plane.specs.passenger_capacity,
                    },
                    destination,
                    hours_remaining,
                }
            })
            .collect();

        Observation {
            time: self.time,
            cash: self.player.cash,
            airports,
            planes,
        }
    }

    // ************************
    // ******* GUI APIs *******
    // ************************

    pub fn get_cash(&self) -> f32 {
        self.player.cash
    }

    pub fn get_time(&self) -> String {
        self.days_and_hours(self.time)
    }

    pub fn airports(&self) -> &[(Airport, Coordinate)] {
        &self.map.airports
    }

    pub fn planes(&self) -> &Vec<Airplane> {
        &self.airplanes
    }

    /// Return the available airplane models for purchases in this game.
    /// Includes custom models loaded from YAML according to replace/add strategy.
    pub fn available_models(&self) -> Vec<(String, AirplaneSpecs)> {
        let mut v: Vec<(String, AirplaneSpecs)> = self
            .model_catalog
            .iter()
            .map(|(name, specs)| (name.clone(), *specs))
            .collect();
        v.sort_by(|a, b| a.1.purchase_price.partial_cmp(&b.1.purchase_price).unwrap());
        v
    }
}

fn model_specs_from_config(m: &AirplaneModelConfig) -> AirplaneSpecs {
    AirplaneSpecs {
        mtow: m.mtow,
        cruise_speed: m.cruise_speed,
        fuel_capacity: m.fuel_capacity,
        fuel_consumption: m.fuel_consumption,
        operating_cost: m.operating_cost,
        payload_capacity: m.payload_capacity,
        passenger_capacity: m.passenger_capacity,
        purchase_price: m.purchase_price,
        min_runway_length: m.min_runway_length,
        role: m.role,
    }
}

fn validate_model_config(m: &AirplaneModelConfig) -> Result<(), GameError> {
    if m.name.trim().is_empty() {
        return Err(GameError::InvalidConfig {
            msg: "airplane model name cannot be empty".into(),
        });
    }
    if m.mtow <= 0.0
        || m.cruise_speed <= 0.0
        || m.fuel_capacity <= 0.0
        || m.fuel_consumption <= 0.0
        || m.purchase_price <= 0.0
        || m.min_runway_length <= 0.0
    {
        return Err(GameError::InvalidConfig {
            msg: format!(
                "airplane '{}' has non-positive required numeric fields",
                m.name
            ),
        });
    }
    if m.operating_cost < 0.0 || m.payload_capacity < 0.0 {
        return Err(GameError::InvalidConfig {
            msg: format!("airplane '{}' has negative cost or payload", m.name),
        });
    }
    match m.role {
        crate::utils::airplanes::models::AirplaneRole::Cargo => {
            if m.payload_capacity <= 0.0 {
                return Err(GameError::InvalidConfig {
                    msg: format!(
                        "airplane '{}' role cargo requires payload_capacity > 0",
                        m.name
                    ),
                });
            }
        }
        crate::utils::airplanes::models::AirplaneRole::Passenger => {
            if m.passenger_capacity == 0 {
                return Err(GameError::InvalidConfig {
                    msg: format!(
                        "airplane '{}' role passenger requires passenger_capacity > 0",
                        m.name
                    ),
                });
            }
        }
        crate::utils::airplanes::models::AirplaneRole::Mixed => {
            if m.payload_capacity <= 0.0 || m.passenger_capacity == 0 {
                return Err(GameError::InvalidConfig {
                    msg: format!(
                        "airplane '{}' role mixed requires both passenger_capacity and payload_capacity",
                        m.name
                    ),
                });
            }
        }
    }
    Ok(())
}
