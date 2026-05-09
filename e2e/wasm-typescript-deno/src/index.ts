export async function loadDenoTarget() {
  if (Deno.env.get("BUILD_DENO") !== "1") {
    throw new Error("Deno wasm target is only available when BUILD_DENO=1");
  }
  const sdk = await import("../../../crates/wasm/npm/dist/deno/cow_sdk_wasm.js");
  sdk.__cow_sdk_wasm_init();
  return sdk;
}
