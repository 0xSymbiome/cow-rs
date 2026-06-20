import { describe, expect, test } from "vitest";
import { CID, defaultFacade } from "./fixtures.js";

describe("facade cancellation", () => {
  test("rejects an already-aborted call before dispatch", async () => {
    const { IpfsClient } = defaultFacade();
    let called = false;
    const client = new IpfsClient({
      ipfsUri: "https://ipfs.example.test/ipfs",
      transport: {
        kind: "callback",
        callback: async () => {
          called = true;
          return { status: 200, headers: {}, body: "{}" };
        }
      }
    });
    const controller = new AbortController();
    controller.abort();

    await expect(client.fetchAppDataFromCid(CID, { signal: controller.signal })).rejects.toMatchObject({
      kind: "cancelled",
      message: expect.stringContaining("AbortController")
    });
    expect(called).toBe(false);
    client.dispose();
  });
});
