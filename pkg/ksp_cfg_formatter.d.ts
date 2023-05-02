/* tslint:disable */
/* eslint-disable */
/**
* Can JS see this?
* @param {string} text
* @param {boolean} insert_spaces
* @param {number} tab_size
* @returns {string}
*/
export function ksp_fmt(text: string, insert_spaces: boolean, tab_size: number): string;
/**
* blah
* @param {string} text
* @returns {Uint32Array | undefined}
*/
export function ksp_validate(text: string): Uint32Array | undefined;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly ksp_fmt: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly ksp_validate: (a: number, b: number, c: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
