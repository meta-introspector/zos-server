/* tslint:disable */
/* eslint-disable */

export class SolanaP2P {
    free(): void;
    [Symbol.dispose](): void;
    get_block(slot: bigint): Promise<any>;
    get_contract(): string;
    get_signatures(): Promise<any>;
    name(): string;
    constructor();
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_solanap2p_free: (a: number, b: number) => void;
    readonly solanap2p_get_block: (a: number, b: bigint) => any;
    readonly solanap2p_get_contract: (a: number) => [number, number];
    readonly solanap2p_get_signatures: (a: number) => any;
    readonly solanap2p_name: (a: number) => [number, number];
    readonly solanap2p_new: () => number;
    readonly wasm_bindgen__closure__destroy__hdc2716501c7b37f8: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__ha78d98c47eec2a46: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h9319af9da0e7ecb4: (a: number, b: number, c: any) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
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
