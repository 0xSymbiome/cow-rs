import { describe, expect, test } from "vitest";
import { APP_DATA_CONTENT, CID, ORDER, OWNER, defaultFacade } from "./fixtures.js";

describe("default facade", () => {
  test("exports pure helpers through the public entry point", () => {
    const sdk = defaultFacade();
    expect(sdk.wasmVersion()).toMatch(/^\d+\.\d+\.\d+/);
    expect(Array.from(sdk.supportedChainIds())).toContain(1);
    expect(sdk.computeOrderUid(ORDER, 1, OWNER).value.orderUid).toMatch(/^0x[0-9a-f]{112}$/);
  });

  test("adapts transport fetch sugar into the callback transport ABI", async () => {
    const { IpfsClient } = defaultFacade();
    const client = new IpfsClient({
      ipfsUri: "https://ipfs.example.test/ipfs",
      transport: {
        kind: "fetch",
        fetch: async (url: RequestInfo | URL, init?: RequestInit) => {
          expect(String(url)).toBe(`https://ipfs.example.test/ipfs/${CID}`);
          expect(init?.method).toBe("GET");
          return new Response(APP_DATA_CONTENT, {
            status: 200,
            headers: { "content-type": "application/json" }
          });
        }
      }
    });

    const result = await client.fetchAppDataFromCid(CID);
    expect(result.schemaVersion).toBe("v1");
    expect(result.value.document.appCode).toBe("CoW Swap");
    client.dispose();
  });
});
