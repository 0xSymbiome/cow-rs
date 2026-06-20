/**
 * Coarse, switchable classification of an orderbook rejection. Lets a consumer
 * branch on the action a rejection calls for — fix the request, fund the
 * wallet, re-quote, wait, or escalate — without matching every wire tag. The
 * `__unknown` member is the forward-compatible sentinel for a category a newer
 * SDK may introduce.
 */
export type OrderBookRejectionCategory =
  | "authorization"
  | "insufficientFunds"
  | "invalidOrder"
  | "notFound"
  | "conflict"
  | "unfulfillable"
  | "server"
  | "__unknown";

/**
 * The services `errorType` wire tag for a specific orderbook rejection, carried
 * by an orderbook error. Branch on this where
 * {@link OrderBookRejectionCategory} is too coarse — e.g. to tell
 * `"InsufficientAllowance"` (needs a token approval) from `"InsufficientBalance"`
 * (needs funds). The listed tags mirror the services error schemas for
 * autocomplete; the union stays open, so a tag a newer services release
 * introduces is still a valid value rather than a type error.
 */
export type OrderBookErrorType =
  | "DuplicatedOrder"
  | "OldOrderActivelyBidOn"
  | "QuoteNotFound"
  | "QuoteNotVerified"
  | "InvalidQuote"
  | "MissingFrom"
  | "WrongOwner"
  | "InvalidEip1271Signature"
  | "InvalidSignature"
  | "IncompatibleSigningScheme"
  | "InsufficientBalance"
  | "InsufficientAllowance"
  | "ZeroAmount"
  | "NonZeroFee"
  | "SellAmountOverflow"
  | "TooMuchGas"
  | "TooManyLimitOrders"
  | "TransferSimulationFailed"
  | "InsufficientValidTo"
  | "ExcessiveValidTo"
  | "InvalidNativeSellToken"
  | "SameBuyAndSellToken"
  | "UnsupportedToken"
  | "UnsupportedBuyTokenDestination"
  | "UnsupportedSellTokenSource"
  | "UnsupportedOrderType"
  | "AppDataInvalid"
  | "InvalidAppData"
  | "AppDataHashMismatch"
  | "AppDataMismatch"
  | "AppdataFromMismatch"
  | "MetadataSerializationFailed"
  | "NoLiquidity"
  | "TradingOutsideAllowedWindow"
  | "TokenTemporarilySuspended"
  | "InsufficientLiquidity"
  | "CustomSolverError"
  | "InvalidTradeFilter"
  | "InvalidLimit"
  | "LIMIT_OUT_OF_BOUNDS"
  | "SellAmountDoesNotCoverFee"
  | "AlreadyCancelled"
  | "OrderFullyExecuted"
  | "OrderExpired"
  | "OrderNotFound"
  | "NotFound"
  | "OnChainOrder"
  | "Forbidden"
  | "InternalServerError"
  // eslint-disable-next-line @typescript-eslint/ban-types
  | (string & {});

/**
 * The field shapes a {@link CowError} can carry, discriminated by `kind`. Every
 * thrown {@link CowError} exposes these as enumerable own properties, and
 * {@link CowError.toJSON} returns exactly this shape. The `__unknown` member is
 * the forward-compatible sentinel: it preserves the unrecognised value in `raw`
 * so a caught error is never silently dropped.
 */
export type CowErrorData =
  | { kind: "invalidInput"; message: string; field?: string }
  | { kind: "unknownEnumValue"; message: string; field: string; value: string }
  | { kind: "unsupportedChain"; message: string; chainId: number }
  | { kind: "walletRequest"; method: string; code?: number; message: string }
  | { kind: "walletTimeout"; message: string; timeoutMs: number }
  | {
      kind: "transport";
      class: string;
      message: string;
      status?: number;
      headers?: [string, string][];
      body?: string;
    }
  | {
      kind: "orderbook";
      code?: string;
      category?: OrderBookRejectionCategory;
      // The services `errorType` wire tag (e.g. `"InsufficientAllowance"`),
      // present when the response carried a recognised rejection envelope. The
      // fine-grained partner of the coarse `category`; the sanitized tag only,
      // never the free-form services description.
      errorType?: OrderBookErrorType;
      message: string;
      // Mirrors the native `OrderbookError::is_retryable` / `backoff_hint`
      // verdict the Rust core emits, so a JavaScript consumer driving its own
      // retry loop reaches the same decision without re-deriving the retryable
      // status set. Optional to match the generated wasm shape, where the field
      // carries `#[serde(default)]`; an absent value reads as `false`.
      retryable?: boolean;
      retryAfterMs?: number;
    }
  | { kind: "subgraph"; message: string }
  | { kind: "signing"; message: string }
  | { kind: "appData"; class?: string; message: string }
  | { kind: "cancelled"; message: string }
  | { kind: "internal"; message: string }
  | { kind: "__unknown"; message: string; raw: unknown };

