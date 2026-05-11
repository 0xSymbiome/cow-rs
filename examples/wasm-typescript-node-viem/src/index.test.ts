import { describe, expect, test } from "vitest";
import { EXAMPLE_SIGNATURE, OWNER, signWithViemWallet } from "./index.js";

describe("Node viem example", () => {
  test("signs an order through the EIP-1193 request path", async () => {
    const signed = await signWithViemWallet();

    expect(signed.schemaVersion).toBe("v1");
    expect(signed.value.from).toBe(OWNER);
    expect(signed.value.signingScheme).toBe("eip712");
    expect(signed.value.signature).toBe(EXAMPLE_SIGNATURE);
  });
});
