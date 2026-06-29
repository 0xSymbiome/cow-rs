/* tslint:disable */
/* eslint-disable */

export interface WalletConfig {
    timeoutMs?: number;
}

export interface SigningOptions extends SdkClientOptions {
    walletConfig?: WalletConfig;
}

export type TypedDataSignerCallback = (
envelope: TypedDataEnvelope<Value>,
) => Promise<string> | string;

export type DigestSignerCallback = (
digest: string,
) => Promise<string> | string;

export type CustomEip1271Callback = (
request: CowEip1271SignRequest,
) => Promise<string> | string;



export type Value = unknown;
export type CowError = WasmError;

export interface SdkClientOptions {
    timeoutMs?: number;
    signal?: AbortSignal;
}


/**
 * A decoded `GPv2Settlement` (or inherited `GPv2Signing`) event.
 *
 * Projects `cow_sdk_contracts::SettlementEvent`. Addresses and the order UID
 * are lowercase `0x`-prefixed hex; amounts are base-10 atom strings; the
 * interaction `selector` is a `0x`-prefixed 4-byte hex string. The `kind`
 * discriminator distinguishes the variants.
 */
export type SettlementEvent = { kind: "trade"; owner: string; sellToken: string; buyToken: string; sellAmount: string; buyAmount: string; feeAmount: string; orderUid: string } | { kind: "interaction"; target: string; value: string; selector: string } | { kind: "settlement"; solver: string } | { kind: "orderInvalidated"; owner: string; orderUid: string } | { kind: "preSignature"; owner: string; orderUid: string; signed: boolean };

/**
 * A decoded eth-flow on-chain order lifecycle event.
 *
 * Projects `cow_sdk_contracts::EthFlowEvent`. The placement `order` reuses the
 * canonical [`OrderData`] shape (its `validTo` is the on-chain clamped value;
 * the trader\'s real expiry travels in the opaque `data` trailer). `signature`
 * and `data` are `0x`-prefixed hex strings carrying the raw on-chain signature
 * payload and the opaque trailing data field; addresses and the order UID are
 * lowercase `0x`-prefixed hex. The `kind` discriminator distinguishes the
 * variants.
 */
export type EthFlowEvent = { kind: "orderPlacement"; sender: string; order: OrderData; signingScheme: string; signature: string; data: string } | { kind: "orderInvalidation"; orderUid: string } | { kind: "orderRefund"; orderUid: string; refunder: string };

/**
 * A single EIP-712 typed-data field descriptor.
 */
export interface TypedDataField {
    /**
     * Field name as it appears in the typed-data schema.
     */
    name: string;
    /**
     * Solidity type name for the field.
     */
    type: string;
}

