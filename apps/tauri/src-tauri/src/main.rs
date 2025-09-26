#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;

use rusty_runways_core::game::Observation;
use rusty_runways_core::statistics::DailyStats;
use rusty_runways_core::utils::airplanes::models::AirplaneModel;
use rusty_runways_core::Game;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use strum::IntoEnumIterator;
use tauri::State;

#[derive(Default)]
struct AppState {
    game: Mutex<Option<Game>>,
}

fn default_starting_cash() -> f32 {
    650_000.0
}

#[derive(Deserialize)]
struct NewGameArgs {
    #[serde(default)]
    seed: Option<u64>,
    #[serde(rename = "numAirports", alias = "num_airports")]
    num_airports: Option<usize>,
    #[serde(
        rename = "startingCash",
        alias = "starting_cash",
        default = "default_starting_cash"
    )]
    starting_cash: f32,
}

#[tauri::command]
fn new_game(state: State<AppState>, args: NewGameArgs) -> Result<(), String> {
    let seed = args.seed.unwrap_or(0);
    let game = Game::new(seed, args.num_airports, args.starting_cash);
    // schedule initial events as in Game::new already does
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    *guard = Some(game);
    Ok(())
}

#[tauri::command]
fn load_game_cmd(state: State<AppState>, name: String) -> Result<(), String> {
    let game = Game::load_game(&name).map_err(|e| e.to_string())?;
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    *guard = Some(game);
    Ok(())
}

#[tauri::command]
fn save_game_cmd(state: State<AppState>, name: String) -> Result<(), String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;
    game.save_game(&name).map_err(|e| e.to_string())
}

#[tauri::command]
fn observe(state: State<AppState>) -> Result<Observation, String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;
    Ok(game.observe())
}

#[tauri::command]
fn advance(state: State<AppState>, hours: u64) -> Result<Observation, String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.advance(hours);
    Ok(game.observe())
}

#[tauri::command]
fn stats_cmd(state: State<AppState>) -> Result<Vec<DailyStats>, String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;
    Ok(game.stats.clone())
}

#[tauri::command]
fn depart_plane(state: State<AppState>, plane: usize, dest: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.depart_plane(plane, dest).map_err(|e| e.to_string())
}

#[tauri::command]
fn load_order(state: State<AppState>, order: usize, plane: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.load_order(order, plane).map_err(|e| e.to_string())
}

#[tauri::command]
fn unload_order(state: State<AppState>, order: usize, plane: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.unload_order(order, plane).map_err(|e| e.to_string())
}

#[tauri::command]
fn unload_orders(state: State<AppState>, orders: Vec<usize>, plane: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.unload_orders(orders, plane).map_err(|e| e.to_string())
}

#[tauri::command]
fn unload_all(state: State<AppState>, plane: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.unload_all(plane).map_err(|e| e.to_string())
}

#[tauri::command]
fn refuel_plane(state: State<AppState>, plane: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.refuel_plane(plane).map_err(|e| e.to_string())
}

#[tauri::command]
fn maintenance(state: State<AppState>, plane: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.maintenance_on_airplane(plane)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn sell_plane_cmd(state: State<AppState>, plane: usize) -> Result<f32, String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.sell_plane(plane).map_err(|e| e.to_string())
}

#[derive(Serialize)]
struct OrderDto {
    id: usize,
    destination_id: usize,
    value: f32,
    deadline: u64,
    payload_kind: String,
    cargo_type: Option<String>,
    weight: Option<f32>,
    passenger_count: Option<u32>,
}

#[derive(Serialize)]
struct PlaneInfoDto {
    id: usize,
    model: String,
    status: String,
    x: f32,
    y: f32,
    fuel_current: f32,
    fuel_capacity: f32,
    payload_current: f32,
    payload_capacity: f32,
    passenger_current: u32,
    passenger_capacity: u32,
    current_airport_id: Option<usize>,
    manifest: Vec<OrderDto>,
}

#[tauri::command]
fn plane_info(state: State<AppState>, plane_id: usize) -> Result<PlaneInfoDto, String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;

    let plane = game
        .planes()
        .iter()
        .find(|p| p.id == plane_id)
        .ok_or_else(|| "plane not found".to_string())?;

    // find current airport id, if any
    let current_airport_id = game
        .airports()
        .iter()
        .position(|(_, coord)| *coord == plane.location);

    let manifest = plane
        .manifest
        .iter()
        .map(|o| OrderDto {
            id: o.id,
            destination_id: o.destination_id,
            value: o.value,
            deadline: o.deadline,
            payload_kind: o.payload.kind_label().to_string(),
            cargo_type: o.cargo_type().map(|c| format!("{:?}", c)),
            weight: o.cargo_weight(),
            passenger_count: o.passenger_count(),
        })
        .collect();

    Ok(PlaneInfoDto {
        id: plane.id,
        model: format!("{:?}", plane.model),
        status: format!("{:?}", plane.status),
        x: plane.location.x,
        y: plane.location.y,
        fuel_current: plane.current_fuel,
        fuel_capacity: plane.specs.fuel_capacity,
        payload_current: plane.current_payload,
        payload_capacity: plane.specs.payload_capacity,
        passenger_current: plane.current_passengers,
        passenger_capacity: plane.specs.passenger_capacity,
        current_airport_id,
        manifest,
    })
}

