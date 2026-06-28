import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

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
export const APP_DATA_CONTENT = '{"appCode":"CoW Swap","metadata":{},"version":"0.7.0"}';

export function defaultFacade(): any {
  return require("../dist/default/index.cjs");
}

export function orderbookFacade(): any {
  return require("../dist/orderbook/index.cjs");
}

export function signingFacade(): any {
  return require("../dist/signing/index.cjs");
}
