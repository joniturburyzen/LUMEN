/* tslint:disable */
/* eslint-disable */

export class Quill {
    free(): void;
    [Symbol.dispose](): void;
    constructor(canvas_id: string);
    render(time: number, w: number, h: number): void;
    set_tilt(x: number, y: number): void;
    tick(dt: number): void;
    tilt_x: number;
    tilt_y: number;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_quill_tilt_x: (a: number) => number;
    readonly __wbg_get_quill_tilt_y: (a: number) => number;
    readonly __wbg_quill_free: (a: number, b: number) => void;
    readonly __wbg_set_quill_tilt_x: (a: number, b: number) => void;
    readonly __wbg_set_quill_tilt_y: (a: number, b: number) => void;
    readonly quill_new: (a: number, b: number) => number;
    readonly quill_render: (a: number, b: number, c: number, d: number) => void;
    readonly quill_set_tilt: (a: number, b: number, c: number) => void;
    readonly quill_tick: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_exn_store: (a: number) => void;
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
