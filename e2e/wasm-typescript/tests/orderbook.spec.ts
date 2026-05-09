import {
  appDataHexToCid,
  appDataInfo,
  cidToAppDataHex,
  computeOrderUid,
  deploymentAddresses,
  domainSeparator,
  orderTypedData,
  supportedChainIds,
  validateAppDataDoc,
  wasmVersion
} from "cow-sdk-wasm-test-package/nodejs";
import { describe, expect, test } from "vitest";

export const ORDER = {
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

export const OWNER = "0x3333333333333333333333333333333333333333";
export const CID = "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";

const APP_DATA_DOC = {
  appCode: "CoW Swap",
  metadata: {},
  version: "0.7.0"
};

describe("node package surface", () => {
  test("reports a crate version", () => {
    expect(wasmVersion()).toMatch(/^\d+\.\d+\.\d+/);
  });

  test("exposes supported chains", () => {
    expect(supportedChainIds()).toContain(1);
    expect(supportedChainIds()).toContain(100);
  });

  test("computes a domain separator", () => {
    expect(domainSeparator(1)).toMatch(/^0x[0-9a-f]{64}$/);
  });

  test("builds order typed data", () => {
    const typedData = orderTypedData(ORDER, 1);
    expect(typedData.primaryType).toBe("Order");
    expect(typedData.domain.chainId).toBe(1);
    expect(typedData.message.sellToken).toBe(ORDER.sellToken);
  });

  test("computes canonical order UID output", () => {
    const generated = computeOrderUid(ORDER, 1, OWNER);
    expect(generated.orderUid).toMatch(/^0x[0-9a-f]{112}$/);
    expect(generated.orderDigest).toMatch(/^0x[0-9a-f]{64}$/);
  });

  test("resolves deployment addresses", () => {
    const addresses = deploymentAddresses(1);
    expect(addresses.settlement).toMatch(/^0x[0-9a-fA-F]{40}$/);
    expect(addresses.vaultRelayer).toMatch(/^0x[0-9a-fA-F]{40}$/);
  });

  test("round-trips app-data hash and CID", () => {
    expect(appDataHexToCid(ORDER.appData)).toBe(CID);
    expect(cidToAppDataHex(CID)).toBe(ORDER.appData);
  });

  test("returns app-data info", () => {
    const info = appDataInfo(APP_DATA_DOC);
    expect(info.cid).toBe(CID);
    expect(info.appDataHex).toBe(ORDER.appData);
  });

  test("validates app-data document", () => {
    expect(validateAppDataDoc(APP_DATA_DOC).success).toBe(true);
  });
});
