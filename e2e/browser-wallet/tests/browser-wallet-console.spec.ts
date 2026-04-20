import { expect, test, type Page } from "@playwright/test";

import {
  ORDER_UID,
  WRAPPED_NATIVE,
  BUY_TOKEN,
  createApiCapture,
  routeBrowserWalletOrderbook,
} from "../fixtures/cow-api";
import {
  installInjectedWalletFixtures,
  type InjectedWalletFixtureSet,
} from "../fixtures/injected-wallet";

const PRIMARY_ACCOUNT = "0x4444444444444444444444444444444444444444";
const SECONDARY_ACCOUNT = "0x5555555555555555555555555555555555555555";
const SEPOLIA_CHAIN_ID = 11155111;
const SEPOLIA_CHAIN_HEX = "0xaa36a7";

const MULTI_WALLET_FIXTURE: InjectedWalletFixtureSet = {
  wallets: [
    {
      label: "MetaMask",
      uuid: "wallet-metamask",
      rdns: "io.metamask",
      icon: "data:text/plain,metamask",
      accounts: [PRIMARY_ACCOUNT],
      chainId: SEPOLIA_CHAIN_HEX,
      isMetaMask: true,
    },
    {
      label: "Rabby",
      uuid: "wallet-rabby",
      rdns: "io.rabby",
      icon: "data:text/plain,rabby",
      accounts: [SECONDARY_ACCOUNT],
      chainId: SEPOLIA_CHAIN_HEX,
      isRabby: true,
    },
  ],
};

const REJECT_TYPED_DATA_FIXTURE: InjectedWalletFixtureSet = {
  wallets: [
    {
      label: "MetaMask",
      uuid: "wallet-metamask",
      rdns: "io.metamask",
      icon: "data:text/plain,metamask",
      accounts: [PRIMARY_ACCOUNT],
      chainId: SEPOLIA_CHAIN_HEX,
      isMetaMask: true,
      failures: {
        eth_signTypedData_v4: {
          code: 4001,
          message: "User rejected typed-data signature",
        },
      },
    },
  ],
};

