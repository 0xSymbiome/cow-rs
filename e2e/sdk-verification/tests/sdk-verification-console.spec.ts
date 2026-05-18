import { expect, test, type Page, type Route } from "@playwright/test";

import {
  MAINNET_USDC,
  MAINNET_WETH,
  ORDERBOOK_QUOTE_URL,
  ORDERBOOK_VERSION_URL,
  SUBGRAPH_URL_GLOB,
  emptyTotalsSubgraphResponse,
  fulfillJson,
  fulfillPreflight,
  malformedQuoteResponse,
  quoteResponse,
  subgraphResponse,
  validateQuoteRequestShape,
  validateSubgraphRequestShape,
  type JsonRecord,
} from "../fixtures/cow-api";

let browserErrors: string[];
let allowedBrowserErrors: RegExp[];

interface RuntimeOutput {
  surface: string;
  mode: string;
  chainId: number;
  sdkConstructed: boolean;
  wrappedNative: {
    address: string;
  };
  sampleOrder: {
    sellToken: string;
    buyToken: string;
  };
  sampleOrderNotes: {
    buyToken: string;
  };
}

interface TradingDefaultsOutput {
  defaultSlippageBps: number;
  maxSlippageBps: number;
}

interface OrderbookVersionOutput {
  version: string;
}

interface OrderbookQuoteOutput {
  verified: boolean;
  quote: {
    sellToken: string;
    buyToken: string;
  };
}

interface TradingQuoteOutput {
  quoteResults: {
    quoteResponse: {
      verified: boolean;
    };
  };
  derived: {
    limitTradeParameters: {
      sellToken: string;
    };
  };
}

interface TotalsOutput {
  tokens: string;
  volumeUsd: string;
}

test.beforeEach(async ({ page }) => {
  browserErrors = [];
  allowedBrowserErrors = [];
  page.on("pageerror", (error) => browserErrors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      browserErrors.push(message.text());
    }
  });
});

test.afterEach(() => {
  const unexpected = browserErrors.filter(
    (error) => !allowedBrowserErrors.some((allowed) => allowed.test(error)),
  );
  expect(unexpected).toEqual([]);
});

test("property-0 loads the WASM bundle and runs deterministic exports", async ({ page }) => {
  await loadConsole(page);

  const runtime = await outputJson<RuntimeOutput>(page, "#runtime-output");
  expect(runtime.surface).toBe("cow-sdk");
  expect(runtime.mode).toBe("wasm-console");
  expect(runtime.chainId).toBe(1);
  expect(runtime.sdkConstructed).toBe(true);
  expectAddressEqual(runtime.wrappedNative.address, MAINNET_WETH);
  expectAddressEqual(runtime.sampleOrder.sellToken, MAINNET_WETH);
  expectAddressEqual(runtime.sampleOrder.buyToken, MAINNET_USDC);
  expect(runtime.sampleOrderNotes.buyToken).toContain("Static USDC");

  await page.locator("#btn-chains").click();
  const chains = await outputJson<
    Array<{
      chainId: number;
      apiPath: string;
      wrappedNative: { address: string };
    }>
  >(page, "#runtime-output");
  const mainnet = chains.find((entry) => entry.chainId === 1);
  expect(mainnet).toBeDefined();
  expect(mainnet?.apiPath).toBe("mainnet");
  expectAddressEqual(mainnet!.wrappedNative.address, MAINNET_WETH);

  await page.locator("#btn-trading-defaults").click();
  const defaults = await outputJson<TradingDefaultsOutput>(page, "#trading-output");
  expect(defaults.defaultSlippageBps).toBe(50);
  expect(defaults.maxSlippageBps).toBe(10000);
});

