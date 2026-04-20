import { expect, test, type Page } from "@playwright/test";

import {
  APP_DATA_HASH,
  DEFAULT_ORDER_UID,
  MAINNET_USDC,
  MAINNET_WETH,
  OWNER,
  defaultAppDataPayload,
  defaultLatestCompetitionPayload,
  defaultOrderPayload,
  defaultTradesPayload,
  routeAppData,
  routeOrderByUid,
  routeOrderTrades,
  routeSolverCompetitionLatest,
  routeSubgraphQuery,
  subgraphResponse,
  type JsonRecord,
} from "../fixtures/cow-api";

let browserErrors: string[];

test.beforeEach(async ({ page }) => {
  browserErrors = [];
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      browserErrors.push(message.text());
    }
  });
});

test.afterEach(() => {
  expect(browserErrors).toEqual([]);
});

test("manual-network: Latest Competition renders deterministic solver-competition payload", async ({
  page,
}) => {
  const issues: string[] = [];
  await routeSolverCompetitionLatest(page, { issues });
  await loadConsole(page);

  await page.locator("#btn-ob-auction").click();
  const payload = await outputJson<JsonRecord>(page, "#orderbook-output");

  const expected = defaultLatestCompetitionPayload();
  expect(payload.auctionId).toBe(expected.auctionId);
  expect(payload.transactionHashes).toEqual(expected.transactionHashes);

  const solutions = payload.solutions as JsonRecord[];
  expect(Array.isArray(solutions)).toBe(true);
  expect(solutions).toHaveLength(1);

  // The reviewed solver-settlement contract tracks ranking, solver address,
  // score, and clearing prices. Order lists inside a settlement are not part
  // of the typed boundary and are therefore not re-serialised by the SDK.
  expect(solutions[0].ranking).toBe(1);
  expect(solutions[0].solverAddress).toBe("0x0000000000000000000000000000000000000001");
  expect(solutions[0].score).toBe("1");
  const clearingPrices = solutions[0].clearingPrices as Record<string, string>;
  expect(clearingPrices).toBeDefined();
  const clearingKeys = Object.keys(clearingPrices).map((key) => key.toLowerCase());
  expect(clearingKeys).toEqual(
    expect.arrayContaining([MAINNET_WETH.toLowerCase(), MAINNET_USDC.toLowerCase()]),
  );

  expect(issues).toEqual([]);
});

test("manual-network: Order panel resolves seeded uid into deterministic order envelope", async ({
  page,
}) => {
  const issues: string[] = [];
  await routeOrderByUid(page, { issues });
  await loadConsole(page);

  await page.locator("#lookup-order-uid").fill(DEFAULT_ORDER_UID);
  await page.locator("#btn-ob-order").click();

  const order = await outputJson<JsonRecord>(page, "#orderbook-output");
  const expected = defaultOrderPayload();
  expect(order.uid).toBe(expected.uid);
  expectAddressEqual(order.owner as string, OWNER);
  expectAddressEqual(order.sellToken as string, MAINNET_WETH);
  expectAddressEqual(order.buyToken as string, MAINNET_USDC);
  expect(order.kind).toBe("sell");
  expect(order.status).toBe("open");

  expect(issues).toEqual([]);
});

test("manual-network: Order Trades panel renders deterministic trades array for seeded uid", async ({
  page,
}) => {
  const issues: string[] = [];
  await routeOrderTrades(page, { issues });
  await loadConsole(page);

  await page.locator("#lookup-order-uid").fill(DEFAULT_ORDER_UID);
  await page.locator("#btn-ob-order-trades").click();

  const trades = await outputJson<JsonRecord[]>(page, "#orderbook-output");
  const expected = defaultTradesPayload();
  expect(Array.isArray(trades)).toBe(true);
  expect(trades).toHaveLength(expected.length);
  expect(trades[0].orderUid).toBe(DEFAULT_ORDER_UID);
  expectAddressEqual(trades[0].owner as string, OWNER);
  expectAddressEqual(trades[0].sellToken as string, MAINNET_WETH);
  expectAddressEqual(trades[0].buyToken as string, MAINNET_USDC);

  expect(issues).toEqual([]);
});

