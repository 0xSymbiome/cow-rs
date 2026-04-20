import { expect, test, type Page } from "@playwright/test";

import { createApiCapture, routeBrowserWalletOrderbook } from "../fixtures/cow-api";
import {
  installInjectedWalletFixtures,
  type InjectedWalletFixtureSet,
} from "../fixtures/injected-wallet";

const PRIMARY_ACCOUNT = "0x4444444444444444444444444444444444444444";
const MAINNET_CHAIN_ID = 1;
const SEPOLIA_CHAIN_ID = 11155111;
const MAINNET_CHAIN_HEX = "0x1";

const SINGLE_WALLET_MAINNET_FIXTURE: InjectedWalletFixtureSet = {
  wallets: [
    {
      label: "MetaMask",
      uuid: "wallet-metamask",
      rdns: "io.metamask",
      icon: "data:text/plain,metamask",
      accounts: [PRIMARY_ACCOUNT],
      chainId: MAINNET_CHAIN_HEX,
      isMetaMask: true,
    },
  ],
};

const SINGLE_WALLET_REJECT_SIGN_FIXTURE: InjectedWalletFixtureSet = {
  wallets: [
    {
      label: "MetaMask",
      uuid: "wallet-metamask",
      rdns: "io.metamask",
      icon: "data:text/plain,metamask",
      accounts: [PRIMARY_ACCOUNT],
      chainId: "0xaa36a7", // Sepolia
      isMetaMask: true,
      failures: {
        eth_signTypedData_v4: {
          code: 4001,
          message: "User rejected typed-data signature",
        },
        personal_sign: {
          code: 4001,
          message: "User rejected message signature",
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

test("single-wallet connect auto-confirms and blocks live quote until chain matches", async ({
  page,
}) => {
  const capture = createApiCapture();
  await installInjectedWalletFixtures(page, SINGLE_WALLET_MAINNET_FIXTURE);
  await routeBrowserWalletOrderbook(page, capture);
  await loadConsole(page);
  await expect(page.locator("#env")).toHaveValue("staging");

  let state = await contractState(page);
  expect(state.walletCount).toBe(1);
  expect(state.walletLabels).toEqual(["MetaMask"]);
  expect(state.requiresSelection).toBe(false);
  expect(state.confirmedIndex).toBe(0);
  expect(state.confirmedWallet).toBe("MetaMask");
  await expect(page.locator("#confirm-wallet")).toBeDisabled();
  await expect(page.locator("[data-testid='injected-live-gate']")).toHaveText(
    "Connect the wallet before order signing or live orderbook actions.",
  );

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");

  state = await contractState(page);
  expect(state.selectedIndex).toBe(0);
  expect(state.selectedWallet).toBe("MetaMask");
  expect(state.sessionConnected).toBe(true);
  expect(state.sessionAccount).toBe(PRIMARY_ACCOUNT);
  expect(state.sessionChain).toBe(MAINNET_CHAIN_ID);

  await expect(page.locator("#live-quote")).toBeDisabled();
  await expect(page.locator("[data-testid='injected-live-gate']")).toContainText(
    `Wallet session chain ${MAINNET_CHAIN_ID} does not match selected console chain ${SEPOLIA_CHAIN_ID}.`,
  );

  await page.locator("#switch-wallet").click();
  await expectInjectedState(page, "switch-wallet", "success");

  state = await contractState(page);
  expect(state.sessionChain).toBe(SEPOLIA_CHAIN_ID);
  await expect(page.locator("#live-quote")).toBeEnabled();

  await page.locator("#live-quote").click();
  await expectInjectedState(page, "live-quote", "success");

  state = await contractState(page);
  expect(state.quoteStatus).toBe("verified");
  expect(state.quoteId).toBe(81);
  expect(capture.issues).toEqual([]);
  expect(capture.quoteBodies).toHaveLength(1);
});

test("prod keeps static live orderbook actions gated while local signing stays available", async ({
  page,
}) => {
  await installInjectedWalletFixtures(page, SINGLE_WALLET_MAINNET_FIXTURE);
  await loadConsole(page);

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");
  await page.locator("#switch-wallet").click();
  await expectInjectedState(page, "switch-wallet", "success");

  await page.selectOption("#env", "prod");
  await expect(page.locator("#sign-order")).toBeEnabled();
  await expect(page.locator("#live-quote")).toBeDisabled();
  await expect(page.locator("#submit-order")).toBeDisabled();
  await expect(page.locator("#cancel-order")).toBeDisabled();
  await expect(page.locator("[data-testid='injected-live-gate']")).toContainText(
    "Static browser-live orderbook actions are enabled only for staging on this page.",
  );
});

test("typed-data rejection renders the classified EIP-1193 4001 label", async ({ page }) => {
  await installInjectedWalletFixtures(page, SINGLE_WALLET_REJECT_SIGN_FIXTURE);
  await loadConsole(page);

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");

  // The wallet session reports Sepolia so the chain-coherence guard allows
  // sign-order against the default console chain. The fixture then rejects
  // with a 4001 on eth_signTypedData_v4.
  await page.locator("#sign-order").click();
  await expectInjectedState(page, "sign-order", "error");

  const errorLabel = page.locator("#injected-output [data-testid='error-label']");
  await expect(errorLabel).toBeVisible();
  await expect(errorLabel).toContainText("EIP-1193 4001");
  await expect(errorLabel).toContainText("Request rejected by user");
  await expect(errorLabel).toHaveAttribute("data-code", "EIP-1193 4001");
});

test("chain-bound signer surfaces the classified chain-mismatch label before live actions", async ({
  page,
}) => {
  await installInjectedWalletFixtures(page, SINGLE_WALLET_MAINNET_FIXTURE);
  await loadConsole(page);

  await page.locator("#connect-wallet").click();
  await expectInjectedState(page, "connect-wallet", "success");

  // Wallet session is on mainnet; console chain is Sepolia. The console
  // disables the sign-order button before the click can fire and surfaces
  // the chain-mismatch explanation in the button's title so downstream
  // clicks never reach the chain-bound signer. The guard is reviewed under
  // the chain-coherence contract and must fail closed without producing a
  // signature.
  const signOrderButton = page.locator("#sign-order");
  await expect(signOrderButton).toBeDisabled();
  await expect(signOrderButton).toHaveAttribute(
    "title",
    /Wallet session chain .*does not match selected console chain .*Use Switch Chain before live actions\./,
  );
});

async function loadConsole(page: Page): Promise<void> {
  await page.goto("/");
  await expect(page.locator("[data-testid='injected-last-action']")).toHaveText("bootstrap");
  await expect(page.locator("[data-testid='injected-last-status']")).toHaveText("success");
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