#[tauri::command]
fn airport_orders(state: State<AppState>, airport_id: usize) -> Result<Vec<OrderDto>, String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;
    let (airport, _) = game
        .airports()
        .iter()
        .find(|(a, _)| a.id == airport_id)
        .ok_or_else(|| "airport not found".to_string())?;
    let orders = airport
        .orders
        .iter()
        .map(|o| OrderDto {
            id: o.id,
            destination_id: o.destination_id,
            value: o.value,
            deadline: o.deadline,
            payload_kind: o.payload.kind_label().to_string(),
            cargo_type: o.cargo_type().map(|c| format!("{:?}", c)),
            weight: o.cargo_weight(),
            passenger_count: o.passenger_count(),
        })
        .collect();
    Ok(orders)
}

#[derive(Serialize)]
struct ModelDto {
    name: String,
    mtow: f32,
    cruise_speed: f32,
    fuel_capacity: f32,
    fuel_consumption: f32,
    operating_cost: f32,
    payload_capacity: f32,
    passenger_capacity: u32,
    purchase_price: f32,
    min_runway_length: f32,
    role: String,
}

#[tauri::command]
fn list_models() -> Vec<ModelDto> {
    AirplaneModel::iter()
        .map(|m| {
            let s = m.specs();
            ModelDto {
                name: format!("{:?}", m),
                mtow: s.mtow,
                cruise_speed: s.cruise_speed,
                fuel_capacity: s.fuel_capacity,
                fuel_consumption: s.fuel_consumption,
                operating_cost: s.operating_cost,
                payload_capacity: s.payload_capacity,
                passenger_capacity: s.passenger_capacity,
                purchase_price: s.purchase_price,
                min_runway_length: s.min_runway_length,
                role: format!("{:?}", s.role),
            }
        })
        .collect()
}

#[tauri::command]
fn buy_plane_cmd(state: State<AppState>, model: String, airport_id: usize) -> Result<(), String> {
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_mut().ok_or("no game running")?;
    game.buy_plane(&model, airport_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn plane_can_fly_to(
    state: State<AppState>,
    plane_id: usize,
    dest_id: usize,
) -> Result<bool, String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;
    let plane = game
        .planes()
        .iter()
        .find(|p| p.id == plane_id)
        .ok_or("plane not found")?;
    let (airport, coord) = game
        .airports()
        .iter()
        .find(|(a, _)| a.id == dest_id)
        .ok_or("airport not found")?;
    Ok(plane.can_fly_to(airport, coord).is_ok())
}

#[derive(Serialize)]
struct FeasibilityDto {
    ok: bool,
    reason: Option<String>,
}

#[tauri::command]
fn plane_reachability(
    state: State<AppState>,
    plane_id: usize,
    dest_id: usize,
) -> Result<FeasibilityDto, String> {
    let guard = state.game.lock().map_err(|_| "state poisoned")?;
    let game = guard.as_ref().ok_or("no game running")?;
    let plane = game
        .planes()
        .iter()
        .find(|p| p.id == plane_id)
        .ok_or("plane not found")?;
    let (airport, coord) = game
        .airports()
        .iter()
        .find(|(a, _)| a.id == dest_id)
        .ok_or("airport not found")?;
    match plane.can_fly_to(airport, coord) {
        Ok(_) => Ok(FeasibilityDto {
            ok: true,
            reason: None,
        }),
        Err(e) => Ok(FeasibilityDto {
            ok: false,
            reason: Some(e.to_string()),
        }),
    }
}

#[tauri::command]
fn start_from_config_yaml(state: State<AppState>, yaml: String) -> Result<(), String> {
    let cfg: rusty_runways_core::config::WorldConfig =
        serde_yaml::from_str(&yaml).map_err(|e| e.to_string())?;
    let game = rusty_runways_core::Game::from_config(cfg).map_err(|e| e.to_string())?;
    let mut guard = state.game.lock().map_err(|_| "state poisoned")?;
    *guard = Some(game);
    Ok(())
}

#[tauri::command]
fn start_from_config_path(state: State<AppState>, path: String) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    start_from_config_yaml(state, text)
}

#[tauri::command]
fn list_saves() -> Result<Vec<String>, String> {
    let dir = Path::new("save_games");
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut names = vec![];
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            new_game,
            load_game_cmd,
            save_game_cmd,
            observe,
            advance,
            depart_plane,
            load_order,
            unload_order,
            unload_orders,
            unload_all,
            refuel_plane,
            maintenance,
            sell_plane_cmd,
            plane_info,
            airport_orders,
            list_models,
            buy_plane_cmd,
            plane_can_fly_to,
            plane_reachability,
            start_from_config_yaml,
            start_from_config_path,
            list_saves,
            stats_cmd,
        ])
        .setup(|_app| Ok(()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