test("manual-network: App Data panel renders deterministic AppData document for seeded hash", async ({
  page,
}) => {
  const issues: string[] = [];
  await routeAppData(page, { issues });
  await loadConsole(page);

  await page.locator("#lookup-appdata-hex").fill(APP_DATA_HASH);
  await page.locator("#btn-ob-appdata").click();

  const response = await outputJson<JsonRecord>(page, "#orderbook-output");
  const expected = defaultAppDataPayload();
  expect(response.fullAppData).toBe(expected.fullAppData);

  const parsed = JSON.parse(response.fullAppData as string) as JsonRecord;
  expect(parsed.appCode).toBe("cow-rs/wasm-console");
  expect(parsed.environment).toBe("browser");

  expect(issues).toEqual([]);
});

test("manual-network: Subgraph Totals resolves deterministic totals response for reviewed operation", async ({
  page,
}) => {
  const issues: string[] = [];
  const captured: JsonRecord[] = [];
  await routeSubgraphQuery(page, "Totals", subgraphResponse("Totals"), {
    issues,
    captured,
  });
  await loadConsole(page);

  await page.locator("#subgraph-api-key").fill("mock-key");
  await page.locator("#btn-subgraph-totals").click();

  const totals = await outputJson<JsonRecord>(page, "#subgraph-output");
  expect(totals.tokens).toBe("2");
  expect(totals.orders).toBe("3");
  expect(totals.volumeUsd).toBe("10.25");

  expect(captured.length).toBeGreaterThan(0);
  expect(captured[0].operationName).toBe("Totals");
  expect(issues).toEqual([]);
});

test("manual-network: Subgraph custom operation resolves deterministic daily-volume response via query matcher", async ({
  page,
}) => {
  const issues: string[] = [];
  const captured: JsonRecord[] = [];
  await routeSubgraphQuery(page, "LastDaysVolume", subgraphResponse("LastDaysVolume"), {
    issues,
    captured,
  });
  await loadConsole(page);

  await page.locator("#subgraph-api-key").fill("mock-key");
  await page.locator("#btn-subgraph-days").click();

  const response = await outputJson<JsonRecord>(page, "#subgraph-output");
  const dailyTotals = response.dailyTotals as JsonRecord[] | undefined;
  if (Array.isArray(dailyTotals)) {
    expect(dailyTotals.length).toBeGreaterThan(0);
    expect(dailyTotals[0].volumeUsd).toBe("123.45");
  } else {
    // The console may unwrap the dailyTotals array directly.
    const first = (response as unknown as JsonRecord[])[0];
    expect(first.volumeUsd).toBe("123.45");
  }

  expect(captured.length).toBeGreaterThan(0);
  expect(captured[0].operationName).toBe("LastDaysVolume");
  expect(issues).toEqual([]);
});

async function outputJson<T>(page: Page, selector: string): Promise<T> {
  const output = page.locator(selector);
  await expect(output).not.toContainText("Loading");
  await expect(output).not.toContainText("Working");
  await expect(output).not.toContainText("Failed to load the WASM module");
  const text = (await output.textContent()) ?? "";
  try {
    return JSON.parse(text) as T;
  } catch (error) {
    throw new Error(`Invalid JSON in ${selector}: ${text}\n${String(error)}`);
  }
}

async function loadConsole(page: Page): Promise<void> {
  await page.goto("/");
  await expect(page.locator("#runtime-output")).toContainText('"surface": "cow-sdk"');
}

// Protocol-constant tables are emitted as lowercase hex, so address equality
// must be case-insensitive. Checksum-case constants remain as documentation.
function expectAddressEqual(actual: string, expected: string): void {
  expect(actual.toLowerCase()).toBe(expected.toLowerCase());
}
