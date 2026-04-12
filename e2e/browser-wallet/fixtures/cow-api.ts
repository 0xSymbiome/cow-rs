import type { Page, Route } from "@playwright/test";

export type JsonRecord = Record<string, unknown>;

export const API_BASE_URL = "https://api.cow.fi/sepolia";
export const ORDERBOOK_VERSION_URL = `${API_BASE_URL}/api/v1/version`;
export const ORDERBOOK_QUOTE_URL = `${API_BASE_URL}/api/v1/quote`;
export const ORDERBOOK_ORDERS_URL = `${API_BASE_URL}/api/v1/orders`;
export const ORDERBOOK_APP_DATA_GLOB = `${API_BASE_URL}/api/v1/app_data/*`;

export const WRAPPED_NATIVE = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";
export const BUY_TOKEN = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";
export const ORDER_UID = `0x${"11".repeat(56)}`;

export interface BrowserWalletApiCapture {
  issues: string[];
  quoteBodies: JsonRecord[];
  appDataBodies: JsonRecord[];
  orderBodies: JsonRecord[];
  cancelBodies: JsonRecord[];
}

export function createApiCapture(): BrowserWalletApiCapture {
  return {
    issues: [],
    quoteBodies: [],
    appDataBodies: [],
    orderBodies: [],
    cancelBodies: [],
  };
}

export async function routeBrowserWalletOrderbook(
  page: Page,
  capture: BrowserWalletApiCapture,
): Promise<void> {
  await page.route(ORDERBOOK_VERSION_URL, async (route) => {
    if (route.request().method() !== "GET") {
      capture.issues.push(`version request used ${route.request().method()} instead of GET`);
    }
    await route.fulfill({
      status: 200,
      headers: corsHeaders("text/plain"),
      body: "mock-browser-wallet-orderbook",
    });
  });

  await page.route(ORDERBOOK_QUOTE_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "POST") {
      capture.issues.push(`quote request used ${route.request().method()} instead of POST`);
    }
    const body = requestJson(route, capture.issues, "quote");
    capture.issues.push(...validateQuoteRequestShape(body));
    if (isRecord(body)) {
      capture.quoteBodies.push(body);
    }
    await fulfillJson(route, quoteResponse(isRecord(body) ? body : {}));
  });

  await page.route(ORDERBOOK_APP_DATA_GLOB, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "PUT") {
      capture.issues.push(`app-data upload used ${route.request().method()} instead of PUT`);
    }
    const body = requestJson(route, capture.issues, "app-data upload");
    if (!isRecord(body) || typeof body.fullAppData !== "string" || body.fullAppData.length === 0) {
      capture.issues.push("app-data upload must include a non-empty fullAppData string");
    } else {
      capture.appDataBodies.push(body);
    }
    await fulfillJson(route, {
      fullAppData: isRecord(body) && typeof body.fullAppData === "string" ? body.fullAppData : "{}",
    });
  });

  await page.route(ORDERBOOK_ORDERS_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }

    if (route.request().method() === "POST") {
      const body = requestJson(route, capture.issues, "order submission");
      capture.issues.push(...validateOrderRequestShape(body));
      if (isRecord(body)) {
        capture.orderBodies.push(body);
      }
      await fulfillJson(route, ORDER_UID);
      return;
    }

    if (route.request().method() === "DELETE") {
      const body = requestJson(route, capture.issues, "order cancellation");
      capture.issues.push(...validateCancellationRequestShape(body));
      if (isRecord(body)) {
        capture.cancelBodies.push(body);
      }
      await route.fulfill({
        status: 200,
        headers: corsHeaders(),
        body: "",
      });
      return;
    }

    capture.issues.push(`orders request used unexpected method ${route.request().method()}`);
    await fulfillJson(route, { error: "unexpected method" }, 405);
  });
}

export function corsHeaders(contentType = "application/json"): Record<string, string> {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-headers": "content-type, x-api-key",
    "access-control-allow-methods": "GET, POST, PUT, DELETE, OPTIONS",
    "content-type": contentType,
  };
}

export async function fulfillPreflight(route: Route): Promise<void> {
  await route.fulfill({
    status: 204,
    headers: corsHeaders(),
    body: "",
  });
}

export async function fulfillJson(route: Route, body: unknown, status = 200): Promise<void> {
  await route.fulfill({
    status,
    headers: corsHeaders(),
    body: JSON.stringify(body),
  });
}

