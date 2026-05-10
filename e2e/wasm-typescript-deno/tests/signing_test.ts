import { loadDenoTarget } from "../src/index.ts";

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
const SIGNATURE =
  "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b";

function denoEnabled() {
  return Deno.env.get("BUILD_DENO") === "1";
}

Deno.test({
  name: "Deno target reports version",
  ignore: !denoEnabled(),
  async fn() {
    const sdk = await loadDenoTarget();
    if (!/^\d+\.\d+\.\d+/.test(sdk.wasmVersion())) {
      throw new Error("version is not semver-like");
    }
  }
});

Deno.test({
  name: "Deno target exposes supported chains",
  ignore: !denoEnabled(),
  async fn() {
    const sdk = await loadDenoTarget();
    if (!sdk.supportedChainIds().includes(1)) {
      throw new Error("mainnet chain id missing");
    }
  }
});

Deno.test({
  name: "Deno target converts app-data hash to CID",
  ignore: !denoEnabled(),
  async fn() {
    const sdk = await loadDenoTarget();
    const cid = sdk.appDataHexToCid(ORDER.appData).value;
    if (cid !== "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df") {
      throw new Error("CID mismatch");
    }
  }
});

Deno.test({
  name: "Deno target computes order UID",
  ignore: !denoEnabled(),
  async fn() {
    const sdk = await loadDenoTarget();
    const uid = sdk.computeOrderUid(ORDER, 1, OWNER).value.orderUid;
    if (!/^0x[0-9a-f]{112}$/.test(uid)) {
      throw new Error("UID shape mismatch");
    }
  }
});

Deno.test({
  name: "Deno target signs typed data through callback",
  ignore: !denoEnabled(),
  async fn() {
    const sdk = await loadDenoTarget();
    const signed = await sdk.signOrderWithTypedDataSigner(ORDER, 1, OWNER, () => SIGNATURE);
    if (signed.value.signingScheme !== "eip712") {
      throw new Error("signing scheme mismatch");
    }
  }
});

Deno.test({
  name: "Deno target rejects unsupported chain",
  ignore: !denoEnabled(),
  async fn() {
    const sdk = await loadDenoTarget();
    let failed = false;
    try {
      sdk.domainSeparator(13337);
    } catch {
      failed = true;
    }
    if (!failed) {
      throw new Error("unsupported chain was accepted");
    }
  }
});
