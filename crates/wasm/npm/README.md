# cow-sdk-wasm package

TypeScript-callable WebAssembly bindings for the CoW Protocol Rust SDK.

Build the package artifacts with:

```sh
bash crates/wasm/npm/scripts/build.sh
```

The generated `package.json` is rendered from `package.template.json`; set
`NPM_PACKAGE_NAME` when preparing a named package.
