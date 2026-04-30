import type { Page, Route } from "@playwright/test";

export const ORDERBOOK_VERSION_URL = "https://barn.api.cow.fi/mainnet/api/v1/version";
export const ORDERBOOK_QUOTE_URL = "https://barn.api.cow.fi/mainnet/api/v1/quote";
export const ORDERBOOK_SOLVER_COMPETITION_LATEST_URL =
  "https://barn.api.cow.fi/mainnet/api/v1/solver_competition/latest";
export const ORDERBOOK_ORDER_BASE_URL = "https://barn.api.cow.fi/mainnet/api/v1/orders";
export const ORDERBOOK_ORDER_URL_GLOB = `${ORDERBOOK_ORDER_BASE_URL}/*`;
export const ORDERBOOK_TRADES_URL_GLOB = "https://barn.api.cow.fi/mainnet/api/v2/trades**";
export const ORDERBOOK_APP_DATA_BASE_URL = "https://barn.api.cow.fi/mainnet/api/v1/app_data";
export const ORDERBOOK_APP_DATA_URL_GLOB = `${ORDERBOOK_APP_DATA_BASE_URL}/*`;
export const SUBGRAPH_URL_GLOB = "https://gateway.thegraph.com/api/mock-key/subgraphs/id/**";

export const OWNER = "0x4444444444444444444444444444444444444444";
export const MAINNET_WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
export const MAINNET_USDC = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
export const MAINNET_SETTLEMENT = "0x9008d19f58aabd9ed0d60971565aa8510560ab41";
export const APP_DATA_HASH =
  "0x6caf30d0b35e6523444e6a6eb9c5562ba5480cdab16e00cb46963f1dc6cda0e1";
export const DEFAULT_ORDER_UID = `0x${"11".repeat(56)}`;
export const DEFAULT_TX_HASH = `0x${"aa".repeat(32)}`;

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
  assertAddressField(body, issues, "sellToken", MAINNET_WETH);
  assertAddressField(body, issues, "buyToken", MAINNET_USDC);
  assertAddressField(body, issues, "from", OWNER);

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

function assertAddressField(
  body: JsonRecord,
  issues: string[],
  field: string,
  expected: string,
): void {
  const actual = body[field];
  if (typeof actual !== "string" || actual.toLowerCase() !== expected.toLowerCase()) {
    issues.push(`quote request ${field} must be ${expected}`);
  }
}

function isRecord(value: unknown): value is JsonRecord {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

export function defaultLatestCompetitionPayload(): JsonRecord {
  return {
    auctionId: 27,
    authBlock: 21_000_000,
    transactionHashes: [DEFAULT_TX_HASH],
    solutions: [
      {
        solver: "0x0000000000000000000000000000000000000001",
        solverAddress: "0x0000000000000000000000000000000000000001",
        score: "1",
        ranking: 1,
        ref: "mock-solver",
        orders: [
          {
            id: DEFAULT_ORDER_UID,
            uid: DEFAULT_ORDER_UID,
            owner: OWNER,
            appData: APP_DATA_HASH,
            sellToken: MAINNET_WETH,
            buyToken: MAINNET_USDC,
            executedSellAmount: "100000000000000000",
            executedBuyAmount: "250000000",
          },
        ],
        clearingPrices: {
          [MAINNET_WETH]: "1",
          [MAINNET_USDC]: "1",
        },
      },
    ],
  };
}

export function defaultOrderPayload(): JsonRecord {
  return {
    uid: DEFAULT_ORDER_UID,
    owner: OWNER,
    appData: APP_DATA_HASH,
    appDataHash: APP_DATA_HASH,
    sellToken: MAINNET_WETH,
    buyToken: MAINNET_USDC,
    receiver: OWNER,
    sellAmount: "100000000000000000",
    buyAmount: "250000000",
    feeAmount: "0",
    kind: "sell",
    partiallyFillable: false,
    sellTokenBalance: "erc20",
    buyTokenBalance: "erc20",
    validTo: 1_900_000_000,
    creationDate: "2030-03-17T17:46:40Z",
    status: "open",
    class: "limit",
    signingScheme: "eip712",
    signature: `0x${"02".repeat(65)}`,
    settlementContract: MAINNET_SETTLEMENT,
    executedSellAmount: "0",
    executedBuyAmount: "0",
    executedFeeAmount: "0",
    executedSurplusFee: "0",
    invalidated: false,
  };
}

export function defaultTradesPayload(): JsonRecord[] {
  return [
    {
      blockNumber: 21_000_000,
      logIndex: 1,
      orderUid: DEFAULT_ORDER_UID,
      owner: OWNER,
      sellToken: MAINNET_WETH,
      buyToken: MAINNET_USDC,
      sellAmount: "100000000000000000",
      buyAmount: "250000000",
      sellAmountBeforeFees: "100000000000000000",
      txHash: DEFAULT_TX_HASH,
    },
  ];
}

export function defaultAppDataPayload(): JsonRecord {
  return {
    fullAppData: JSON.stringify({
      version: "1.14.0",
      appCode: "cow-rs/wasm-console",
      environment: "browser",
      metadata: {
        quote: {
          slippageBips: 50,
        },
      },
    }),
  };
}

export interface RouteCaptureOptions {
  issues?: string[];
}

export interface LatestCompetitionRouteOptions extends RouteCaptureOptions {
  body?: JsonRecord;
}

export async function routeSolverCompetitionLatest(
  page: Page,
  options: LatestCompetitionRouteOptions = {},
): Promise<void> {
  await page.route(ORDERBOOK_SOLVER_COMPETITION_LATEST_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      options.issues?.push(
        `solver_competition/latest request used ${route.request().method()} instead of GET`,
      );
    }
    await fulfillJson(route, options.body ?? defaultLatestCompetitionPayload());
  });
}

