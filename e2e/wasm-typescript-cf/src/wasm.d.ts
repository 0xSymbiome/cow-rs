declare module "cow-sdk-js-test-package/trading/edge/wasm" {
  const wasmModule: WebAssembly.Module;
  export default wasmModule;
}

declare module "*?raw" {
  const source: string;
  export default source;
}