test("route-mocked orderbook and subgraph flows return reviewable JSON", async ({ page }) => {
  const requestIssues: string[] = [];
  const quoteRequests: JsonRecord[] = [];
  await routeOrderbookVersion(page, requestIssues);
  await routeOrderbookQuote(page, requestIssues, quoteRequests, "success");
  await routeSubgraph(page, requestIssues, "success");

  await loadConsole(page);

  await page.locator("#btn-ob-version").click();
  const version = await outputJson<OrderbookVersionOutput>(page, "#orderbook-output");
  expect(version.version).toBe("mock-orderbook-version");

  await page.locator("#btn-ob-quote").click();
  const quote = await outputJson<OrderbookQuoteOutput>(page, "#orderbook-output");
  expect(quote.verified).toBe(true);
  expectAddressEqual(quote.quote.sellToken, MAINNET_WETH);
  expectAddressEqual(quote.quote.buyToken, MAINNET_USDC);

  await page.locator("#btn-trading-quote").click();
  const trading = await outputJson<TradingQuoteOutput>(page, "#trading-output");
  expect(trading.quoteResults.quoteResponse.verified).toBe(true);
  expectAddressEqual(trading.derived.limitTradeParameters.sellToken, MAINNET_WETH);
  expect(quoteRequests).toHaveLength(2);

  await page.locator("#subgraph-api-key").fill("mock-key");
  await page.locator("#btn-subgraph-totals").click();
  const totals = await outputJson<TotalsOutput>(page, "#subgraph-output");
  expect(totals.tokens).toBe("2");
  expect(totals.volumeUsd).toBe("10.25");

  expect(requestIssues).toEqual([]);
});

test("malformed orderbook responses surface as classified diagnostic labels", async ({ page }) => {
  const requestIssues: string[] = [];
  const quoteRequests: JsonRecord[] = [];
  await routeOrderbookQuote(page, requestIssues, quoteRequests, "malformed");

  await loadConsole(page);
  await page.locator("#btn-ob-quote").click();

  await expect(page.locator("#orderbook-output")).toContainText("missing field");
  const errorLabel = page.locator("#orderbook-output [data-testid='error-label']");
  await expect(errorLabel).toBeVisible();
  await expect(errorLabel).toHaveAttribute("data-code", "MALFORMED-JSON");
  await expect(errorLabel).toContainText("Malformed JSON payload");
  expect(requestIssues).toEqual([]);
  expect(quoteRequests).toHaveLength(1);
});

test("orderbook network failures surface as visible errors", async ({ page }) => {
  const requestIssues: string[] = [];
  await routeOrderbookVersion(page, requestIssues, "network-error");

  await loadConsole(page);
  await page.locator("#btn-ob-version").click();

  await expect(page.locator("#orderbook-output")).toContainText("Error", { timeout: 20_000 });
  await expect(page.locator("#orderbook-output")).toContainText("transport error");
  // The SDK redacts the upstream fetch error message by construction --
  // untrusted upstream text may carry URLs, hostnames, or credentials --
  // so the user-visible detail is the sentinel rather than the raw cause.
  await expect(page.locator("#orderbook-output")).toContainText("[redacted]");
  await expect(page.locator("#orderbook-output")).not.toContainText("mock network failure");
  expect(requestIssues).toEqual([]);
});

test("malformed subgraph responses surface as visible errors", async ({ page }) => {
  const requestIssues: string[] = [];
  await routeSubgraph(page, requestIssues, "empty-totals");

  await loadConsole(page);
  await page.locator("#subgraph-api-key").fill("mock-key");
  await page.locator("#btn-subgraph-totals").click();

  await expect(page.locator("#subgraph-output")).toContainText("Error");
  await expect(page.locator("#subgraph-output")).toContainText("No totals found");
  expect(requestIssues).toEqual([]);
});

