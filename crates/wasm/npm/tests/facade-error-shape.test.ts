import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

/**
 * Drift gate: the hand-written facade `CowError` `orderbook` member must stay in
 * sync with the generated wasm-bindgen error shape. The Rust `WasmError`
 * `Orderbook` variant emits `retryable` (always serialized) and `retryAfterMs`
 * (the parsed `Retry-After`, optional). The facade type and the committed
 * declaration snapshot must both surface them so the JavaScript retry verdict
 * cannot silently regress below the Rust core (ADR 0047 keeps the facade
 * hand-written; ADR 0060 promises the fields cross to JS).
 */
function orderbookErrorMember(source: string): string {
  const marker = 'kind: "orderbook"';
  const start = source.indexOf(marker);
  expect(start, "an `orderbook` error member must be declared").toBeGreaterThan(-1);
  const end = source.indexOf("}", start);
  expect(end).toBeGreaterThan(start);
  return source.slice(start, end);
}

const RETRY_FIELDS = ["retryable", "retryAfterMs"] as const;

describe("orderbook error retry-field parity (facade ⇄ generated)", () => {
  test("the hand-written facade union carries the retry verdict", () => {
    const facade = readFileSync(
      fileURLToPath(new URL("../src/errors.ts", import.meta.url)),
      "utf8",
    );
    const member = orderbookErrorMember(facade);
    for (const field of RETRY_FIELDS) {
      expect(member, `facade orderbook member must declare \`${field}\``).toContain(field);
    }
  });

  test("the generated declaration snapshot still emits the same retry fields", () => {
    const snapshot = readFileSync(
      fileURLToPath(new URL("../../snapshots/raw/orderbook.d.ts", import.meta.url)),
      "utf8",
    );
    const member = orderbookErrorMember(snapshot);
    for (const field of RETRY_FIELDS) {
      expect(member, `generated snapshot orderbook member must keep \`${field}\``).toContain(field);
    }
  });
});
