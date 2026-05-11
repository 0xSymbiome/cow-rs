import { expect, test } from "@playwright/test";

const OWNER = "0x3333333333333333333333333333333333333333";
const SIGNATURE =
  "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b";

test("signs through a MetaMask-style injected provider", async ({ page }) => {
  await page.addInitScript(
    ({ owner, signature }) => {
      window.ethereum = {
        async request({ method, params }) {
          if (method === "eth_requestAccounts") {
            return [owner];
          }
          if (method === "eth_signTypedData_v4") {
            const typedData = JSON.parse(String(params?.[1]));
            if (typedData.primaryType !== "Order") {
              throw new Error("unexpected typed-data primary type");
            }
            return signature;
          }
          throw new Error(`unexpected method ${method}`);
        }
      };
    },
    { owner: OWNER, signature: SIGNATURE }
  );

  await page.goto("/");
  await page.getByRole("button", { name: "Connect and sign" }).click();
  await expect(page.locator("#status")).toContainText("eip712");

  const result = await page.evaluate(() => window.__cowSdkWasmMetaMaskExample);
  expect(result?.schemaVersion).toBe("v1");
  expect(result?.value.from).toBe(OWNER);
  expect(result?.value.signature).toBe(SIGNATURE);
});
