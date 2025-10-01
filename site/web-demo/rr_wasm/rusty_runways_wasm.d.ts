/* tslint:disable */
/* eslint-disable */
export function new_game(seed: bigint | null | undefined, num_airports: number | null | undefined, starting_cash: number): void;
export function observe(): any;
export function advance(hours: bigint): any;
export function plane_info(plane_id: number): any;
export function airport_orders(airport_id: number): any;
export function depart_plane(plane: number, dest: number): void;
export function refuel_plane(plane: number): void;
export function maintenance(plane: number): void;
export function load_order(order: number, plane: number): void;
export function unload_order(order: number, plane: number): void;
export function unload_all(plane: number): void;
export function list_models(): any;
export function buy_plane(model: string, airport_id: number): void;
export function plane_can_fly_to(plane_id: number, dest_id: number): boolean;
export function plane_reachability(plane_id: number, dest_id: number): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly new_game: (a: number, b: bigint, c: number, d: number) => void;
  readonly observe: () => [number, number, number];
  readonly advance: (a: bigint) => [number, number, number];
  readonly plane_info: (a: number) => [number, number, number];
  readonly airport_orders: (a: number) => [number, number, number];
  readonly depart_plane: (a: number, b: number) => [number, number];
  readonly refuel_plane: (a: number) => [number, number];
  readonly maintenance: (a: number) => [number, number];
  readonly load_order: (a: number, b: number) => [number, number];
  readonly unload_order: (a: number, b: number) => [number, number];
  readonly unload_all: (a: number) => [number, number];
  readonly list_models: () => [number, number, number];
  readonly buy_plane: (a: number, b: number, c: number) => [number, number];
  readonly plane_can_fly_to: (a: number, b: number) => [number, number, number];
  readonly plane_reachability: (a: number, b: number) => [number, number, number];
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
