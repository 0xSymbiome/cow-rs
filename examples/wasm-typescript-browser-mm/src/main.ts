import {
  signOrderWithTypedDataSigner,
  type OrderInput,
  type SignedOrderDto,
  type WasmEnvelope
} from "cow-sdk-wasm-local";

const ORDER: OrderInput = {
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
};

type EthereumProvider = {
  request(args: { method: string; params?: unknown[] }): Promise<unknown>;
};

declare global {
  interface Window {
    ethereum?: EthereumProvider;
    __cowSdkWasmMetaMaskExample?: WasmEnvelope<SignedOrderDto>;
  }
}

export async function connectAndSign(
  provider: EthereumProvider = window.ethereum as EthereumProvider
): Promise<WasmEnvelope<SignedOrderDto>> {
  if (!provider) {
    throw new Error("window.ethereum is unavailable; install or unlock MetaMask");
  }

  const accounts = await provider.request({ method: "eth_requestAccounts" });
  const [owner] = Array.isArray(accounts) ? accounts : [];
  if (typeof owner !== "string") {
    throw new Error("MetaMask did not return an owner account");
  }

  const signed = await signOrderWithTypedDataSigner(
    ORDER,
    1,
    owner,
    async (envelope) => {
      const signature = await provider.request({
        method: "eth_signTypedData_v4",
        params: [owner, JSON.stringify(envelope)]
      });
      if (typeof signature !== "string") {
        throw new Error("MetaMask did not return a signature");
      }
      return signature;
    },
    { walletConfig: { timeoutMs: 20_000 } }
  );

  window.__cowSdkWasmMetaMaskExample = signed;
  return signed;
}

const status = document.querySelector<HTMLPreElement>("#status");
const button = document.querySelector<HTMLButtonElement>("#connect");

button?.addEventListener("click", () => {
  status!.textContent = "signing";
  void connectAndSign()
    .then((signed) => {
      status!.textContent = JSON.stringify(
        {
          owner: signed.value.from,
          scheme: signed.value.signingScheme,
          signature: signed.value.signature
        },
        null,
        2
      );
    })
    .catch((error: unknown) => {
      status!.textContent = error instanceof Error ? error.message : String(error);
    });
});
