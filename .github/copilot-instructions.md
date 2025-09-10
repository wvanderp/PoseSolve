# PoseSolve - Camera Pose Estimation Application

**ALWAYS follow these instructions first and only fallback to additional search and context gathering if the information in the instructions is incomplete or found to be in error.**

PoseSolve is a React + TypeScript web application with a Rust WebAssembly computational core for estimating camera pose (position + orientation) from user-identified landmarks. The app runs entirely in-browser with offline capability.

## Working Effectively

### Bootstrap and Build (NEVER CANCEL - Allow full completion times)

Always run these commands in sequence on a fresh clone:

1. **Install Node.js dependencies**:

   ```bash
   CYPRESS_INSTALL_BINARY=0 npm install
   ```

   - **Takes ~10 seconds**. NEVER CANCEL.
   - **Network restriction note**: Cypress binary fails in firewall-restricted environments; use CYPRESS_INSTALL_BINARY=0 to skip.

2. **Install Rust toolchain** (if not already available):

   ```bash
   rustc --version && cargo --version || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   - **Takes 5-10 minutes**. NEVER CANCEL.
   - **CRITICAL**: Rust is required for WASM compilation.

3. **Install wasm-pack** (if not already available):

   ```bash
   cargo install wasm-pack
   ```

   - **Takes 2-3 minutes**. NEVER CANCEL. Set timeout to 300+ seconds.

4. **Configure WASM build for network restrictions**:

   - The Cargo.toml in `crates/solver/Cargo.toml` includes `wasm-opt = false` to avoid downloading binaryen in restricted environments.
   - **Never remove this setting** in firewall-restricted environments.

5. **Build WASM package**:
   ```bash
   npm run build:wasm
   ```
   - **Takes 10-15 seconds initially, <1 second for incremental builds**. NEVER CANCEL.
   - **Network restriction note**: Fails without `wasm-opt = false` configuration.

### Production Build and Testing

```bash
npm run build
```

- **Takes ~2 seconds**. NEVER CANCEL.
- **Output**: Creates `dist/` directory with optimized build
- **Preview**: `npm run preview` serves on http://localhost:4173

## Validation and Testing

### Manual Application Testing

**ALWAYS test the application functionality after making changes:**

1. **Start dev server**: `npm run dev`
2. **Open browser**: Navigate to http://localhost:5173
3. **Verify UI components**:

   - Left panel: Image upload area with "Choose File" button
   - Right panel: Map (Leaflet integration - tiles may be blocked by network restrictions)
   - Bottom: "Solve" button and status indicators
   - Interface: Zoom controls, file upload, help tips

4. **Test basic interactions**:
   - File upload dialog opens when clicking "Choose File"
   - Map controls (zoom in/out) respond
   - Status shows "Points: 0 px, 0 world, links: 0"
   - Application loads without JavaScript errors (check browser console)

### Type Checking

```bash
npx tsc --noEmit --skipLibCheck
```

- **Takes ~5 seconds**. NEVER CANCEL.
- **Note**: Test files show errors for missing test dependencies (vitest, @testing-library/react) - this is expected.
- **Alternative**: Use `--skipLibCheck` flag to ignore test file errors and focus on main application code.

### Component Tests

```bash
npm run ct:run
```

- **Network restriction note**: **Cypress binary fails in firewall-restricted environments**. This is expected and documented.
- **Alternative**: Use `npx tsc --noEmit` for basic validation instead.

### Rust Tests

```bash
cd crates/solver && cargo test
```

- **Takes ~10 seconds**. NEVER CANCEL.

## Architecture Overview

### Project Structure

```
├── src/                    # React TypeScript frontend
│   ├── components/         # UI components (ImageCanvas, WorldMap, etc.)
│   ├── state/             # Zustand state management
│   ├── worker/            # Web Worker integration (Comlink)
│   └── types/             # TypeScript type definitions
├── crates/solver/         # Rust WebAssembly solver
│   ├── src/lib.rs         # WASM bindings and algorithms
│   └── Cargo.toml         # Rust dependencies and WASM config
├── cypress/               # Component tests (may fail in restricted environments)
├── package.json           # Node.js dependencies and scripts
├── vite.config.ts         # Build configuration
└── index.html             # Entry point
```

### Key Technologies

- **Frontend**: React 19, TypeScript, Vite, Tailwind CSS
- **State**: Zustand + React Query
- **Maps**: Leaflet with OpenStreetMap tiles
- **Computation**: Rust → WebAssembly (via wasm-pack)
- **Worker**: Web Worker with Comlink for smooth UI
- **Testing**: Cypress component tests, TypeScript checking

## Common Tasks and Commands

### Development Workflow

```bash
# Start development (most common)
npm run dev                    # ~200ms startup

# Full build validation
npm run build                  # ~2 seconds

# Type checking
npx tsc --noEmit --skipLibCheck   # ~5 seconds

# WASM rebuild (after Rust changes)
npm run build:wasm            # ~10 seconds initially, <1s incremental
```

### File Locations

- **Main app**: `src/App.tsx`
- **Image panel**: `src/components/ImageCanvas.tsx`
- **Map panel**: `src/components/WorldMap.tsx`
- **State management**: `src/state/store.ts`
- **WASM solver**: `crates/solver/src/lib.rs`
- **Types**: `src/types/` directory

## Network Restrictions and Workarounds

### Known Network-Related Issues

1. **Cypress binary download fails**: Use `CYPRESS_INSTALL_BINARY=0 npm install`
2. **WASM optimization fails**: Requires `wasm-opt = false` in Cargo.toml
3. **Map tiles blocked**: Expected in restricted environments; application still functional
4. **binaryen download fails**: Resolved by disabling wasm-opt

### Environment Setup for Restricted Networks

```bash
# Install with workarounds
CYPRESS_INSTALL_BINARY=0 npm install
npm run build:wasm  # Ensure wasm-opt = false is set
npm run dev
```

## Troubleshooting

### Build Failures

- **"wasm-pack not found"**: Run `cargo install wasm-pack` (takes 2-3 minutes)
- **"binaryen download failed"**: Ensure `wasm-opt = false` in `crates/solver/Cargo.toml`
- **TypeScript errors in tests**: Install test dependencies: `npm install --save-dev vitest @testing-library/react`

### Runtime Issues

- **Map tiles not loading**: Expected in network-restricted environments; core functionality works
- **WASM worker fails**: Check browser console; rebuild WASM with `npm run build:wasm`
- **File upload not working**: Check browser compatibility and console errors

## CI/CD Integration

Currently no GitHub Actions workflow exists. When adding CI:

- Use `CYPRESS_INSTALL_BINARY=0` for environments without display
- Set timeouts appropriately: 300+ seconds for wasm-pack install
- Consider caching `~/.cargo` and `node_modules`

## Screenshots

![PoseSolve Application UI](https://github.com/user-attachments/assets/5aab9861-0527-44f1-8daf-125716025ccf)

## Performance Expectations

- **npm install**: ~10 seconds (with Cypress skip)
- **wasm-pack install**: 2-3 minutes (one-time setup)
- **npm run build:wasm**: 10-15 seconds initial, <1 second incremental
- **npm run build**: ~2 seconds
- **npm run dev startup**: ~200ms
- **Application load**: <1 second after server start

**CRITICAL REMINDER**: NEVER CANCEL long-running commands. Build tools require time to complete. Always set appropriate timeouts and wait for completion.