test("csp blocks off-allowlist scripts and connections", async ({ page }) => {
  allowedBrowserErrors = [/Content Security Policy/i, /Refused to/i];
  let scriptRequested = false;
  let connectRequested = false;

  await page.route("https://example.invalid/csp-probe.js", async (route) => {
    scriptRequested = true;
    await route.fulfill({
      status: 200,
      contentType: "application/javascript",
      body: "globalThis.__cspScriptRan = true;",
    });
  });
  await page.route("https://example.invalid/csp-connect", async (route) => {
    connectRequested = true;
    await route.fulfill({ status: 200, body: "allowed" });
  });

  await loadConsole(page);

  const scriptOutcome = await page.evaluate(async () => {
    return await new Promise<string>((resolve) => {
      const script = document.createElement("script");
      script.src = "https://example.invalid/csp-probe.js";
      script.onload = () => resolve("loaded");
      script.onerror = () => resolve("blocked");
      document.head.appendChild(script);
      window.setTimeout(() => resolve("timeout"), 1000);
    });
  });
  const connectOutcome = await page.evaluate(async () => {
    try {
      await fetch("https://example.invalid/csp-connect");
      return "loaded";
    } catch {
      return "blocked";
    }
  });

  expect(scriptOutcome).not.toBe("loaded");
  expect(connectOutcome).toBe("blocked");
  expect(scriptRequested).toBe(false);
  expect(connectRequested).toBe(false);
  await expect
    .poll(async () =>
      page.evaluate(() => Boolean((globalThis as { __cspScriptRan?: boolean }).__cspScriptRan)),
    )
    .toBe(false);
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

async function routeOrderbookVersion(
  page: Page,
  issues: string[],
  mode: "success" | "network-error" = "success",
): Promise<void> {
  if (mode === "network-error") {
    await page.addInitScript((versionUrl) => {
      const originalFetch = window.fetch.bind(window);
      window.fetch = (input, init) => {
        const requestUrl =
          typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
        if (requestUrl === versionUrl) {
          return Promise.reject(new TypeError("mock network failure"));
        }
        return originalFetch(input, init);
      };
    }, ORDERBOOK_VERSION_URL);
    return;
  }

  await page.route(ORDERBOOK_VERSION_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "GET") {
      issues.push(`version request used ${route.request().method()} instead of GET`);
    }
    await route.fulfill({
      status: 200,
      headers: {
        "access-control-allow-origin": "*",
        "content-type": "text/plain",
      },
      body: "mock-orderbook-version",
    });
  });
}

async function routeOrderbookQuote(
  page: Page,
  issues: string[],
  quoteRequests: JsonRecord[],
  mode: "success" | "malformed",
): Promise<void> {
  await page.route(ORDERBOOK_QUOTE_URL, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "POST") {
      issues.push(`quote request used ${route.request().method()} instead of POST`);
    }

    const body = requestJson(route, issues, "quote");
    issues.push(...validateQuoteRequestShape(body));
    if (body && typeof body === "object" && !Array.isArray(body)) {
      quoteRequests.push(body as JsonRecord);
    }

    await fulfillJson(
      route,
      mode === "malformed" ? malformedQuoteResponse() : quoteResponse((body ?? {}) as JsonRecord),
    );
  });
}

async function routeSubgraph(
  page: Page,
  issues: string[],
  mode: "success" | "empty-totals",
): Promise<void> {
  await page.route(SUBGRAPH_URL_GLOB, async (route) => {
    if (route.request().method() === "OPTIONS") {
      await fulfillPreflight(route);
      return;
    }
    if (route.request().method() !== "POST") {
      issues.push(`subgraph request used ${route.request().method()} instead of POST`);
    }

    const body = requestJson(route, issues, "subgraph");
    issues.push(...validateSubgraphRequestShape(body));
    const operationName =
      body && typeof body === "object" && !Array.isArray(body)
        ? String((body as JsonRecord).operationName ?? "")
        : undefined;

    await fulfillJson(
      route,
      mode === "empty-totals" ? emptyTotalsSubgraphResponse() : subgraphResponse(operationName),
    );
  });
}

function requestJson(route: Route, issues: string[], label: string): unknown {
  try {
    return route.request().postDataJSON();
  } catch (error) {
    issues.push(`${label} request body was not valid JSON: ${String(error)}`);
    return undefined;
  }
}

// Protocol-constant tables are emitted as lowercase hex, so address equality
// must be case-insensitive. Checksum-case constants remain as documentation.
function expectAddressEqual(actual: string, expected: string): void {
  expect(actual.toLowerCase()).toBe(expected.toLowerCase());
}
