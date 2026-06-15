import { describe, expect, test } from "vitest";
import { ORDER, OWNER, signingFacade } from "./fixtures.js";

const SIGNATURE =
  "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b";

describe("signing facade", () => {
  test("uses named typed-data callback types at the public boundary", async () => {
    const { signOrderWithTypedDataSigner } = signingFacade();
    const signed = await signOrderWithTypedDataSigner(ORDER, 1, OWNER, async (envelope: any) => {
      expect(envelope.primaryType).toBe("Order");
      return SIGNATURE;
    });

    expect(signed.schemaVersion).toBe("v1");
    expect(signed.value.signingScheme).toBe("eip712");
    expect(signed.value.from).toBe(OWNER);
  });
});
