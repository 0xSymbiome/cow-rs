import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

/**
 * Drift gate: the hand-written facade `CowError` `orderbook` member must stay in
 * sync with the generated wasm-bindgen error shape — same fields, same
 * optionality. The Rust `WasmError` `Orderbook` variant emits `retryable` (with
 * `#[serde(default)]`, so the generated type is optional) and `retryAfterMs`
 * (the parsed `Retry-After`, optional). The facade type and the committed
 * declaration snapshot must agree so the JavaScript retry verdict cannot
 * silently regress below the Rust core (ADR 0039 keeps the facade hand-written;
 * ADR 0060 promises the retry fields cross to JS).
 */
function orderbookErrorMember(source: string): string {
  const marker = 'kind: "orderbook";';
  const start = source.indexOf(marker);
  expect(start, "an `orderbook` error member must be declared").toBeGreaterThan(-1);
  const end = source.indexOf("}", start);
  expect(end).toBeGreaterThan(start);
  return source.slice(start, end);
}

// Classifies how `field` is declared in the member: optional (`field?:`),
// required (`field:`), or absent. The `?:` form is checked first so an optional
// field is never miscounted as required.
function optionality(member: string, field: string): "optional" | "required" | "absent" {
  if (new RegExp(`\\b${field}\\?\\s*:`).test(member)) return "optional";
  if (new RegExp(`\\b${field}\\s*:`).test(member)) return "required";
  return "absent";
}

const RETRY_FIELDS = ["retryable", "retryAfterMs"] as const;

const facade = readFileSync(
  fileURLToPath(new URL("../src/errors.ts", import.meta.url)),
  "utf8",
);
const snapshot = readFileSync(
  fileURLToPath(new URL("../../snapshots/raw/orderbook.d.ts", import.meta.url)),
  "utf8",
);

describe("orderbook error retry-field parity (facade ⇄ generated)", () => {
  const facadeMember = orderbookErrorMember(facade);
  const snapshotMember = orderbookErrorMember(snapshot);

  for (const field of RETRY_FIELDS) {
    test(`\`${field}\` is present in both and its optionality matches`, () => {
      const facadeOptionality = optionality(facadeMember, field);
      const snapshotOptionality = optionality(snapshotMember, field);
      expect(facadeOptionality, `facade orderbook member must declare \`${field}\``).not.toBe(
        "absent",
      );
      expect(
        snapshotOptionality,
        `generated snapshot orderbook member must keep \`${field}\``,
      ).not.toBe("absent");
      expect(
        facadeOptionality,
        `facade \`${field}\` (${facadeOptionality}) must match the generated shape (${snapshotOptionality})`,
      ).toBe(snapshotOptionality);
    });
  }
});