export function quoteResponse(requestBody: JsonRecord): JsonRecord {
  const appDataHash =
    typeof requestBody.appDataHash === "string" ? requestBody.appDataHash : `0x${"aa".repeat(32)}`;

  return {
    quote: {
      sellToken: requestBody.sellToken ?? WRAPPED_NATIVE,
      buyToken: requestBody.buyToken ?? BUY_TOKEN,
      receiver: requestBody.receiver ?? requestBody.from ?? `0x${"44".repeat(20)}`,
      sellAmount: "10000000000000000",
      buyAmount: "2500000000000000000",
      validTo: 1900000000,
      appData: appDataHash,
      appDataHash,
      feeAmount: "0",
      kind: requestBody.kind === "buy" ? "buy" : "sell",
      partiallyFillable: false,
      sellTokenBalance: requestBody.sellTokenBalance ?? "erc20",
      buyTokenBalance: requestBody.buyTokenBalance ?? "erc20",
    },
    from: requestBody.from ?? `0x${"44".repeat(20)}`,
    expiration: "2030-03-17T17:46:40Z",
    id: 81,
    verified: true,
    protocolFeeBps: "0",
  };
}

export function validateQuoteRequestShape(body: unknown): string[] {
  if (!isRecord(body)) {
    return ["quote request body must be a JSON object"];
  }

  const issues: string[] = [];
  assertField(body, issues, "sellToken", WRAPPED_NATIVE);
  assertField(body, issues, "buyToken", BUY_TOKEN);

  if (!isHexAddress(body.from)) {
    issues.push("quote request must include a valid from address");
  }
  if (body.kind !== "sell" && body.kind !== "buy") {
    issues.push("quote request kind must be sell or buy");
  }

  const hasSellAmount = typeof body.sellAmountBeforeFee === "string";
  const hasBuyAmount = typeof body.buyAmountAfterFee === "string";
  if (hasSellAmount === hasBuyAmount) {
    issues.push("quote request must set exactly one of sellAmountBeforeFee or buyAmountAfterFee");
  }

  if (typeof body.appDataHash !== "string" || !isHexValue(body.appDataHash, 64)) {
    issues.push("quote request must include a valid appDataHash");
  }

  return issues;
}

export function validateOrderRequestShape(body: unknown): string[] {
  if (!isRecord(body)) {
    return ["order submission body must be a JSON object"];
  }

  const issues: string[] = [];
  assertField(body, issues, "sellToken", WRAPPED_NATIVE);
  assertField(body, issues, "buyToken", BUY_TOKEN);

  if (!isHexAddress(body.from)) {
    issues.push("order submission must include a valid from address");
  }
  if (typeof body.signature !== "string" || !isHexValue(body.signature, 130)) {
    issues.push("order submission must include a 65-byte hex signature");
  }
  if (typeof body.signingScheme !== "string") {
    issues.push("order submission must include a signingScheme");
  }
  if (typeof body.quoteId !== "number") {
    issues.push("order submission must preserve the quoteId from the quote response");
  }
  if (!body.appData && !isHexValue(body.appDataHash, 64)) {
    issues.push("order submission must include appData or appDataHash");
  }

  return issues;
}

export function validateCancellationRequestShape(body: unknown): string[] {
  if (!isRecord(body)) {
    return ["order cancellation body must be a JSON object"];
  }

  const issues: string[] = [];
  if (!Array.isArray(body.orderUids) || body.orderUids.length !== 1 || body.orderUids[0] !== ORDER_UID) {
    issues.push("order cancellation must include the submitted order UID");
  }
  if (typeof body.signature !== "string" || !isHexValue(body.signature, 130)) {
    issues.push("order cancellation must include a 65-byte hex signature");
  }
  if (typeof body.signingScheme !== "string") {
    issues.push("order cancellation must include a signingScheme");
  }

  return issues;
}

function assertField(body: JsonRecord, issues: string[], field: string, expected: string): void {
  if (body[field] !== expected) {
    issues.push(`${field} must be ${expected}`);
  }
}

function requestJson(route: Route, issues: string[], label: string): unknown {
  try {
    return route.request().postDataJSON();
  } catch (error) {
    issues.push(`${label} body was not valid JSON: ${String(error)}`);
    return undefined;
  }
}

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isHexAddress(value: unknown): boolean {
  return typeof value === "string" && isHexValue(value, 40);
}

function isHexValue(value: unknown, hexChars: number): boolean {
  return typeof value === "string" && new RegExp(`^0x[a-fA-F0-9]{${hexChars}}$`).test(value);
}
