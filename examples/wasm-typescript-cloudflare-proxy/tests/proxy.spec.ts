import { describe, expect, test } from "vitest";
import { forwardOrderbookRequest } from "../src/worker.js";

describe("Cloudflare proxy transport", () => {
  test("forwards SDK orderbook requests through fetch", async () => {
    const fetcher = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
      expect(String(input)).toBe("https://api.cow.fi/mainnet/api/v1/quote");
      expect(init?.method).toBe("POST");
      expect(init?.body).toBe('{"kind":"sell"}');
      expect(new Headers(init?.headers).get("content-type")).toBe("application/json");

      return new Response('{"quote":{"id":1}}', {
        status: 200,
        headers: { "content-type": "application/json" }
      });
    };

    const response = await forwardOrderbookRequest(
      {
        method: "POST",
        url: "https://api.cow.fi/mainnet/api/v1/quote",
        headers: { "content-type": "application/json" },
        body: '{"kind":"sell"}'
      },
      fetcher
    );

    expect(response.status).toBe(200);
    expect(response.headers?.["content-type"]).toContain("application/json");
    expect(response.body).toBe('{"quote":{"id":1}}');
  });
});
