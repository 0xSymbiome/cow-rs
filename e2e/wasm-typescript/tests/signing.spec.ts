import {
  computeOrderUid,
  signCancellationEthSignDigest,
  signCancellationWithEip1193,
  signCancellationWithTypedDataSigner,
  signOrderEthSignDigest,
  signOrderWithEip1193,
  signOrderWithTypedDataSigner
} from "cow-sdk-wasm-test-package";
import { describe, expect, test } from "vitest";
import { ORDER, OWNER } from "./orderbook.spec.js";

const SIGNATURE =
  "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b";

function orderUid() {
  return computeOrderUid(ORDER, 1, OWNER).value.orderUid;
}

describe("callback signing", () => {
  test("signs an order through typed data callback", async () => {
    const signed = await signOrderWithTypedDataSigner(ORDER, 1, OWNER, async (envelope: any) => {
      expect(envelope.primaryType).toBe("Order");
      return SIGNATURE;
    });

    expect(signed.schemaVersion).toBe("v1");
    expect(signed.value.signingScheme).toBe("eip712");
    expect(signed.value.from).toBe(OWNER);
  });

  test("signs an order through EIP-1193 callback", async () => {
    const signed = await signOrderWithEip1193(ORDER, 1, OWNER, async (request: any) => {
      expect(request.method).toBe("eth_signTypedData_v4");
      expect(request.params?.[0]).toBe(OWNER);
      return SIGNATURE;
    });

    expect(signed.value.signingScheme).toBe("eip712");
  });

  test("signs an order digest with eth_sign scheme", async () => {
    const signed = await signOrderEthSignDigest(ORDER, 1, OWNER, async (digest: string) => {
      expect(digest).toMatch(/^0x[0-9a-f]{64}$/);
      return SIGNATURE;
    });

    expect(signed.value.signingScheme).toBe("ethsign");
  });

  test("signs typed-data cancellations", async () => {
    const uid = orderUid();
    const signed = await signCancellationWithTypedDataSigner([uid], 1, async (envelope: any) => {
      expect(envelope.primaryType).toBe("OrderCancellations");
      return SIGNATURE;
    });

    expect(signed.value.orderUids).toEqual([uid]);
    expect(signed.value.signingScheme).toBe("eip712");
  });

  test("signs EIP-1193 cancellations", async () => {
    const uid = orderUid();
    const signed = await signCancellationWithEip1193([uid], 1, OWNER, async (request: any) => {
      expect(request.method).toBe("eth_signTypedData_v4");
      return SIGNATURE;
    });

    expect(signed.value.orderUids).toEqual([uid]);
  });

  test("signs digest cancellations", async () => {
    const uid = orderUid();
    const signed = await signCancellationEthSignDigest([uid], 1, async (digest: string) => {
      expect(digest).toMatch(/^0x[0-9a-f]{64}$/);
      return SIGNATURE;
    });

    expect(signed.value.signingScheme).toBe("ethsign");
  });
});
