use once_cell::sync::OnceCell;
use rusty_runways_core::Game;
use strum::IntoEnumIterator;
use wasm_bindgen::prelude::*;

static GAME: OnceCell<std::sync::Mutex<Game>> = OnceCell::new();

fn with_game<F, T>(f: F) -> Result<T, JsValue>
where
    F: FnOnce(&mut Game) -> Result<T, String>,
{
    let m = GAME
        .get()
        .ok_or_else(|| JsValue::from_str("game not initialized"))?;
    let mut g = m.lock().map_err(|_| JsValue::from_str("mutex poisoned"))?;
    f(&mut g).map_err(|e| JsValue::from_str(&e))
}

#[wasm_bindgen]
pub fn new_game(seed: Option<u64>, num_airports: Option<usize>, starting_cash: f32) {
    let game = Game::new(seed.unwrap_or(0), num_airports, starting_cash);
    let _ = GAME.set(std::sync::Mutex::new(game));
}

#[wasm_bindgen]
pub fn observe() -> Result<JsValue, JsValue> {
    with_game(|g| Ok(serde_wasm_bindgen::to_value(&g.observe()).unwrap()))
}

#[wasm_bindgen]
pub fn advance(hours: u64) -> Result<JsValue, JsValue> {
    with_game(|g| {
        g.advance(hours);
        Ok(serde_wasm_bindgen::to_value(&g.observe()).unwrap())
    })
}

#[wasm_bindgen]
pub fn plane_info(plane_id: usize) -> Result<JsValue, JsValue> {
    with_game(|g| {
        let plane = g
            .planes()
            .iter()
            .find(|p| p.id == plane_id)
            .ok_or_else(|| "plane not found".to_string())?;
        // find current airport id, if any
        let current_airport_id = g
            .airports()
            .iter()
            .position(|(_, coord)| *coord == plane.location);
        #[derive(serde::Serialize)]
        struct OrderDto {
            id: usize,
            destination_id: usize,
            value: f32,
            weight: f32,
            deadline: u64,
            cargo_type: String,
        }
        #[derive(serde::Serialize)]
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
            current_airport_id: Option<usize>,
            manifest: Vec<OrderDto>,
        }
        let manifest = plane
            .manifest
            .iter()
            .map(|o| OrderDto {
                id: o.id,
                destination_id: o.destination_id,
                value: o.value,
                weight: o.weight,
                deadline: o.deadline,
                cargo_type: format!("{:?}", o.name),
            })
            .collect();
        let dto = PlaneInfoDto {
            id: plane.id,
            model: format!("{:?}", plane.model),
            status: format!("{:?}", plane.status),
            x: plane.location.x,
            y: plane.location.y,
            fuel_current: plane.current_fuel,
            fuel_capacity: plane.specs.fuel_capacity,
            payload_current: plane.current_payload,
            payload_capacity: plane.specs.payload_capacity,
            current_airport_id,
            manifest,
        };
        Ok(serde_wasm_bindgen::to_value(&dto).unwrap())
    })
}

#[wasm_bindgen]
pub fn airport_orders(airport_id: usize) -> Result<JsValue, JsValue> {
    with_game(|g| {
        let (airport, _) = g
            .airports()
            .iter()
            .find(|(a, _)| a.id == airport_id)
            .ok_or_else(|| "airport not found".to_string())?;
        #[derive(serde::Serialize)]
        struct OrderDto {
            id: usize,
            destination_id: usize,
            value: f32,
            weight: f32,
            deadline: u64,
            cargo_type: String,
        }
        let orders: Vec<OrderDto> = airport
            .orders
            .iter()
            .map(|o| OrderDto {
                id: o.id,
                destination_id: o.destination_id,
                value: o.value,
                weight: o.weight,
                deadline: o.deadline,
                cargo_type: format!("{:?}", o.name),
            })
            .collect();
        Ok(serde_wasm_bindgen::to_value(&orders).unwrap())
    })
}

