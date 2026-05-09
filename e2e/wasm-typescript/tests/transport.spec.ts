import {
  HttpToIpfsAdapter,
  IpfsClientWithFetch,
  registerFetchCallback
} from "cow-sdk-wasm-test-package/nodejs";
import { describe, expect, test } from "vitest";
import { CID } from "./orderbook.spec.js";

const APP_DATA_CONTENT = '{"appCode":"CoW Swap","metadata":{},"version":"0.7.0"}';
const HASH = "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";

describe("callback HTTP transport", () => {
  test("fetches app-data by CID through adapter callback", async () => {
    const adapter = new HttpToIpfsAdapter((request: any) => {
      expect(request.url).toBe(`https://ipfs.example.test/ipfs/${CID}`);
      expect(request.signal).toBeInstanceOf(AbortSignal);
      return { status: 200, headers: {}, body: APP_DATA_CONTENT };
    }, 500);

    const result = await adapter.fetchAppDataFromCid(CID, "https://ipfs.example.test/ipfs");
    expect(result.document.appCode).toBe("CoW Swap");
  });

  test("fetches app-data by hash through adapter callback", async () => {
    const adapter = new HttpToIpfsAdapter(() => ({
      status: 200,
      headers: {},
      body: APP_DATA_CONTENT
    }), 500);

    const result = await adapter.fetchAppDataFromHex(HASH, "https://ipfs.example.test/ipfs");
    expect(result.document.version).toBe("0.7.0");
  });

  test("shares a registered fetch callback handle", async () => {
    const handle = registerFetchCallback(() => ({
      status: 200,
      headers: {},
      body: APP_DATA_CONTENT
    }));
    const client = IpfsClientWithFetch.fromHandle("https://ipfs.example.test/ipfs", 500, handle.id);
    const result = await client.fetchAppDataFromCid(CID);

    expect(result.document.appCode).toBe("CoW Swap");
    handle.dispose();
  });

  test("maps callback HTTP status failures to typed errors", async () => {
    const adapter = new HttpToIpfsAdapter(() => ({
      status: 404,
      headers: {},
      body: "not found"
    }), 500);

    await expect(adapter.fetchAppDataFromCid(CID, "https://ipfs.example.test/ipfs")).rejects.toMatchObject({
      kind: "appData"
    });
  });
});
