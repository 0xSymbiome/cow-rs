import { pathToFileURL } from "node:url";
import { createWalletClient, custom } from "viem";
import { mainnet } from "viem/chains";
import {
  signOrderWithEip1193,
  type Eip1193Request,
  type OrderInput,
  type SignedOrderDto,
  type WasmEnvelope
} from "cow-sdk-wasm-local";

export const OWNER = "0x3333333333333333333333333333333333333333";

export const ORDER: OrderInput = {
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

export const EXAMPLE_SIGNATURE =
  "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b";

type Eip1193Provider = {
  request(args: { method: string; params?: unknown[] }): Promise<unknown> | unknown;
};

export function exampleProvider(signature = EXAMPLE_SIGNATURE): Eip1193Provider {
  return {
    async request({ method, params }) {
      if (method !== "eth_signTypedData_v4") {
        throw new Error(`unsupported wallet method: ${method}`);
      }
      if (params?.[0] !== OWNER) {
        throw new Error("wallet request owner did not match the configured owner");
      }
      return signature;
    }
  };
}

export async function signWithViemWallet(
  provider: Eip1193Provider = exampleProvider()
): Promise<WasmEnvelope<SignedOrderDto>> {
  const walletClient = createWalletClient({
    account: OWNER,
    chain: mainnet,
    transport: custom(provider as Parameters<typeof custom>[0])
  });
  const requestWithViem = walletClient.request as (request: Eip1193Request) => Promise<unknown>;

  return signOrderWithEip1193(
    ORDER,
    mainnet.id,
    OWNER,
    (request) => requestWithViem(request),
    { walletConfig: { timeoutMs: 10_000 } }
  );
}

async function main(): Promise<void> {
  const signed = await signWithViemWallet();
  console.log(JSON.stringify({ owner: signed.value.from, scheme: signed.value.signingScheme }));
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main();
}
