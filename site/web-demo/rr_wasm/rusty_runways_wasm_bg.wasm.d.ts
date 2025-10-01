/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const new_game: (a: number, b: bigint, c: number, d: number) => void;
export const observe: () => [number, number, number];
export const advance: (a: bigint) => [number, number, number];
export const plane_info: (a: number) => [number, number, number];
export const airport_orders: (a: number) => [number, number, number];
export const depart_plane: (a: number, b: number) => [number, number];
export const refuel_plane: (a: number) => [number, number];
export const maintenance: (a: number) => [number, number];
export const load_order: (a: number, b: number) => [number, number];
export const unload_order: (a: number, b: number) => [number, number];
export const unload_all: (a: number) => [number, number];
export const list_models: () => [number, number, number];
export const buy_plane: (a: number, b: number, c: number) => [number, number];
export const plane_can_fly_to: (a: number, b: number) => [number, number, number];
export const plane_reachability: (a: number, b: number) => [number, number, number];
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_export_2: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
