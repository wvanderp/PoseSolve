# solver (WASM)

Rust crate compiled to WebAssembly via wasm-bindgen.

Build locally:

```
wasm-pack build crates/solver --target web --release
```

This generates `crates/solver/pkg/solver.js` and `solver_bg.wasm` that the web app loads in a Web Worker.
