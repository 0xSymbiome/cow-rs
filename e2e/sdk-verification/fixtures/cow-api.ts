import type { Route } from "@playwright/test";

export const ORDERBOOK_VERSION_URL = "https://barn.api.cow.fi/mainnet/api/v1/version";
export const ORDERBOOK_QUOTE_URL = "https://barn.api.cow.fi/mainnet/api/v1/quote";
export const SUBGRAPH_URL_GLOB = "https://gateway.thegraph.com/api/mock-key/subgraphs/id/**";

export const OWNER = "0x4444444444444444444444444444444444444444";
export const MAINNET_WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
export const MAINNET_USDC = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
export const APP_DATA_HASH =
  "0x6caf30d0b35e6523444e6a6eb9c5562ba5480cdab16e00cb46963f1dc6cda0e1";

export type JsonRecord = Record<string, unknown>;

export function corsHeaders(contentType = "application/json"): Record<string, string> {
  return {
    "access-control-allow-origin": "*",
    "access-control-allow-headers": "content-type, x-api-key",
    "access-control-allow-methods": "GET, POST, OPTIONS",
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
    typeof requestBody.appDataHash === "string" ? requestBody.appDataHash : APP_DATA_HASH;
  const kind = requestBody.kind === "buy" ? "buy" : "sell";

  return {
    quote: {
      sellToken: requestBody.sellToken ?? MAINNET_WETH,
      buyToken: requestBody.buyToken ?? MAINNET_USDC,
      receiver: requestBody.receiver ?? requestBody.from ?? OWNER,
      sellAmount: "100000000000000000",
      buyAmount: "250000000",
      validTo: 1900000000,
      appData: appDataHash,
      appDataHash,
      feeAmount: "0",
      kind,
      partiallyFillable: false,
      sellTokenBalance: requestBody.sellTokenBalance ?? "erc20",
      buyTokenBalance: requestBody.buyTokenBalance ?? "erc20",
    },
    from: requestBody.from ?? OWNER,
    expiration: "2030-03-17T17:46:40Z",
    id: 27,
    verified: true,
    protocolFeeBps: "0",
  };
}

export function malformedQuoteResponse(): JsonRecord {
  return {
    quote: {
      sellToken: MAINNET_WETH,
    },
  };
}

export function validateQuoteRequestShape(body: unknown): string[] {
  if (!isRecord(body)) {
    return ["quote request body must be a JSON object"];
  }

  const issues: string[] = [];
  assertField(body, issues, "sellToken", MAINNET_WETH);
  assertField(body, issues, "buyToken", MAINNET_USDC);
  assertField(body, issues, "from", OWNER);

  const hasSellAmount = typeof body.sellAmountBeforeFee === "string";
  const hasBuyAmount = typeof body.buyAmountAfterFee === "string";
  if (hasSellAmount === hasBuyAmount) {
    issues.push("quote request must set exactly one of sellAmountBeforeFee or buyAmountAfterFee");
  }

  if (body.kind !== "sell" && body.kind !== "buy") {
    issues.push("quote request kind must be sell or buy");
  }

  return issues;
}

export function subgraphResponse(operationName: string | undefined): JsonRecord {
  switch (operationName) {
    case "Totals":
      return {
        data: {
          totals: [
            {
              tokens: "2",
              orders: "3",
              traders: "1",
              settlements: "1",
              volumeUsd: "10.25",
              volumeEth: "4.5",
              feesUsd: "0",
              feesEth: "0",
            },
          ],
        },
      };
    case "LastDaysVolume":
      return {
        data: {
          dailyTotals: [{ timestamp: 1900000000, volumeUsd: "123.45" }],
        },
      };
    case "LastHoursVolume":
      return {
        data: {
          hourlyTotals: [{ timestamp: 1900000000, volumeUsd: "12.34" }],
        },
      };
    default:
      return {
        errors: [{ message: `unexpected operation ${operationName ?? "<missing>"}` }],
      };
  }
}

export function emptyTotalsSubgraphResponse(): JsonRecord {
  return {
    data: {
      totals: [],
    },
  };
}

export function validateSubgraphRequestShape(body: unknown): string[] {
  if (!isRecord(body)) {
    return ["subgraph request body must be a JSON object"];
  }

  const issues: string[] = [];
  if (typeof body.query !== "string" || body.query.length === 0) {
    issues.push("subgraph request must include a query");
  }
  if (
    body.operationName !== "Totals" &&
    body.operationName !== "LastDaysVolume" &&
    body.operationName !== "LastHoursVolume"
  ) {
    issues.push("subgraph request operationName is not recognized");
  }
  return issues;
}

function assertField(
  body: JsonRecord,
  issues: string[],
  field: string,
  expected: string,
): void {
  if (body[field] !== expected) {
    issues.push(`quote request ${field} must be ${expected}`);
  }
}

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