/**
 * Canonical non-negative `uint256` quantity.
 *
 * `Amount` is the typed boundary for atomic token values on every
 * `CoW` Protocol surface: contract hashing, EIP-712 typed data,
 * orderbook DTOs, and decimal-aware display. The newtype is
 * `#[repr(transparent)]` over [`alloy_primitives::U256`], so the
 * in-memory layout is bit-for-bit identical to the alloy primitive and
 * conversion at the alloy seam is free at runtime through
 * [`Amount::as_u256`] (borrowed), [`Amount::into_u256`] (owned), or
 * [`From`] / [`Into`].
 *
 * `Amount` carries cow-owned [`fmt::Display`], [`Serialize`], and
 * [`Deserialize`] impls so the wire form stays the canonical decimal
 * string the orderbook and contract layer accept. The cow-owned
 * `Deserialize` is strict-decimal fail-closed: it rejects `0x`, `0X`,
 * `0o`, `0O`, `0b`, `0B` prefixes (the four alternative radices the
 * alloy [`U256`] `FromStr` impl would otherwise accept silently) so the
 * cow JSON-decimal-only wire contract holds even when the value is fed
 * through serde rather than [`Amount::new`].
 *
 * # Construction
 *
 * Pick the constructor that matches the value you already hold; every
 * path lands on the same atomic `uint256`:
 *
 * - Raw atomic units from an integer — [`Amount::from`] (`u32` / `u64` /
 *   `u128` / `usize`) or [`Amount::from_u256`].
 * - Whole display units from a number — [`Amount::from_units`], for
 *   example `Amount::from_units(1000, 6)` for 1000 USDC (no string and no
 *   hand-counted zeros).
 * - Fractional or untrusted-text display units — [`Amount::parse_units`],
 *   for example `Amount::parse_units(\"1.5\", 18)` for 1.5 WETH.
 * - A decimal or `0x`-hex string of atomic units from a CLI flag,
 *   environment variable, or config file — [`Amount::new`].
 *
 * [`Amount::format_units`] is the inverse of the unit-scaled constructors
 * for human-readable display.
 *
 * # Surface boundary
 *
 * The arithmetic surface is intentionally narrower than the inner
 * [`alloy_primitives::U256`]. `Amount` does **not** expose:
 *
 * - `Add` / `Sub` / `Mul` (and the `*Assign` operators): the bare
 *   `+` `-` `*` operators on the inner `U256` wrap silently on
 *   overflow and underflow, which is incompatible with
 *   financial-amount safety — `a - b` for `a < b` would silently
 *   become a value near `2^256`. Typed arithmetic is therefore
 *   fallible by return: use [`Amount::checked_add`] /
 *   [`Amount::checked_sub`] / [`Amount::checked_mul`] (`-> Option`),
 *   or the explicit [`Amount::saturating_add`] /
 *   [`Amount::saturating_sub`] / [`Amount::saturating_mul`] clamps.
 *   A caller who genuinely wants wrapping reaches through
 *   [`Amount::as_u256`] / [`Amount::into_u256`], making the wrapping
 *   intent visible at the type boundary.
 * - `wrapping_*` / `overflowing_*`: same rationale; the wrapping and
 *   `(value, overflow)` tuple forms belong at the low-level
 *   primitive seam, not on the typed financial surface.
 * - Exponentiation (`pow`, `checked_pow`, `saturating_pow`): raising
 *   a token amount to a power has no money meaning, so no
 *   exponentiation form is exposed.
 * - Bit-inspection helpers (`bit_len`, `bits`, `count_ones`,
 *   `count_zeros`, `leading_zeros`, `trailing_zeros`,
 *   `is_power_of_two`, `next_power_of_two`): counting or measuring
 *   the bits of a token amount has no money meaning either, so none
 *   are exposed. A caller that genuinely needs them reaches through
 *   [`Amount::as_u256`].
 *
 * The shipped surface is: [`Amount::ZERO`], [`Amount::MAX`],
 * [`Amount::new`], [`Amount::checked_add`] / [`Amount::checked_sub`]
 * / [`Amount::checked_mul`], [`Amount::saturating_add`] /
 * [`Amount::saturating_sub`] / [`Amount::saturating_mul`], and
 * [`Amount::as_u256`] / [`Amount::into_u256`] for the explicit alloy
 * seam. This covers every operation cow\'s own crates need to perform
 * on a typed amount.
 *
 * There is no `From<String>` or `From<&str>` conversion: construct through
 * [`Amount::new`] or [`Amount::parse_units`] so malformed input fails closed
 * at the typed boundary rather than via an infallible `.into()`.
 */
export type Amount = string;

/**
 * Coarse, switchable classification of an orderbook rejection, mirrored for
 * the JS error surface.
 *
 * A consumer can branch on the action a rejection calls for — fix the
 * request, fund the wallet, re-quote, wait, or escalate — without matching
 * every wire tag. The category carries no message or code, so it never
 * re-exposes redacted rejection text.
 */
export type OrderBookRejectionCategoryDto = "authorization" | "insufficientFunds" | "invalidOrder" | "notFound" | "conflict" | "unfulfillable" | "server" | "__unknown";

/**
 * Custom EIP-1271 callback request.
 */
export interface CowEip1271SignRequest {
    /**
     * Unsigned order being signed.
     */
    order: OrderData;
    /**
     * Typed-data envelope.
     */
    typedData: TypedDataEnvelope<Value>;
    /**
     * Owner or smart-account address.
     */
    owner: string;
    /**
     * Numeric chain id.
     */
    chainId: number;
}