/** The discriminant tag of a {@link CowError}. */
export type CowErrorKind = CowErrorData["kind"];

// Compile-checked registry of every error kind. `satisfies` forces this table to
// list exactly the `CowErrorData` discriminants: add, remove, or rename a kind
// and the build fails here until the table matches, so the runtime
// known-kind check in `normalizeError` can never drift from the type.
const KNOWN_KINDS = {
  invalidInput: true,
  unknownEnumValue: true,
  unsupportedChain: true,
  walletRequest: true,
  walletTimeout: true,
  transport: true,
  orderbook: true,
  subgraph: true,
  signing: true,
  appData: true,
  cancelled: true,
  internal: true,
  __unknown: true,
} satisfies Record<CowErrorKind, true>;

function isKnownKind(kind: string): kind is CowErrorKind {
  return Object.prototype.hasOwnProperty.call(KNOWN_KINDS, kind);
}

const CANCELLED_MESSAGE =
  "Operation was cancelled. Create a fresh AbortController or retry without an already-aborted signal.";

class CowErrorObject extends Error {
  readonly kind!: CowErrorKind;

  constructor(data: CowErrorData) {
    const message =
      typeof data.message === "string" && data.message.length > 0
        ? data.message
        : defaultMessageFor(data);
    super(message);
    // Restore the prototype link. `extends Error` loses it under ES5/CommonJS
    // transpilation, which would break `instanceof CowError`.
    Object.setPrototypeOf(this, new.target.prototype);
    // Publish `kind` and the per-kind fields (including `raw`) as enumerable own
    // properties so destructuring, `JSON.stringify`, and `toJSON` all see them.
    Object.assign(this, data);
    // `super(message)` leaves `message` non-enumerable, so `JSON.stringify`
    // would drop it; re-publish it as an enumerable own property.
    Object.defineProperty(this, "message", {
      value: message,
      enumerable: true,
      writable: true,
      configurable: true,
    });
  }

  /**
   * Returns the full enumerable field set ({@link CowErrorData}): `kind`,
   * `message`, the per-kind fields, and `raw` for `__unknown`. `name` and
   * `stack` stay off the JSON form. Pairs with {@link CowError.fromJSON} to move
   * an error across a `structuredClone` / worker boundary without losing fields.
   */
  toJSON(): CowErrorData {
    return { ...this } as unknown as CowErrorData;
  }

  /**
   * Rehydrates a {@link CowError} from a plain object — the output of
   * {@link CowError.toJSON} or a structured clone that stripped the prototype.
   * An unrecognised shape becomes the `__unknown` sentinel rather than throwing.
   */
  static fromJSON(value: unknown): CowError {
    return normalizeError(value);
  }
}

// Every instance reports `name === "CowError"` (non-enumerable, matching the
// built-in `Error.name`), so it stays out of the `toJSON` / `JSON.stringify`
// field set.
CowErrorObject.prototype.name = "CowError";

interface CowErrorConstructor {
  new (data: CowErrorData): CowError;
  readonly prototype: CowError;
  /** See {@link CowErrorObject.fromJSON}. */
  fromJSON(value: unknown): CowError;
}

/**
 * The error every SDK call throws: a real {@link Error} subclass whose instances
 * also form a discriminated union keyed by `kind`. Catch it, narrow with
 * {@link isCowError} (or `instanceof CowError`), then `switch (e.kind)` to reach
 * the typed per-kind fields. The redacted `message` is actionable on its own.
 */
export type CowError = CowErrorObject & CowErrorData;

/** Runtime constructor and static helpers for {@link CowError}. */
export const CowError: CowErrorConstructor = CowErrorObject as unknown as CowErrorConstructor;

/**
 * Narrows an unknown caught value to a {@link CowError}. Equivalent to
 * `value instanceof CowError`, exported so consumers do not depend on the
 * class identity directly.
 */
export function isCowError(value: unknown): value is CowError {
  return value instanceof CowError;
}

/**
 * Whether a caught value is a retryable orderbook failure: the SDK exhausted its
 * own retry budget on a transient (rate-limit or server) fault, so a later
 * attempt under your own backoff may succeed. Always `false` for a rejection
 * decided on the request's merits, and for any non-orderbook error.
 */
export function isRetryable(value: unknown): boolean {
  return isCowError(value) && value.kind === "orderbook" && value.retryable === true;
}

/**
 * The server-suggested wait before the next attempt, in milliseconds, parsed
 * from a response `Retry-After` header when one was present; `undefined`
 * otherwise.
 */
export function retryAfterMs(value: unknown): number | undefined {
  return isCowError(value) && value.kind === "orderbook" ? value.retryAfterMs : undefined;
}

/**
 * Whether a caught value is a user-initiated rejection rather than a fault — a
 * wallet request the user declined (EIP-1193 code `4001`) or a cancelled
 * operation. A UI should treat these as a soft, non-error state (dismiss the
 * flow) rather than surfacing them as a failure.
 */
