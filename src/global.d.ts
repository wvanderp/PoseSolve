declare module '/crates/solver/pkg/solver.js' {
  const value: any;
  export default value;
}

declare module '/crates/solver/pkg/solver_bg.wasm' {
  const value: any;
  export default value;
}

// Allow importing wasm-pack pkg by package name if needed
declare module 'crates/solver' {
  const value: any;
  export default value;
}