/**
 * Deployment address output.
 *
 * A default-flavour boundary construct built by the leaf\'s host-safe `helpers`
 * from the chain deployment registry and surfaced by `deploymentAddresses`. The
 * shape is always defined so the host-side `helpers` can build it; only the
 * TypeScript declaration derive is scoped to the wasm-bindgen target.
 */
export interface DeploymentAddresses {
    /**
     * Settlement contract.
     */
    settlement: string;
    /**
     * Vault relayer contract.
     */
    vaultRelayer: string;
    /**
     * `EthFlow` contract.
     */
    ethFlow: string;
}

/**
 * Destination to which the `buyAmount` is transferred upon order fulfillment.
 *
 * This mirrors the services `BuyTokenDestination` enum byte-for-byte on the
 * wire. The buy-side payout path only accepts the ERC-20 and internal
 * variants; the [`SellTokenSource::External`] variant has no buy-side
 * counterpart.
 */
export type BuyTokenDestination = "erc20" | "internal";

/**
 * Generated order UID output.
 *
 * A default-flavour boundary projection that renames the native signing crate\'s
 * generated-order-id fields to the `{orderUid, orderDigest}` the boundary
 * surfaces (from `computeOrderUid` / `orderDigest`). The rename helper that
 * builds it from the native type lives in the leaf\'s host-safe `helpers`; this
 * module carries only the boundary shape. The shape is always defined; only the
 * TypeScript declaration derive is scoped to the wasm-bindgen target.
 */
export interface GeneratedOrderUid {
    /**
     * Compact order UID.
     */
    orderUid: string;
    /**
     * Underlying order digest.
     */
    orderDigest: string;
}

/**
 * Generic EIP-712 envelope shape used by typed helpers and signer payloads.
 *
 * The signer-facing alias uses a canonical JSON string for `message` so the
 * payload travels as one self-contained, digest-complete value: domain,
 * full type map, primary-type name, and message together.
 */
export interface TypedDataEnvelope<M> {
    /**
     * Domain metadata used to compute the typed-data digest.
     */
    domain: TypedDataDomain;
    /**
     * Primary type name for the payload.
     */
    primaryType: string;
    /**
     * Full type map including the primary type and `EIP712Domain`.
     *
     * Typed as `Record` on the TypeScript boundary because the runtime
     * serializer emits a plain JavaScript object for the `BTreeMap`; the
     * override aligns the generated declaration with the wire shape.
     */
    types: Record<string, TypedDataField[]>;
    /**
     * Payload message body.
     */
    message: M;
}

/**
 * Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
 *
 * The wire form is the protocol-canonical `0x`-prefixed 66-character
 * lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
 * [`alloy_primitives::B256`] and forwards `Display`/`Serialize`/
 * `Deserialize` to the inner alloy type, whose canonical defaults already
 * emit the cow lowercase wire form.
 */
export type Hash32 = `0x${string}`;

/**
 * JS-visible typed error envelope for every wasm export.
 */
export type WasmError = { kind: "invalidInput"; message: string; field?: string } | { kind: "unknownEnumValue"; message: string; field: string; value: string } | { kind: "unsupportedChain"; message: string; chainId: number } | { kind: "walletRequest"; method: string; code?: number; message: string } | { kind: "walletTimeout"; message: string; timeoutMs: number } | { kind: "transport"; class: string; message: string; status?: number; headers?: [string, string][]; body?: string } | { kind: "orderbook"; code?: string; category?: OrderBookRejectionCategoryDto; errorType?: string; message: string; retryable?: boolean; retryAfterMs?: number } | { kind: "subgraph"; message: string } | { kind: "signing"; message: string } | { kind: "appData"; class?: string; message: string } | { kind: "cancelled"; message: string } | { kind: "internal"; message: string } | { kind: "__unknown"; message: string; raw: Value };

