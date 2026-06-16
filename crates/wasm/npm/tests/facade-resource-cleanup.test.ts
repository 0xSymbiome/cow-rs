import { describe, expect, test } from "vitest";
import { CID, defaultFacade } from "./fixtures.js";

describe("facade resource cleanup", () => {
  test("dispose is idempotent and blocks later client calls", async () => {
    const { IpfsClient } = defaultFacade();
    const client = new IpfsClient({
      ipfsUri: "https://ipfs.example.test/ipfs",
      transport: {
        kind: "callback",
        callback: async () => ({ status: 200, headers: {}, body: "{}" })
      }
    });

    client.dispose();
    client.dispose();

    await expect(client.fetchAppDataFromCid(CID)).rejects.toMatchObject({
      schemaVersion: "v1",
      kind: "invalidInput",
      field: "client"
    });
  });

  test("[Symbol.dispose] disposes the client so `using` releases it", async () => {
    const { IpfsClient } = defaultFacade();
    const client = new IpfsClient({
      ipfsUri: "https://ipfs.example.test/ipfs",
      transport: {
        kind: "callback",
        callback: async () => ({ status: 200, headers: {}, body: "{}" })
      }
    });

    // Exercise the same path the `using` declaration invokes at scope exit.
    client[Symbol.dispose]();

    await expect(client.fetchAppDataFromCid(CID)).rejects.toMatchObject({
      schemaVersion: "v1",
      kind: "invalidInput",
      field: "client"
    });
  });
});
