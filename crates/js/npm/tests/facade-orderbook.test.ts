import { describe, expect, test } from "vitest";
import { orderbookFacade } from "./fixtures.js";

describe("orderbook facade", () => {
  test("constructs through a single config object and exposes dispose", () => {
    const { OrderBookClient } = orderbookFacade();
    const client = new OrderBookClient({
      chainId: 1,
      transport: {
        kind: "callback",
        callback: async () => ({ status: 200, headers: {}, body: "{}" })
      }
    });

    expect(client).toBeInstanceOf(OrderBookClient);
    expect(typeof client.dispose).toBe("function");
    expect("free" in client).toBe(false);
    client.dispose();
  });
});
