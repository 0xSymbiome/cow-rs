import type { Page } from "@playwright/test";

export interface WalletFailure {
  code: number;
  message: string;
}

export interface InjectedWalletFixture {
  label: string;
  uuid: string;
  rdns: string;
  icon?: string;
  accounts: string[];
  chainId: string;
  connected?: boolean;
  isMetaMask?: boolean;
  isCoinbaseWallet?: boolean;
  isRabby?: boolean;
  failures?: Record<string, WalletFailure>;
}

export interface InjectedWalletFixtureSet {
  wallets: InjectedWalletFixture[];
}

export async function installInjectedWalletFixtures(
  page: Page,
  fixture: InjectedWalletFixtureSet,
): Promise<void> {
  await page.addInitScript((config: InjectedWalletFixtureSet) => {
    class MockInjectedProvider {
      private readonly accounts: string[];
      private readonly label: string;
      private readonly failures: Record<string, WalletFailure>;
      private readonly listeners = new Map<string, Set<(payload: unknown) => void>>();
      private chainId: string;
      private connected: boolean;
      readonly isMetaMask: boolean;
      readonly isCoinbaseWallet: boolean;
      readonly isRabby: boolean;

      constructor(wallet: InjectedWalletFixture) {
        this.accounts = [...wallet.accounts];
        this.label = wallet.label;
        this.failures = wallet.failures ?? {};
        this.chainId = wallet.chainId;
        this.connected = wallet.connected ?? false;
        this.isMetaMask = Boolean(wallet.isMetaMask);
        this.isCoinbaseWallet = Boolean(wallet.isCoinbaseWallet);
        this.isRabby = Boolean(wallet.isRabby);
      }

      on(eventName: string, callback: (payload: unknown) => void): this {
        const callbacks = this.listeners.get(eventName) ?? new Set();
        callbacks.add(callback);
        this.listeners.set(eventName, callbacks);
        return this;
      }

      removeListener(eventName: string, callback: (payload: unknown) => void): this {
        this.listeners.get(eventName)?.delete(callback);
        return this;
      }

      emit(eventName: string, payload: unknown): void {
        for (const callback of this.listeners.get(eventName) ?? []) {
          callback(payload);
        }
      }

      async request(request: { method?: string; params?: unknown[] }): Promise<unknown> {
        const method = request?.method ?? "";
        this.maybeFail(method);

        switch (method) {
          case "eth_accounts":
            return this.connected ? [...this.accounts] : [];
          case "eth_requestAccounts":
            if (!this.connected) {
              this.connected = true;
              this.emit("connect", { chainId: this.chainId });
            }
            this.emit("accountsChanged", [...this.accounts]);
            return [...this.accounts];
          case "eth_chainId":
            return this.chainId;
          case "personal_sign":
            return signatureFor(this.label, "personal");
          case "eth_signTypedData_v4":
            return signatureFor(this.label, "typed");
          case "wallet_switchEthereumChain": {
            const nextChainId =
              typeof request.params?.[0] === "object" &&
              request.params[0] !== null &&
              typeof (request.params[0] as { chainId?: unknown }).chainId === "string"
                ? String((request.params[0] as { chainId: string }).chainId)
                : null;
            if (!nextChainId) {
              throw createWalletError(-32602, "wallet_switchEthereumChain requires a chainId");
            }
            this.chainId = nextChainId;
            this.emit("chainChanged", this.chainId);
            return null;
          }
          case "wallet_addEthereumChain": {
            const nextChainId =
              typeof request.params?.[0] === "object" &&
              request.params[0] !== null &&
              typeof (request.params[0] as { chainId?: unknown }).chainId === "string"
                ? String((request.params[0] as { chainId: string }).chainId)
                : null;
            if (!nextChainId) {
              throw createWalletError(-32602, "wallet_addEthereumChain requires a chainId");
            }
            this.chainId = nextChainId;
            return null;
          }
          case "web3_clientVersion":
            return `mock/${this.label.toLowerCase().replace(/\s+/g, "-")}`;
          default:
            throw createWalletError(4200, `${method} is not implemented by the deterministic fixture`);
        }
      }

      private maybeFail(method: string): void {
        const failure = this.failures[method];
        if (!failure) {
          return;
        }
        throw createWalletError(failure.code, failure.message);
      }
    }

    function createWalletError(code: number, message: string): Error & { code: number } {
      const error = new Error(message) as Error & { code: number };
      error.code = code;
      return error;
    }

    function signatureFor(label: string, kind: string): string {
      const seed = `${label}:${kind}`.toLowerCase().replace(/[^a-z0-9]/g, "");
      const hex = Array.from({ length: 130 }, (_, index) => {
        const charCode = seed.charCodeAt(index % seed.length);
        return (charCode % 16).toString(16);
      }).join("");
      return `0x${hex}`;
    }

    const announcedProviders = config.wallets.map((wallet) => {
      const provider = new MockInjectedProvider(wallet);
      return {
        info: {
          name: wallet.label,
          uuid: wallet.uuid,
          rdns: wallet.rdns,
          icon: wallet.icon ?? `data:text/plain,${encodeURIComponent(wallet.label)}`,
        },
        provider,
      };
    });

    const announceProviders = () => {
      for (const candidate of announcedProviders) {
        window.dispatchEvent(
          new CustomEvent("eip6963:announceProvider", {
            detail: candidate,
          }),
        );
      }
    };

    window.addEventListener("eip6963:requestProvider", announceProviders);
    if (announcedProviders.length > 0) {
      window.ethereum = announcedProviders[0].provider;
    }
  }, fixture);
}