/**
 * Raw EVM event log accepted by the on-chain event decoders.
 *
 * `topics` carries the indexed log topics as `0x`-prefixed 32-byte hex
 * strings with topic-0 (the event signature hash) first; `data` is the
 * ABI-encoded non-indexed payload as a `0x`-prefixed hex string (`\"0x\"` for an
 * empty payload).
 */
export interface EventLog {
    /**
     * Indexed log topics as 0x-prefixed 32-byte hex strings (topic-0 first).
     */
    topics: string[];
    /**
     * ABI-encoded non-indexed log data as a 0x-prefixed hex string.
     */
    data: string;
}

/**
 * Sell or buy side of a trade.
 *
 * Encoded as `keccak256(\"buy\")` / `keccak256(\"sell\")` in the EIP-712
 * `Order` type. The set of variants is fixed by the protocol; adding a third
 * variant would change the protocol, not the SDK. Classified as
 * `protocol-fixed-exhaustive` in the workspace enum policy manifest.
 */
export type OrderKind = "sell" | "buy";

/**
 * Signed order returned by wallet callback exports.
 */
export interface SignedOrder {
    /**
     * Compact order UID.
     */
    orderUid: string;
    /**
     * Signature payload submitted to the orderbook.
     */
    signature: string;
    /**
     * Signing scheme.
     */
    signingScheme: string;
    /**
     * Effective owner submitted as `from`.
     */
    from: string;
    /**
     * Underlying order digest.
     */
    orderDigest: string;
    /**
     * Typed-data envelope used for signing.
     */
    typedData: TypedDataEnvelope<Value>;
    /**
     * Optional quote id.
     */
    quoteId?: number;
}

/**
 * Source from which the `sellAmount` is drawn upon order fulfillment.
 *
 * This mirrors the services `SellTokenSource` enum byte-for-byte on the wire.
 * Orders model the sell-side allowance path independently of the buy-side
 * payout path, which is typed as [`BuyTokenDestination`].
 */
export type SellTokenSource = "erc20" | "external" | "internal";

/**
 * Typed-data domain metadata used for EIP-712 signing.
 */
export interface TypedDataDomain {
    /**
     * Human-readable protocol name.
     */
    name: string;
    /**
     * Domain version string.
     */
    version: string;
    /**
     * Numeric chain id for the typed-data domain.
     */
    chainId: number;
    /**
     * Contract address used as the domain verifier.
     */
    verifyingContract: Address;
}

/**
 * User-domain order shape prepared for signing and trading workflows.
 *
 * It is the canonical signed-order payload and mirrors the upstream services
 * `OrderData` byte-for-byte (the same field set, EIP-712 type hash, and field
 * ordering). This is not an orderbook wire DTO or an ABI struct. It is hashed
 * directly by `cow_sdk_contracts::hash_order` for the EIP-712 digest and UID
 * (a `receiver` of `address(0)` is the legal \"pay-to-owner\" sentinel). It is
 * submitted to the orderbook as `cow_sdk_orderbook::OrderCreation` and read
 * back as the separate `cow_sdk_orderbook::Order` response record.
 *
 * All fields are public and the struct is exhaustive: the field set is the
 * EIP-712 `Order` struct frozen by the deployed settlement contract, so it
 * cannot grow without a protocol-level change. Construct it as a struct
 * literal (named fields make the three addresses and three amounts
 * impossible to transpose) or through [`OrderData::new`] and the chainable
 * `with_*` setters when positional construction reads better at the call
 * site.
 */