type ContractState = {
  lastAction: string;
  lastStatus: string;
  errorText: string;
  walletCount: number;
  walletLabels: string[];
  requiresSelection: boolean;
  confirmedWallet: string | null;
  confirmedIndex: number | null;
  selectedWallet: string | null;
  selectedIndex: number | null;
  sessionConnected: boolean;
  sessionAccount: string | null;
  sessionChain: number | null;
  connectionSource: string | null;
  resetRetained: boolean | null;
  selectionCleared: boolean | null;
  signatureStatus: string;
  quoteStatus: string;
  quoteId: number | null;
  orderUid: string | null;
  cancelStatus: string;
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

test("multi-wallet selection, reset, reconnect, and forget remain explicit", async ({ page }) => {
  await installInjectedWalletFixtures(page, MULTI_WALLET_FIXTURE);
  await loadConsole(page);

  let state = await contractState(page);
  expect(state.walletCount).toBe(2);
  expect(state.walletLabels).toEqual(["MetaMask", "Rabby"]);
  expect(state.requiresSelection).toBe(true);
  expect(state.confirmedWallet).toBeNull();
  expect(state.selectedWallet).toBeNull();

  await page.locator("#wallet-selection").fill("1");
  await page.locator("#confirm-wallet").click();
  await expectInjectedState(page, "confirm-wallet", "success");

  state = await contractState(page);
  expect(state.confirmedIndex).toBe(1);
  expect(state.confirmedWallet).toBe("Rabby");

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");

  state = await contractState(page);
  expect(state.selectedIndex).toBe(1);
  expect(state.selectedWallet).toBe("Rabby");
  expect(state.sessionConnected).toBe(true);
  expect(state.sessionAccount).toBe(SECONDARY_ACCOUNT);
  expect(state.sessionChain).toBe(SEPOLIA_CHAIN_ID);
  expect(state.connectionSource).toBe("cachedDetection");

  await page.locator("#reset-wallet").click();
  await expectInjectedState(page, "reset-wallet", "success");

  state = await contractState(page);
  expect(state.resetRetained).toBe(true);
  expect(state.selectedWallet).toBe("Rabby");
  expect(state.confirmedWallet).toBe("Rabby");
  expect(state.sessionConnected).toBe(false);

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");

  state = await contractState(page);
  expect(state.connectionSource).toBe("selectedWallet");
  expect(state.sessionConnected).toBe(true);
  expect(state.selectedWallet).toBe("Rabby");

  await expect(page.locator("#forget-wallet")).toHaveText("Clear Console Wallet");
  await page.locator("#forget-wallet").click();
  await expectInjectedState(page, "forget-wallet", "success");
  await expect(page.locator("#injected-output")).toContainText(
    "wallet authorization remains managed by the extension",
  );

  state = await contractState(page);
  expect(state.selectionCleared).toBe(true);
  expect(state.selectedWallet).toBeNull();
  expect(state.selectedIndex).toBeNull();
  expect(state.confirmedWallet).toBeNull();
  expect(state.confirmedIndex).toBeNull();
  expect(state.sessionConnected).toBe(false);
});

test("typed-data signing and route-mocked quote-submit-cancel stay deterministic", async ({
  page,
}) => {
  const capture = createApiCapture();
  await installInjectedWalletFixtures(page, MULTI_WALLET_FIXTURE);
  await routeBrowserWalletOrderbook(page, capture);
  await loadConsole(page);
  await connectRabby(page);

  await page.locator("#sign-order").click();
  await expectInjectedState(page, "sign-order", "success");

  let state = await contractState(page);
  expect(state.signatureStatus).toBe("present");
  expect(state.selectedWallet).toBe("Rabby");

  await page.locator("#live-quote").click();
  await expectInjectedState(page, "live-quote", "success");

  state = await contractState(page);
  expect(state.quoteStatus).toBe("verified");
  expect(state.quoteId).toBe(81);

  await page.locator("#submit-order").click();
  await expectInjectedState(page, "submit-order", "success");

  state = await contractState(page);
  expect(state.orderUid).toBe(ORDER_UID);
  await expect(page.locator("#order-uid")).toHaveValue(ORDER_UID);

  await page.locator("#cancel-order").click();
  await expectInjectedState(page, "cancel-order", "success");

  state = await contractState(page);
  expect(state.cancelStatus).toBe("accepted");

  expect(capture.issues).toEqual([]);
  expect(capture.quoteBodies).toHaveLength(2);
  expect(capture.appDataBodies).toHaveLength(1);
  expect(capture.orderBodies).toHaveLength(1);
  expect(capture.cancelBodies).toHaveLength(1);
  expectAddressEqual(capture.quoteBodies[0]?.sellToken as string, WRAPPED_NATIVE);
  expectAddressEqual(capture.quoteBodies[0]?.buyToken as string, BUY_TOKEN);
  expectAddressEqual(capture.orderBodies[0]?.sellToken as string, WRAPPED_NATIVE);
  expectAddressEqual(capture.orderBodies[0]?.buyToken as string, BUY_TOKEN);
});

test("rejected typed-data signing stays visible on the DOM contract surface", async ({ page }) => {
  await installInjectedWalletFixtures(page, REJECT_TYPED_DATA_FIXTURE);
  await loadConsole(page);

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");

  await page.locator("#sign-order").click();
  await expectInjectedState(page, "sign-order", "error");

  const state = await contractState(page);
  expect(state.errorText).toContain("User rejected typed-data signature");
  expect(state.signatureStatus).toBe("error");
  expect(state.selectedWallet).toBe("MetaMask");
  expect(state.sessionConnected).toBe(true);
});

async function loadConsole(page: Page): Promise<void> {
  await page.goto("/");
  await expect(page.locator("[data-testid='injected-last-action']")).toHaveText("bootstrap");
  await expect(page.locator("[data-testid='injected-last-status']")).toHaveText("success");
}

async function connectRabby(page: Page): Promise<void> {
  await page.locator("#wallet-selection").fill("1");
  await page.locator("#confirm-wallet").click();
  await expectInjectedState(page, "confirm-wallet", "success");
  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");
}

async function expectInjectedState(
  page: Page,
  actionName: string,
  status: "success" | "error",
): Promise<void> {
  await expect(page.locator("[data-testid='injected-last-action']")).toHaveText(actionName);
  await expect(page.locator("[data-testid='injected-last-status']")).toHaveText(status);
}

async function contractState(page: Page): Promise<ContractState> {
  const text = await page.locator("[data-testid='injected-contract-state']").textContent();
  if (!text) {
    throw new Error("injected contract state was empty");
  }
  return JSON.parse(text) as ContractState;
}

// Protocol-constant tables are emitted as lowercase hex, so address equality
// must be case-insensitive. Checksum-case constants remain as documentation.
function expectAddressEqual(actual: string, expected: string): void {
  expect(actual.toLowerCase()).toBe(expected.toLowerCase());
}
