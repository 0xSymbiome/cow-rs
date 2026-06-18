declare module "cow-sdk-wasm-test-package/trading/edge/wasm" {
  const wasmModule: WebAssembly.Module;
  export default wasmModule;
}

declare module "*?raw" {
  const source: string;
  export default source;
}