export interface OrderData {
    /**
     * Sell token address.
     */
    sellToken: Address;
    /**
     * Buy token address.
     */
    buyToken: Address;
    /**
     * Receiver of the bought tokens. Defaults to the zero address — which the
     * settlement contract interprets as pay-to-owner — when omitted on the input
     * boundary; always serialized on a resolved order.
     */
    receiver?: Address;
    /**
     * Exact sell amount for sell orders or maximum sell amount for buy orders.
     */
    sellAmount: Amount;
    /**
     * Exact buy amount for buy orders or minimum buy amount for sell orders.
     */
    buyAmount: Amount;
    /**
     * Expiration timestamp encoded as `uint32`.
     */
    validTo: number;
    /**
     * App-data hash linked to the order.
     */
    appData: AppDataHash;
    /**
     * Fee amount encoded in sell-token units.
     */
    feeAmount: Amount;
    /**
     * Order side.
     */
    kind: OrderKind;
    /**
     * Whether the order can be partially filled.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: SellTokenSource;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: BuyTokenDestination;
}

/**
 * Validated 32-byte app-data hash.
 *
 * The wire form is the protocol-canonical `0x`-prefixed 66-character
 * lowercase hexadecimal string. The newtype is `#[repr(transparent)]`
 * over [`alloy_primitives::B256`], so the in-memory layout is
 * bit-for-bit identical to the alloy primitive and conversion at the
 * alloy seam is free at runtime through [`AppDataHash::as_alloy`]
 * (borrowed), [`AppDataHash::into_alloy`] (owned), or [`From`] /
 * [`Into`].
 *
 * `AppDataHash` forwards [`Serialize`] / [`Deserialize`] to the inner
 * [`alloy_primitives::B256`] via `#[serde(transparent)]` because the
 * alloy lowercase 0x-prefixed default already matches the cow wire
 * form. [`fmt::Display`] is a one-line delegate to the inner primitive
 * for the same reason.
 *
 * Equality, hash, and ordering derive from the packed 32-byte
 * representation, which is equivalent to the documented
 * case-insensitive comparison contract because every valid value parses
 * to the same bytes regardless of input casing.
 *
 *
 */
export type AppDataHash = `0x${string}`;

/**
 * Validated EVM address.
 *
 * The wire form is the protocol-canonical `0x`-prefixed 42-character
 * lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
 * [`alloy_primitives::Address`], so the in-memory layout is bit-for-bit
 * identical to the alloy primitive and conversion at the alloy seam is free
 * at runtime through [`Address::as_alloy`] (borrowed), [`Address::into_alloy`]
 * (owned), or [`From`] / [`Into`].
 *
 * `Address` carries cow-owned [`fmt::Display`], [`Serialize`], and
 * [`Deserialize`] impls because alloy\'s default `Display` for
 * [`alloy_primitives::Address`] emits the EIP-55 mixed-case checksum form,
 * while the cow protocol wire form is lowercase. The cow `Display` impl
 * writes `format!(\"{:#x}\", self.0)` which routes through alloy\'s
 * [`fmt::LowerHex`] impl and emits lowercase 0x-prefixed hex.
 *
 * [`PartialEq`], [`Eq`], [`Hash`](std::hash::Hash), [`PartialOrd`], and
 * [`Ord`] derive from the inner alloy primitive, which compares addresses on
 * the packed 20-byte representation.
 */
export type Address = `0x${string}`;

/**
 * Validated `CoW` order UID.
 *
 * The wire form is the protocol-canonical `0x`-prefixed 114-character
 * lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
 * [`alloy_primitives::FixedBytes<56>`] and forwards `Display`/`Serialize`/
 * `Deserialize` to the inner alloy type, whose canonical defaults already
 * emit the cow lowercase wire form.
 *
 *
 *
 */
export type OrderUid = `0x${string}`;

/**
 * Validated hex payload used for calldata and byte blobs.
 *
 * The wire form is the protocol-canonical `0x`-prefixed lowercase
 * hexadecimal string. The newtype is `#[repr(transparent)]` over
 * [`alloy_primitives::Bytes`] and forwards `Display`/`Serialize`/
 * `Deserialize` to the inner alloy type, whose canonical defaults already
 * emit the cow lowercase wire form. Odd-length inputs are left-padded with
 * one zero nibble during construction so the stored value remains
 * byte-aligned hex.
 */
export type HexData = `0x${string}`;

/**
 * Version tag carried by wasm output and error envelopes.
 */
export type SchemaVersion = "v1" | "__unknown";

/**
 * Versioned output envelope.
 */
export interface WasmEnvelope<T> {
    /**
     * Schema version.
     */
    schemaVersion: SchemaVersion;
    /**
     * Envelope payload.
     */
    value: T;
}

/**
 * Wrapped-native token metadata.
 *
 * A default-flavour boundary construct built by the leaf\'s host-safe `helpers`
 * from the native wrapped-native lookup and surfaced by `wrappedNativeToken`.
 * The shape is always defined; only the TypeScript declaration derive is scoped
 * to the wasm-bindgen target.
 */
export interface WrappedNativeToken {
    /**
     * Wrapped-native token contract address.
     */
    address: string;
    /**
     * Token symbol, such as `WETH` or `WXDAI`.
     */
    symbol: string;
    /**
     * Token decimals.
     */
    decimals: number;
}


/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_js_init(): void;

/**
 * Computes the canonical order UID and order digest for an unsigned order.
 *
 * The UID combines the EIP-712 order digest, owner address, and validity
 * timestamp using the same packing rules as the native Rust SDK.
 *
 * @param order Unsigned order fields to hash and pack.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Order owner address included in the UID suffix.
 * @returns A versioned envelope with `orderUid` and `orderDigest`.
 * @throws CowError when the order, owner, or chain id is invalid.
 */
export function computeOrderUid(order: OrderData, chainId: number, owner: string): WasmEnvelope<GeneratedOrderUid>;

/**
 * Decodes an eth-flow on-chain order lifecycle event log into a typed event.
 *
 * Dispatches on the log's topic-0 across the `CoWSwapOnchainOrders`
 * `OrderPlacement` / `OrderInvalidation` events and the `CoWSwapEthFlow`
 * `OrderRefund` event. The decode is fail-closed: the topic set and on-chain
 * signing scheme are validated and every order UID is length-checked, so a
 * malformed or hostile log returns a typed error rather than panicking.
 *
 * @param log Raw log with `topics` (0x-prefixed 32-byte hex, topic-0 first)
 * and `data` (0x-prefixed hex, `"0x"` when empty).
 * @returns A versioned envelope containing the decoded eth-flow event.
 * @throws CowError when the log is malformed or its topic set matches no known
 * eth-flow lifecycle event.
 */
export function decodeEthFlowLog(log: EventLog): WasmEnvelope<EthFlowEvent>;

/**
 * Decodes a `GPv2Settlement` event log into a typed settlement event.
 *
 * Dispatches on the log's topic-0 across `Trade`, `Interaction`, `Settlement`,
 * `OrderInvalidated`, and `PreSignature`. The decode is fail-closed: the topic
 * set is validated before ABI decoding and every order UID is length-checked,
 * so a malformed or hostile log returns a typed error rather than panicking.
 *
 * @param log Raw log with `topics` (0x-prefixed 32-byte hex, topic-0 first)
 * and `data` (0x-prefixed hex, `"0x"` when empty).
 * @returns A versioned envelope containing the decoded settlement event.
 * @throws CowError when the log is malformed or its topic set matches no known
 * settlement event.
 */
export function decodeSettlementLog(log: EventLog): WasmEnvelope<SettlementEvent>;

/**
 * Returns canonical CoW Protocol deployment addresses for a chain.
 *
 * The optional environment selects production or staging deployment data. When
 * omitted, the helper uses the SDK default environment.
 *
 * @param chainId EVM chain id to resolve.
 * @param env Optional CoW environment name, such as `prod` or `staging`.
 * @returns Settlement, VaultRelayer, EthFlow, and AllowListAuth addresses.
 * @throws CowError when the chain or environment is unsupported.
 */
export function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<DeploymentAddresses>;

/**
 * Computes the CoW Protocol EIP-712 domain separator for a supported chain.
 *
 * Use this helper when a JavaScript host needs to compare the domain hash used
 * by the Rust SDK with another signing stack. The input is an EVM chain id,
 * not a CoW environment selector.
 *
 * @param chainId EVM chain id supported by the deployment registry.
 * @returns The `0x`-prefixed 32-byte domain separator.
 * @throws CowError when the chain is not supported.
 */
export function domainSeparator(chainId: number): any;

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA order signature.
 *
 * Use this pure helper when a smart-account flow already has the wrapped ECDSA
 * signature and needs the contract-signature payload bytes expected by CoW
 * Protocol order submission.
 *
 * @param order Unsigned order used to derive the EIP-1271 payload.
 * @param ecdsaSignature Wrapped ECDSA signature as a `0x`-prefixed string.
 * @returns A versioned envelope containing the encoded EIP-1271 payload.
 * @throws CowError when the order or signature is invalid.
 */
export function eip1271SignaturePayload(order: OrderData, ecdsaSignature: string): WasmEnvelope<string>;

/**
 * Builds signer-facing EIP-712 typed data for an unsigned order.
 *
 * The returned envelope contains the domain, type map, primary type, and
 * order message that wallet libraries expect for EIP-712 signing. It is
 * deterministic for the provided order and chain id.
 *
 * @param order Unsigned order fields using the native order shape.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @returns A versioned envelope containing typed-data DTO fields.
 * @throws CowError when order parsing or chain validation fails.
 */
export function orderTypedData(order: OrderData, chainId: number): WasmEnvelope<TypedDataEnvelope<Value>>;

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 *
 * The SDK computes the canonical order digest, passes the digest as a
 * `0x`-prefixed string to the callback, normalizes the signature, and returns
 * an `ethsign` signed-order DTO.
 *
 * @param order Unsigned order fields to sign.
 * @param chainId EVM chain id used for the digest.
 * @param owner Owner address used in the generated order UID.
 * @param digestSigner Callback that signs the digest string.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderEthSignDigest(order: OrderData, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrder>>;

/**
 * Signs an order through a custom EIP-1271 callback.
 *
 * Use this method when the JavaScript host owns the smart-account or
 * account-abstraction client and can return the final contract signature
 * directly. The SDK still builds typed data and the deterministic order UID.
 *
 * @param order Unsigned order to sign.
 * @param chainId EVM chain id for the EIP-712 domain.
 * @param owner Smart-account owner address used in the generated order UID.
 * @param customCallback Callback that returns the final EIP-1271 signature.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed-order DTO.
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithCustomEip1271(order: OrderData, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrder>>;

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 *
 * The SDK sends the EIP-712 envelope to the provided typed-data callback,
 * then converts the returned ECDSA signature into the CoW EIP-1271 payload.
 * Per-call options may attach cancellation and wallet timeout settings.
 *
 * @param order Unsigned order to sign.
 * @param chainId EVM chain id for the EIP-712 domain.
 * @param owner Smart-account owner address used in the generated order UID.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed-order DTO.
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithEip1271(order: OrderData, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrder>>;

/**
 * Signs an order through a typed-data callback.
 *
 * The SDK builds the EIP-712 typed-data envelope, passes it to the callback,
 * normalizes the returned ECDSA signature, and returns the signed-order DTO
 * with the canonical order UID and digest.
 *
 * @param order Unsigned order fields to sign.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Owner address used in the generated order UID.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithTypedDataSigner(order: OrderData, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrder>>;

/**
 * Returns the EVM chain ids supported by the SDK deployment registry.
 *
 * This is a pure helper and does not perform network I/O. The returned list is
 * suitable for runtime validation, UI selection, or capability checks before a
 * client is constructed.
 *
 * @returns A typed array of supported EVM chain ids.
 */
export function supportedChainIds(): Uint32Array;

/**
 * Returns the version of the wasm package runtime.
 *
 * The value comes from the Rust package metadata used to build the wasm
 * artifact and can be included in diagnostics or compatibility checks.
 *
 * @returns The semantic version string for this wasm build.
 */
export function wasmVersion(): string;

/**
 * Returns wrapped-native token metadata for a chain.
 *
 * Use this to recognise a wrap pair in a swap UI — compare a selected token's
 * address against the returned address — or to display the wrapped-native
 * token. This is a pure lookup and performs no network I/O.
 *
 * @param chainId EVM chain id to resolve.
 * @returns The wrapped-native token address, symbol, and decimals.
 * @throws CowError when the chain is not supported.
 */
export function wrappedNativeToken(chainId: number): WasmEnvelope<WrappedNativeToken>;