#[wasm_bindgen]
pub fn depart_plane(plane: usize, dest: usize) -> Result<(), JsValue> {
    with_game(|g| {
        g.depart_plane(plane, dest)
            .map_err(|e| e.to_string())
            .map(|_| ())
    })
}

#[wasm_bindgen]
pub fn refuel_plane(plane: usize) -> Result<(), JsValue> {
    with_game(|g| g.refuel_plane(plane).map_err(|e| e.to_string()).map(|_| ()))
}

#[wasm_bindgen]
pub fn maintenance(plane: usize) -> Result<(), JsValue> {
    with_game(|g| {
        g.maintenance_on_airplane(plane)
            .map_err(|e| e.to_string())
            .map(|_| ())
    })
}

#[wasm_bindgen]
pub fn load_order(order: usize, plane: usize) -> Result<(), JsValue> {
    with_game(|g| {
        g.load_order(order, plane)
            .map_err(|e| e.to_string())
            .map(|_| ())
    })
}

#[wasm_bindgen]
pub fn unload_order(order: usize, plane: usize) -> Result<(), JsValue> {
    with_game(|g| {
        g.unload_order(order, plane)
            .map_err(|e| e.to_string())
            .map(|_| ())
    })
}

#[wasm_bindgen]
pub fn unload_all(plane: usize) -> Result<(), JsValue> {
    with_game(|g| g.unload_all(plane).map_err(|e| e.to_string()).map(|_| ()))
}

#[wasm_bindgen]
pub fn list_models() -> Result<JsValue, JsValue> {
    use rusty_runways_core::utils::airplanes::models::AirplaneModel;
    #[derive(serde::Serialize)]
    struct ModelDto {
        name: String,
        mtow: f32,
        cruise_speed: f32,
        fuel_capacity: f32,
        fuel_consumption: f32,
        operating_cost: f32,
        payload_capacity: f32,
        purchase_price: f32,
        min_runway_length: f32,
    }
    let models: Vec<ModelDto> = AirplaneModel::iter()
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
                purchase_price: s.purchase_price,
                min_runway_length: s.min_runway_length,
            }
        })
        .collect();
    Ok(serde_wasm_bindgen::to_value(&models).unwrap())
}

#[wasm_bindgen]
pub fn buy_plane(model: String, airport_id: usize) -> Result<(), JsValue> {
    with_game(|g| {
        g.buy_plane(&model, airport_id)
            .map_err(|e| e.to_string())
            .map(|_| ())
    })
}

#[wasm_bindgen]
pub fn plane_can_fly_to(plane_id: usize, dest_id: usize) -> Result<bool, JsValue> {
    with_game(|g| {
        let plane = g
            .planes()
            .iter()
            .find(|p| p.id == plane_id)
            .ok_or_else(|| "plane not found".to_string())?;
        let (airport, coord) = g
            .airports()
            .iter()
            .find(|(a, _)| a.id == dest_id)
            .ok_or_else(|| "airport not found".to_string())?;
        Ok(plane.can_fly_to(airport, coord).is_ok())
    })
}

#[wasm_bindgen]
pub fn plane_reachability(plane_id: usize, dest_id: usize) -> Result<JsValue, JsValue> {
    #[derive(serde::Serialize)]
    struct FeasibilityDto {
        ok: bool,
        reason: Option<String>,
    }
    with_game(|g| {
        let plane = g
            .planes()
            .iter()
            .find(|p| p.id == plane_id)
            .ok_or_else(|| "plane not found".to_string())?;
        let (airport, coord) = g
            .airports()
            .iter()
            .find(|(a, _)| a.id == dest_id)
            .ok_or_else(|| "airport not found".to_string())?;
        let dto = match plane.can_fly_to(airport, coord) {
            Ok(_) => FeasibilityDto {
                ok: true,
                reason: None,
            },
            Err(e) => FeasibilityDto {
                ok: false,
                reason: Some(e.to_string()),
            },
        };
        Ok(serde_wasm_bindgen::to_value(&dto).unwrap())
    })
}
