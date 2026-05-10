import { IpfsClient } from "cow-sdk-wasm-test-package/nodejs";
import { describe, expect, test } from "vitest";
import { CID } from "./orderbook.spec.js";

const APP_DATA_CONTENT = '{"appCode":"CoW Swap","metadata":{},"version":"0.7.0"}';
const HASH = "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";

function ipfsClient(callback: (request: any) => any) {
  return new IpfsClient({
    ipfsUri: "https://ipfs.example.test/ipfs",
    timeoutMs: 500,
    transport: { kind: "callback", callback }
  });
}

describe("callback HTTP transport", () => {
  test("fetches app-data by CID through client callback", async () => {
    const client = ipfsClient((request: any) => {
      expect(request.url).toBe(`https://ipfs.example.test/ipfs/${CID}`);
      expect(request.signal).toBeInstanceOf(AbortSignal);
      return { status: 200, headers: {}, body: APP_DATA_CONTENT };
    });

    const result = await client.fetchAppDataFromCid(CID);
    expect(result.document.appCode).toBe("CoW Swap");
  });

  test("fetches app-data by hash through client callback", async () => {
    const client = ipfsClient(() => ({
      status: 200,
      headers: {},
      body: APP_DATA_CONTENT
    }));

    const result = await client.fetchAppDataFromHex(HASH);
    expect(result.document.version).toBe("0.7.0");
  });

  test("keeps callback registration internal to the client", async () => {
    const client = ipfsClient((request: any) => {
      expect("id" in request).toBe(false);
      return { status: 200, headers: {}, body: APP_DATA_CONTENT };
    });
    const result = await client.fetchAppDataFromCid(CID);

    expect(result.document.appCode).toBe("CoW Swap");
  });

  test("maps callback HTTP status failures to typed errors", async () => {
    const client = ipfsClient(() => ({
      status: 404,
      headers: {},
      body: "not found"
    }));

    await expect(client.fetchAppDataFromCid(CID)).rejects.toMatchObject({
      kind: "appData"
    });
  });
});
