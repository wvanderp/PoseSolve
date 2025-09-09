# Development setup

This repo contains a Vite + React (TypeScript) frontend and a Rust→WASM solver crate wired through a Web Worker (Comlink).

## Prereqs
- Node.js 18+ and npm
- Rust toolchain (stable) and `wasm-pack`

## 1) Install Node deps
```powershell
npm install
```

## 2) Run the dev server
```powershell
npm run dev
```
Open http://localhost:5173. The app runs with a stub solver if the WASM package is not built yet.

## 3) Install Rust + wasm-pack (Windows)
- Install Rust (MSVC toolchain): https://rustup.rs/
  - After install, restart your terminal and verify:
    ```powershell
    rustc --version
    cargo --version
    ```
- Install wasm-pack:
  ```powershell
  cargo install wasm-pack
  ```

## 4) Build the WASM solver
```powershell
npm run build:wasm
```
This creates `crates/solver/pkg/solver.js` and `solver_bg.wasm`. Reload the app; the worker will load the real WASM functions.

## Notes
- Dev server: Ctrl+C to stop. Use `npm run preview` to test a production build locally.
- If the worker warns about WASM not initialized, make sure step 4 succeeded.
