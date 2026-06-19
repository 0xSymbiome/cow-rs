import {
  appDataHexToCid,
  cidToAppDataHex,
  computeOrderUid,
  domainSeparator,
  initialize,
  orderTypedData,
  supportedChainIds,
  wasmVersion
} from "cow-sdk-wasm-test-package";

const ORDER = {
  sellToken: "0x1111111111111111111111111111111111111111",
  buyToken: "0x2222222222222222222222222222222222222222",
  receiver: "0x4444444444444444444444444444444444444444",
  sellAmount: "1000000000000000000",
  buyAmount: "2000000000000000000",
  validTo: 1735689600,
  appData: "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df",
  feeAmount: "0",
  kind: "sell",
  partiallyFillable: false,
  sellTokenBalance: "erc20",
  buyTokenBalance: "erc20"
} as const;

const OWNER = "0x3333333333333333333333333333333333333333";

export async function runBrowserSmoke() {
  // The browser (`browser`/`import`) condition resolves the web-target build, so
  // the wasm module is instantiated explicitly once before any export runs — the
  // loader fetches it through `new URL(import.meta.url)`, which Vite emits as an
  // asset. (Node tests keep the auto-initializing CommonJS build.)
  await initialize();
  const cid = appDataHexToCid(ORDER.appData).value;
  const uid = computeOrderUid(ORDER, 1, OWNER).value;
  const typedData = orderTypedData(ORDER, 1).value;
  const result = {
    chainIds: supportedChainIds(),
    cid,
    hash: cidToAppDataHex(cid).value,
    domainSeparator: domainSeparator(1),
    primaryType: typedData.primaryType,
    uid: uid.orderUid,
    version: wasmVersion()
  };

  window.__cowSdkWasmSmoke = result;
  document.querySelector("#root")!.textContent = JSON.stringify(result);
  return result;
}

declare global {
  interface Window {
    __cowSdkWasmSmoke?: Awaited<ReturnType<typeof runBrowserSmoke>>;
  }
}

void runBrowserSmoke();
