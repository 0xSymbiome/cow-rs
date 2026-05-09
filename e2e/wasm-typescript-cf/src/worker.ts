import init, {
  appDataHexToCid,
  computeOrderUid,
  supportedChainIds,
  wasmVersion,
  type OrderInput
} from "cow-sdk-wasm-test-package/cloudflare";
import wasmModule from "cow-sdk-wasm-test-package/cloudflare/wasm";

const ORDER: OrderInput = {
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
};

const OWNER = "0x3333333333333333333333333333333333333333";

let ready: Promise<void> | undefined;
let initCount = 0;

async function ensureReady() {
  if (!ready) {
    initCount += 1;
    ready = init({ module_or_path: wasmModule } as unknown as Parameters<typeof init>[0]).then(
      () => undefined
    );
  }
  await ready;
}

export default {
  async fetch(request: Request): Promise<Response> {
    await ensureReady();
    const url = new URL(request.url);

    if (url.pathname === "/version") {
      return Response.json({ version: wasmVersion() });
    }
    if (url.pathname === "/init-count") {
      return Response.json({ initCount });
    }
    if (url.pathname === "/chains") {
      return Response.json({ chains: Array.from(supportedChainIds()) });
    }
    if (url.pathname === "/cid") {
      return Response.json({ cid: appDataHexToCid(ORDER.appData) });
    }
    if (url.pathname === "/uid") {
      return Response.json(computeOrderUid(ORDER, 1, OWNER));
    }

    return new Response("not found", { status: 404 });
  }
} satisfies ExportedHandler;