export interface OrderByUidRouteOptions extends RouteCaptureOptions {
  bodyByUid?: Record<string, JsonRecord>;
  fallback?: JsonRecord;
}

export async function routeOrderByUid(
  page: Page,
  options: OrderByUidRouteOptions = {},
): Promise<void> {
  await page.route(ORDERBOOK_ORDER_URL_GLOB, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      options.issues?.push(
        `orders/{uid} request used ${route.request().method()} instead of GET`,
      );
    }
    const uid = uidFromUrl(route.request().url());
    const body =
      (uid && options.bodyByUid?.[uid]) ?? options.fallback ?? defaultOrderPayload();
    await fulfillJson(route, body);
  });
}

export interface OrderTradesRouteOptions extends RouteCaptureOptions {
  body?: JsonRecord[];
  byOrderUid?: Record<string, JsonRecord[]>;
}

export async function routeOrderTrades(
  page: Page,
  options: OrderTradesRouteOptions = {},
): Promise<void> {
  await page.route(ORDERBOOK_TRADES_URL_GLOB, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      options.issues?.push(
        `trades request used ${route.request().method()} instead of GET`,
      );
    }
    const orderUid = orderUidFromQuery(route.request().url());
    const body =
      (orderUid && options.byOrderUid?.[orderUid]) ?? options.body ?? defaultTradesPayload();
    await fulfillJson(route, body);
  });
}

export interface AppDataRouteOptions extends RouteCaptureOptions {
  bodyByHash?: Record<string, JsonRecord>;
  fallback?: JsonRecord;
}

export async function routeAppData(
  page: Page,
  options: AppDataRouteOptions = {},
): Promise<void> {
  await page.route(ORDERBOOK_APP_DATA_URL_GLOB, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      options.issues?.push(
        `app_data/{hash} request used ${route.request().method()} instead of GET`,
      );
    }
    const hash = hashFromUrl(route.request().url());
    const body =
      (hash && options.bodyByHash?.[hash]) ?? options.fallback ?? defaultAppDataPayload();
    await fulfillJson(route, body);
  });
}

export interface SubgraphQueryRouteOptions extends RouteCaptureOptions {
  captured?: JsonRecord[];
  validate?: boolean;
}

export type SubgraphMatcher = string | ((body: JsonRecord) => boolean);

export async function routeSubgraphQuery(
  page: Page,
  matcher: SubgraphMatcher,
  response: JsonRecord,
  options: SubgraphQueryRouteOptions = {},
): Promise<void> {
  await page.route(SUBGRAPH_URL_GLOB, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "POST") {
      options.issues?.push(
        `subgraph request used ${route.request().method()} instead of POST`,
      );
    }

    let body: JsonRecord = {};
    try {
      const parsed = route.request().postDataJSON();
      if (isRecord(parsed)) {
        body = parsed;
      } else {
        options.issues?.push("subgraph request body was not a JSON object");
      }
    } catch (error) {
      options.issues?.push(`subgraph request body was not valid JSON: ${String(error)}`);
    }

    if (options.validate !== false) {
      options.issues?.push(...validateSubgraphRequestShape(body));
    }
    options.captured?.push(body);

    if (matchesSubgraphRequest(matcher, body)) {
      await fulfillJson(route, response);
      return;
    }

    const operationName =
      typeof body.operationName === "string" ? body.operationName : undefined;
    await fulfillJson(route, subgraphResponse(operationName));
  });
}

function matchesSubgraphRequest(matcher: SubgraphMatcher, body: JsonRecord): boolean {
  if (typeof matcher === "string") {
    if (typeof body.operationName === "string" && body.operationName === matcher) {
      return true;
    }
    return typeof body.query === "string" && body.query.includes(matcher);
  }
  return matcher(body);
}

function uidFromUrl(url: string): string {
  const [pathOnly] = url.split("?", 1);
  const segments = pathOnly.split("/");
  return segments[segments.length - 1] ?? "";
}

function hashFromUrl(url: string): string {
  return uidFromUrl(url);
}

function orderUidFromQuery(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.searchParams.get("orderUid") ?? "";
  } catch {
    return "";
  }
}
