import { invoke } from '@tauri-apps/api/core'
import { isTauri } from '@/lib/tauri'

// Compute a URL that works in both dev (vite, base "/") and
// on GitHub Pages where the site is served under a subpath, e.g. 
// /RustyRunways/web-demo/.
function wasmModulePath(): string {
  // Served from public/rr_wasm by build_web_demo.sh into docs/web-demo/rr_wasm
  // Resolve relative to the current page directory to avoid root-absolute paths.
  if (typeof window === 'undefined') {
    return '/rr_wasm/rusty_runways_wasm.js'
  }
  const pathname = window.location.pathname
  const baseDir = pathname.endsWith('/') ? pathname : pathname.replace(/[^/]*$/, '')
  return `${baseDir}rr_wasm/rusty_runways_wasm.js`
}

export type Observation = {
  time: number
  cash: number
  airports: { id: number; name: string; x: number; y: number; fuel_price: number; runway_length: number; num_orders: number }[]
  planes: {
    id: number
    model: string
    x: number
    y: number
    status: string
    destination?: number | null
    hours_remaining?: number | null
    fuel: { current: number; capacity: number }
    payload: {
      cargo_current: number
      cargo_capacity: number
      passenger_current: number
      passenger_capacity: number
    }
  }[]
}

export async function newGame(seed: string | undefined, airportCount: number, startingCash: number): Promise<void> {
  const parsedSeed = seed && seed.trim() !== '' ? Number(seed) : undefined
  if (isTauri()) {
    await invoke('new_game', {
      args: {
        seed: parsedSeed,
        numAirports: airportCount,
        startingCash: startingCash,
      },
    })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    wasm.new_game(parsedSeed, airportCount, startingCash)
  }
}

export async function loadGame(name: string): Promise<void> {
  await invoke('load_game_cmd', { name })
}

export async function saveGame(name: string): Promise<void> {
  await invoke('save_game_cmd', { name })
}

export async function observe(): Promise<Observation> {
  if (isTauri()) {
    return await invoke<Observation>('observe')
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return (await wasm.observe()) as Observation
  }
}

export async function advance(hours = 1): Promise<Observation> {
  if (isTauri()) {
    return await invoke<Observation>('advance', { hours })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return (await wasm.advance(hours)) as Observation
  }
}

export async function departPlane(plane: number, dest: number): Promise<void> {
  if (isTauri()) {
    await invoke('depart_plane', { plane, dest })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.depart_plane(plane, dest)
  }
}

export async function refuelPlane(plane: number): Promise<void> {
  if (isTauri()) {
    await invoke('refuel_plane', { plane })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.refuel_plane(plane)
  }
}

export async function sellPlane(plane: number): Promise<number> {
  if (isTauri()) {
    return await invoke<number>('sell_plane_cmd', { plane })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return await wasm.sell_plane(plane)
  }
}

export async function maintenance(plane: number): Promise<void> {
  if (isTauri()) {
    await invoke('maintenance', { plane })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.maintenance(plane)
  }
}

export async function loadOrder(order: number, plane: number): Promise<void> {
  if (isTauri()) {
    await invoke('load_order', { order, plane })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.load_order(order, plane)
  }
}

export async function unloadOrder(order: number, plane: number): Promise<void> {
  if (isTauri()) {
    await invoke('unload_order', { order, plane })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.unload_order(order, plane)
  }
}

export async function unloadAll(plane: number): Promise<void> {
  if (isTauri()) {
    await invoke('unload_all', { plane })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.unload_all(plane)
  }
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
  passenger_current: number
  passenger_capacity: number
  current_airport_id: number | null
  manifest: OrderDto[]
}

export type OrderDto = {
  id: number
  destination_id: number
  value: number
  deadline: number
  payload_kind: string
  cargo_type?: string
  weight?: number
  passenger_count?: number
}

export async function planeInfo(planeId: number): Promise<PlaneInfo> {
  if (isTauri()) {
    return await invoke<PlaneInfo>('plane_info', { plane_id: planeId, planeId })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return (await wasm.plane_info(planeId)) as PlaneInfo
  }
}

export async function airportOrders(airportId: number): Promise<OrderDto[]> {
  if (isTauri()) {
    return await invoke<OrderDto[]>('airport_orders', { airport_id: airportId, airportId })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return (await wasm.airport_orders(airportId)) as OrderDto[]
  }
}

export type ModelDto = {
  name: string
  mtow: number
  cruise_speed: number
  fuel_capacity: number
  fuel_consumption: number
  operating_cost: number
  payload_capacity: number
  passenger_capacity: number
  purchase_price: number
  min_runway_length: number
  role: string
}

export async function listModels(): Promise<ModelDto[]> {
  if (isTauri()) {
    return await invoke<ModelDto[]>('list_models')
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return (await wasm.list_models()) as ModelDto[]
  }
}

export async function buyPlane(model: string, airportId: number): Promise<void> {
  if (isTauri()) {
    await invoke('buy_plane_cmd', { model, airport_id: airportId, airportId })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    await wasm.buy_plane(model, airportId)
  }
}

export async function canFly(planeId: number, destId: number): Promise<boolean> {
  if (isTauri()) {
    return await invoke<boolean>('plane_can_fly_to', { plane_id: planeId, dest_id: destId, planeId, destId })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return await wasm.plane_can_fly_to(planeId, destId)
  }
}

export type FeasibilityDto = { ok: boolean; reason?: string }
export async function reachability(planeId: number, destId: number): Promise<FeasibilityDto> {
  if (isTauri()) {
    return await invoke<FeasibilityDto>('plane_reachability', { plane_id: planeId, dest_id: destId, planeId, destId })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    return (await wasm.plane_reachability(planeId, destId)) as FeasibilityDto
  }
}

export async function startFromConfigYaml(yaml: string): Promise<void> {
  if (isTauri()) {
    await invoke('start_from_config_yaml', { yaml })
  } else {
    const wasm = await import(/* @vite-ignore */ wasmModulePath())
    // TODO: parse YAML in wasm; for demo start with defaults
    wasm.new_game(undefined, 10, 100000)
  }
}

export async function startFromConfigPath(path: string): Promise<void> {
  await invoke('start_from_config_path', { path })
}

export async function listSaves(): Promise<string[]> {
  return await invoke<string[]>('list_saves')
}