export function isUserRejection(value: unknown): boolean {
  if (!isCowError(value)) {
    return false;
  }
  return (value.kind === "walletRequest" && value.code === 4001) || value.kind === "cancelled";
}

export function normalizeError(raw: unknown): CowError {
  if (isRecord(raw)) {
    // The Rust `WasmError` serializes through a json-compatible serializer with
    // serde `rename_all(_fields) = "camelCase"`, so it already crosses the
    // boundary as a camelCase plain object — no field renaming is needed here.
    const kind = typeof raw.kind === "string" ? raw.kind : undefined;

    if (kind && isKnownKind(kind)) {
      if (kind === "__unknown") {
        // Rehydrating an already-`__unknown` error (e.g. via `fromJSON`):
        // preserve its `message` and `raw` exactly, including a `raw` of `null`,
        // so the `toJSON` -> `fromJSON` round-trip is lossless.
        return new CowError({
          kind: "__unknown",
          message:
            typeof raw.message === "string" && raw.message.length > 0
              ? raw.message
              : unknownMessage(),
          raw: "raw" in raw ? raw.raw : raw,
        });
      }

      return new CowError({ ...raw, kind } as CowErrorData);
    }

    if (kind) {
      return new CowError({ kind: "__unknown", message: unknownMessage(), raw });
    }
  }

  if (raw instanceof Error) {
    return (
      classifyDeserializationFailure(raw.message) ??
      new CowError({ kind: "internal", message: internalMessage(raw.message) })
    );
  }

  return (
    classifyDeserializationFailure(String(raw)) ??
    new CowError({ kind: "internal", message: internalMessage(String(raw)) })
  );
}

// Input-DTO deserialization failures cross the wasm boundary as a plain
// `Error`: the generated wasm-bindgen glue throws the serde message, so it never
// carries a structured `kind`. These are CALLER input errors — a value that does
// not match the documented input type (unknown enum variant, missing/unknown
// field, wrong type) — not SDK-internal faults, so they must normalize to
// `invalidInput` rather than `internal` (whose contract implies an SDK bug). The
// verbatim detail is preserved because it already names the offending
// field/variant and the expected set, e.g.
// "unknown variant `teleport`, expected `sell` or `buy`".
const DESERIALIZATION_FAILURE_PATTERNS: readonly RegExp[] = [
  /unknown variant `/,
  /missing field `/,
  /unknown field `/,
  /duplicate field `/,
  /invalid type:/,
  /invalid length\b/,
  /invalid value:/,
  /data did not match any variant/,
];

function classifyDeserializationFailure(message: string): CowError | undefined {
  if (!DESERIALIZATION_FAILURE_PATTERNS.some((pattern) => pattern.test(message))) {
    return undefined;
  }
  const detail = message.replace(/^Error:\s*/, "");
  const field = detail.match(/(?:missing|unknown|duplicate) field `([^`]+)`/)?.[1];
  const reason = `Invalid SDK input: ${detail}. Check the value against the documented input type and retry.`;
  return field !== undefined
    ? new CowError({ kind: "invalidInput", field, message: reason })
    : new CowError({ kind: "invalidInput", message: reason });
}

export function cancelledError(): CowError {
  return new CowError({ kind: "cancelled", message: CANCELLED_MESSAGE });
}

export function invalidInput(field: string, reason: string): CowError {
  return new CowError({
    kind: "invalidInput",
    field,
    message: `Invalid \`${field}\`: ${reason}. Check the value supplied for \`${field}\` and retry with a valid SDK input.`,
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

// Default actionable message for a payload that arrived without one. The wasm
// always supplies a message, so this is a defensive fallback that keeps a
// hand-constructed or future error from carrying a blank `message`.
function defaultMessageFor(data: CowErrorData): string {
  switch (data.kind) {
    case "unknownEnumValue":
      return `Unsupported value \`${data.value}\` for \`${data.field}\`. Use one of the documented values for this field.`;
    case "unsupportedChain":
      return `Unsupported chain ID ${data.chainId}. Call supportedChainIds() before constructing requests and route unsupported networks to another integration.`;
    case "walletTimeout":
      return `Wallet request timed out after ${data.timeoutMs} ms. Increase walletConfig.timeoutMs or ask the user to approve the wallet request before the timeout.`;
    case "cancelled":
      return CANCELLED_MESSAGE;
    case "__unknown":
      return unknownMessage();
    default:
      return "CoW Protocol SDK error.";
  }
}

function internalMessage(detail: string): string {
  return `SDK internal error: ${detail}. This indicates serialization or invariant failure; retry with the same inputs only after checking the reported input shape.`;
}

function unknownMessage(): string {
  return "SDK received an unrecognized error variant. Inspect raw, preserve it in logs without credentials, and update the SDK if the variant is now documented.";
}
