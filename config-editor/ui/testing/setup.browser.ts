import init from '../../pkg/engine';
import wasmUrl from '../../pkg/engine_bg.wasm?url';

// The browser component tests use the real Rust engine, so initialize it once.
await init({ module_or_path: wasmUrl });
