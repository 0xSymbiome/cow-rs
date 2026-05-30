import { expect, test, type Page } from "@playwright/test";

import {
  APP_DATA_HASH,
  MAINNET_SETTLEMENT,
  MAINNET_USDC,
  MAINNET_WETH,
  OWNER,
  fulfillJson,
  fulfillPreflight,
} from "../fixtures/cow-api";

const ORDER_UID = `0x${"11".repeat(56)}`;
const SIGNATURE = `0x${"22".repeat(64)}1b`;
const ORDERBOOK_LATEST_COMPETITION_URL =
  "https://barn.api.cow.fi/mainnet/api/v2/solver_competition/latest";
const ORDERBOOK_ORDER_URL = `https://barn.api.cow.fi/mainnet/api/v1/orders/${ORDER_UID}`;
const ORDERBOOK_APP_DATA_URL = `https://barn.api.cow.fi/mainnet/api/v1/app_data/${APP_DATA_HASH}`;

type LatestCompetitionOutput = {
  auctionId: number;
  auction?: {
    orders?: string[];
  };
};

type OrderOutput = {
  uid: string;
  owner: string;
  appData: string;
};

type AppDataOutput = {
  fullAppData: string;
};

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

test("latest competition seeds orderbook lookups and invalid live actions start gated", async ({
  page,
}) => {
  const issues: string[] = [];
  await routeLatestCompetition(page, issues);
  await routeOrder(page, issues);
  await routeAppData(page, issues);

  await loadConsole(page);
  await expect(page.locator("#env")).toHaveValue("staging");
  await expect(page.locator("[data-testid='browser-live-gate']")).toHaveText(
    "Static browser-live CoW orderbook actions are enabled for staging.",
  );
  await expect(page.locator("#btn-trading-quote")).toBeEnabled();
  await expect(page.locator("#btn-ob-auction")).toBeEnabled();

  await page.selectOption("#env", "prod");
  await expect(page.locator("[data-testid='browser-live-gate']")).toContainText(
    "Production browser-live CoW orderbook calls are disabled on this static page.",
  );
  await expect(page.locator("#btn-trading-quote")).toBeDisabled();
  await expect(page.locator("#btn-ob-version")).toBeDisabled();
  await expect(page.locator("#btn-ob-quote")).toBeDisabled();
  await expect(page.locator("#btn-ob-auction")).toBeDisabled();

  await page.selectOption("#env", "staging");
  await expect(page.locator("[data-testid='browser-live-gate']")).toHaveText(
    "Static browser-live CoW orderbook actions are enabled for staging.",
  );
  await expect(page.locator("#btn-trading-quote")).toBeEnabled();
  await expect(page.locator("#btn-ob-auction")).toBeEnabled();

  await expect(page.locator("#btn-ob-order")).toBeDisabled();
  await expect(page.locator("#btn-ob-order-trades")).toBeDisabled();
  await expect(page.locator("#btn-ob-appdata")).toBeDisabled();

  await expect(page.locator("#btn-eip1271")).toBeEnabled();
  await page.locator("#eip1271-signature").fill("");
  await expect(page.locator("#btn-eip1271")).toBeDisabled();
  await page.locator("#eip1271-signature").fill(SIGNATURE);
  await expect(page.locator("#btn-eip1271")).toBeEnabled();
  await page.locator("#btn-eip1271").click();
  await expect(page.locator("#order-output")).not.toContainText("Error");

  await page.locator("#btn-ob-auction").click();
  const competition = await outputJson<LatestCompetitionOutput>(page, "#orderbook-output");
  expect(competition.auctionId).toBe(12714210);
  expect(competition.auction?.orders?.[0]).toBe(ORDER_UID);

  await expect(page.locator("#lookup-order-uid")).toHaveValue(ORDER_UID);
  await expect(page.locator("#lookup-owner")).toHaveValue(OWNER);
  await expect(page.locator("#lookup-appdata-hex")).toHaveValue(APP_DATA_HASH);
  await expect(page.locator("#btn-ob-order")).toBeEnabled();
  await expect(page.locator("#btn-ob-order-trades")).toBeEnabled();
  await expect(page.locator("#btn-ob-appdata")).toBeEnabled();

  await page.locator("#btn-ob-order").click();
  const order = await outputJson<OrderOutput>(page, "#orderbook-output");
  expect(order.uid).toBe(ORDER_UID);
  expect(order.owner).toBe(OWNER);
  expect(order.appData).toBe(APP_DATA_HASH);

  await page.locator("#btn-ob-appdata").click();
  const appData = await outputJson<AppDataOutput>(page, "#orderbook-output");
  expect(appData.fullAppData).toContain("\"metadata\"");
  expect(issues).toEqual([]);
});

async function loadConsole(page: Page): Promise<void> {
  await page.goto("/");
  await expect(page.locator("#runtime-output")).toContainText('"surface": "cow-sdk"');
}

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

async function routeLatestCompetition(page: Page, issues: string[]): Promise<void> {
  await page.route(ORDERBOOK_LATEST_COMPETITION_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      issues.push(`latest competition request used ${route.request().method()} instead of GET`);
    }
    await fulfillJson(route, {
      auctionId: 12714210,
      auctionStartBlock: 24878923,
      auction: {
        orders: [ORDER_UID],
      },
      transactionHashes: [`0x${"33".repeat(32)}`],
      solutions: [],
    });
  });
}

async function routeOrder(page: Page, issues: string[]): Promise<void> {
  await page.route(ORDERBOOK_ORDER_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      issues.push(`order request used ${route.request().method()} instead of GET`);
    }
    await fulfillJson(route, {
      sellToken: MAINNET_WETH,
      buyToken: MAINNET_USDC,
      receiver: OWNER,
      sellAmount: "1234567890",
      buyAmount: "1200000000",
      validTo: 1700000000,
      appData: APP_DATA_HASH,
      feeAmount: "1000",
      kind: "buy",
      partiallyFillable: true,
      sellTokenBalance: "erc20",
      buyTokenBalance: "erc20",
      signingScheme: "eip712",
      signature: "0x1234",
      settlementContract: MAINNET_SETTLEMENT,
      owner: OWNER,
      uid: ORDER_UID,
      creationDate: "2020-12-03T18:35:18.814523Z",
      executedSellAmount: "100",
      executedSellAmountBeforeFees: "99",
      executedBuyAmount: "90",
      executedFeeAmount: "11",
      executedFee: "9",
      invalidated: false,
      status: "open",
      class: "market",
    });
  });
}

async function routeAppData(page: Page, issues: string[]): Promise<void> {
  await page.route(ORDERBOOK_APP_DATA_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      issues.push(`app-data request used ${route.request().method()} instead of GET`);
    }
    await fulfillJson(route, {
      fullAppData: JSON.stringify({
        metadata: {
          source: "example-readiness",
        },
      }),
    });
  });
}
