import { invoke } from '@tauri-apps/api/core'

export type Observation = {
  time: number
  cash: number
  airports: { id: number; name: string; x: number; y: number; fuel_price: number; runway_length: number; num_orders: number }[]
  planes: { id: number; model: string; x: number; y: number; status: string; destination?: number | null; hours_remaining?: number | null; fuel: { current: number; capacity: number }; payload: { current: number; capacity: number } }[]
}

export async function newGame(seed: string | undefined, airportCount: number, startingCash: number): Promise<void> {
  const parsedSeed = seed && seed.trim() !== '' ? Number(seed) : undefined
  // Tauri v2 command expects a single `args` object (we also support snake_case via aliases on the Rust side)
  await invoke('new_game', {
    args: {
      seed: parsedSeed,
      numAirports: airportCount,
      startingCash: startingCash,
    },
  })
}

export async function loadGame(name: string): Promise<void> {
  await invoke('load_game_cmd', { name })
}

export async function saveGame(name: string): Promise<void> {
  await invoke('save_game_cmd', { name })
}

export async function observe(): Promise<Observation> {
  return await invoke<Observation>('observe')
}

export async function advance(hours = 1): Promise<Observation> {
  return await invoke<Observation>('advance', { hours })
}

export async function departPlane(plane: number, dest: number): Promise<void> {
  await invoke('depart_plane', { plane, dest })
}

export async function refuelPlane(plane: number): Promise<void> {
  await invoke('refuel_plane', { plane })
}

export async function maintenance(plane: number): Promise<void> {
  await invoke('maintenance', { plane })
}

export async function loadOrder(order: number, plane: number): Promise<void> {
  await invoke('load_order', { order, plane })
}

export async function unloadOrder(order: number, plane: number): Promise<void> {
  await invoke('unload_order', { order, plane })
}

export async function unloadAll(plane: number): Promise<void> {
  await invoke('unload_all', { plane })
}

export type PlaneInfo = {
  id: number
  model: string
  status: string
  x: number
  y: number
  fuel_current: number
  fuel_capacity: number
  payload_current: number
  payload_capacity: number
  current_airport_id: number | null
  manifest: OrderDto[]
}

export type OrderDto = {
  id: number
  destination_id: number
  value: number
  weight: number
  deadline: number
  cargo_type: string
}

export async function planeInfo(planeId: number): Promise<PlaneInfo> {
  // Send both snake_case and camelCase to be robust with Tauri arg naming
  return await invoke<PlaneInfo>('plane_info', { plane_id: planeId, planeId })
}

export async function airportOrders(airportId: number): Promise<OrderDto[]> {
  return await invoke<OrderDto[]>('airport_orders', { airport_id: airportId, airportId })
}

export type ModelDto = {
  name: string
  mtow: number
  cruise_speed: number
  fuel_capacity: number
  fuel_consumption: number
  operating_cost: number
  payload_capacity: number
  purchase_price: number
  min_runway_length: number
}

export async function listModels(): Promise<ModelDto[]> {
  return await invoke<ModelDto[]>('list_models')
}

export async function buyPlane(model: string, airportId: number): Promise<void> {
  await invoke('buy_plane_cmd', { model, airport_id: airportId, airportId })
}

export async function canFly(planeId: number, destId: number): Promise<boolean> {
  return await invoke<boolean>('plane_can_fly_to', { plane_id: planeId, dest_id: destId, planeId, destId })
}

export type FeasibilityDto = { ok: boolean; reason?: string }
export async function reachability(planeId: number, destId: number): Promise<FeasibilityDto> {
  return await invoke<FeasibilityDto>('plane_reachability', { plane_id: planeId, dest_id: destId, planeId, destId })
}

export async function startFromConfigYaml(yaml: string): Promise<void> {
  await invoke('start_from_config_yaml', { yaml })
}

export async function startFromConfigPath(path: string): Promise<void> {
  await invoke('start_from_config_path', { path })
}

export async function listSaves(): Promise<string[]> {
  return await invoke<string[]>('list_saves')
}
