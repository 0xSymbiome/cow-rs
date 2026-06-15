import { expect, test } from "@playwright/test";

test("browser bundle initializes and exposes deterministic helpers", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#root")).toContainText("primaryType");
  const result = await page.evaluate(() => window.__cowSdkWasmSmoke);

  expect(result?.primaryType).toBe("Order");
  expect(result?.chainIds).toContain(1);
  expect(result?.cid).toBe("f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df");
  expect(result?.hash).toBe("0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df");
  expect(result?.uid).toMatch(/^0x[0-9a-f]{112}$/);
});
