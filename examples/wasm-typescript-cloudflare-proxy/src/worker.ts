import initialize, {
  OrderBookClient,
  supportedChainIds,
  type OrderQuoteRequestInput
} from "cow-sdk-wasm-local/cloudflare";
import wasmModule from "cow-sdk-wasm-local/cloudflare/wasm";

export interface WorkerEnv {
  COW_CHAIN_ID?: string;
  COW_ENV?: "prod" | "staging";
  COW_PARTNER_API_KEY?: string;
}

type Fetcher = (input: RequestInfo | URL, init?: RequestInit) => Promise<Response>;

export interface ProxyFetchRequest {
  method: string;
  url: string;
  headers: Record<string, string>;
  body?: string;
  timeoutMs?: number;
  signal?: AbortSignal;
}

export interface ProxyFetchResponse {
  status: number;
  statusText?: string;
  headers?: Record<string, string>;
  body?: string;
}

let ready: Promise<void> | undefined;

async function ensureReady(): Promise<void> {
  if (!ready) {
    ready = initialize(wasmModule);
  }
  await ready;
}

export async function forwardOrderbookRequest(
  request: ProxyFetchRequest,
  fetcher: Fetcher = fetch
): Promise<ProxyFetchResponse> {
  const response = await fetcher(request.url, {
    method: request.method,
    headers: request.headers,
    body: request.body,
    signal: request.signal
  });
  const headers: Record<string, string> = {};
  response.headers.forEach((value, key) => {
    headers[key] = value;
  });

  return {
    status: response.status,
    statusText: response.statusText,
    headers,
    body: await response.text()
  };
}

export function createOrderBookClient(env: WorkerEnv, fetcher: Fetcher = fetch): OrderBookClient {
  return new OrderBookClient({
    chainId: Number.parseInt(env.COW_CHAIN_ID ?? "1", 10),
    env: env.COW_ENV ?? "prod",
    apiKey: env.COW_PARTNER_API_KEY ?? null,
    transport: {
      kind: "callback",
      callback: (request: ProxyFetchRequest) => forwardOrderbookRequest(request, fetcher)
    },
    transportPolicy: {
      userAgent: "cow-sdk-wasm-cloudflare-proxy-example/0.1.0"
    }
  });
}

export default {
  async fetch(request: Request, env: WorkerEnv): Promise<Response> {
    await ensureReady();
    const url = new URL(request.url);

    if (request.method === "GET" && url.pathname === "/health") {
      return Response.json({
        ok: true,
        supportedChainIds: Array.from(supportedChainIds())
      });
    }

    if (request.method === "POST" && url.pathname === "/quote") {
      const client = createOrderBookClient(env);
      try {
        const quoteRequest = (await request.json()) as OrderQuoteRequestInput;
        const quote = await client.getQuote(quoteRequest, { timeoutMs: 8_000 });
        return Response.json(quote);
      } finally {
        client.dispose();
      }
    }

    return new Response("not found", { status: 404 });
  }
} satisfies ExportedHandler<WorkerEnv>;
