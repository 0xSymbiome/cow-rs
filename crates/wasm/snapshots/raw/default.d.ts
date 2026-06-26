/// <reference lib="esnext.disposable" />
/* tslint:disable */
/* eslint-disable */

export interface IpfsClientConfig {
    ipfsUri?: string | null;
    transport?: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface OrderBookClientConfig {
    chainId: number;
    env?: string | null;
    apiKey?: string | null;
    transport?: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface SubgraphClientConfig {
    chainId: number;
    apiKey: string;
    transport?: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface TradingClientConfig {
    chainId: number;
    env?: string | null;
    appCode: string;
    apiKey?: string | null;
    transport?: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



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



export type ContractReadCallback = (
request: ContractCall,
) => Promise<string> | string;



export type CowFetchMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

export interface CowFetchRequest {
    method: CowFetchMethod;
    url: string;
    headers: Record<string, string>;
    body?: string;
    timeoutMs?: number;
    signal?: AbortSignal;
}

export interface CowFetchResponse {
    status: number;
    statusText?: string;
    headers?: Record<string, string>;
    body?: string;
}

export type CowFetchCallback = (
request: CowFetchRequest,
) => Promise<CowFetchResponse> | CowFetchResponse;

export type HttpTransportConfig =
| { kind: "callback"; callback: CowFetchCallback };



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
 * A single order touched by a solver\'s settlement.
 */
export interface SolverCompetitionOrder {
    /**
     * Order UID.
     */
    id: OrderUid;
    /**
     * Effective sell amount including all fees.
     */
    sellAmount: Amount;
    /**
     * Effective buy amount after all fees.
     */
    buyAmount: Amount;
    /**
     * Buy-token address, when rendered by the API.
     */
    buyToken?: Address;
    /**
     * Sell-token address, when rendered by the API.
     */
    sellToken?: Address;
}

/**
 * Allowance helper parameters.
 */
export interface AllowanceParams {
    /**
     * ERC-20 token address.
     */
    tokenAddress: string;
    /**
     * Owner whose allowance should be inspected.
     */
    owner: string;
    /**
     * Optional chain-id override.
     */
    chainId?: number;
    /**
     * Optional environment override.
     */
    env?: string;
    /**
     * Optional vault-relayer deployment override.
     */
    vaultRelayerOverride?: string;
}

/**
 * App-data bundle used by trading quote and post helpers.
 */
export interface TradingAppDataInfo {
    /**
     * Parsed app-data document.
     *
     * Spelled as the `Value` escape hatch on the TypeScript boundary because
     * the app-data document is arbitrary JSON; the native field is the
     * [`AppDataDoc`] alias of `serde_json::Value`.
     */
    doc: Value;
    /**
     * Canonically serialized app-data payload.
     */
    fullAppData: string;
    /**
     * Keccak-256 digest used in protocol order payloads.
     */
    appDataKeccak256: AppDataHash;
}

/**
 * App-data document input.
 *
 * An app-data-flavour boundary type (the TypeScript declaration derive scopes it
 * to the wasm flavours that surface app-data). The app-data document lowering
 * that consumes it lives in the leaf\'s host-safe `helpers`, so this type carries
 * only the structural shape. The shape is always defined so the host-side
 * `helpers` can build it; only the TypeScript declaration derive is scoped to the
 * wasm-bindgen target and the app-data feature.
 */
export interface AppDataParams {
    /**
     * Application code.
     */
    appCode: string;
    /**
     * Metadata object.
     */
    metadata: Value;
    /**
     * Schema version.
     */
    version: string;
    /**
     * Optional environment label.
     */
    environment?: string;
}

/**
 * App-data document output.
 */
export interface AppDataDocument {
    /**
     * App-data document.
     */
    document: Value;
}

/**
 * App-data validation result.
 *
 * An app-data-flavour boundary projection of the typed
 * `Result<(), AppDataError>` the SDK validator returns: `{success, errors}`,
 * where the rendered error text names only the offending public field and
 * never the caller-supplied value. The TypeScript declaration derive scopes it
 * to the wasm flavours that surface app-data.
 */
export interface ValidationResult {
    /**
     * Whether validation succeeded.
     */
    success: boolean;
    /**
     * Errors when validation failed.
     */
    errors?: string;
}

/**
 * Approval-transaction helper parameters.
 *
 * The chain and environment are taken from the `TradingClient`, matching the
 * other transaction builders; only the token, amount, and an optional
 * vault-relayer deployment override are supplied per call.
 */
export interface ApprovalParams {
    /**
     * ERC-20 token address to approve.
     */
    tokenAddress: string;
    /**
     * Approval amount as a base-unit decimal string.
     */
    amount: string;
    /**
     * Optional vault-relayer deployment override.
     */
    vaultRelayerOverride?: string;
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
 * Competition-status kind returned by `/api/v1/orders/{uid}/status`.
 */
export type CompetitionOrderStatusKind = "open" | "scheduled" | "active" | "solved" | "executing" | "traded" | "cancelled";

/**
 * Competition-status response for an order.
 */
export interface CompetitionOrderStatus {
    /**
     * High-level competition status kind.
     */
    type: CompetitionOrderStatusKind;
    /**
     * Optional solver execution payload.
     */
    value?: SolverExecution[];
}

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
 * Derived identifiers for a validated app-data document.
 */
export interface AppDataInfo {
    /**
     * CID representation of the document.
     */
    cid: string;
    /**
     * Serialized JSON content used to derive the digest.
     */
    appDataContent: string;
    /**
     * `0x`-prefixed app-data digest.
     */
    appDataHex: string;
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
 * Executed protocol-fee metadata returned on trade records.
 */
export interface ExecutedProtocolFee {
    /**
     * Fee policy that produced this fee, when services returns it.
     */
    policy?: Value;
    /**
     * Fee amount taken.
     */
    amount?: Amount;
    /**
     * Token in which the fee was taken.
     */
    token?: Address;
}

/**
 * Executed sell and buy amounts for a solver path.
 */
export interface ExecutedAmounts {
    /**
     * Executed sell amount.
     */
    sell: Amount;
    /**
     * Executed buy amount.
     */
    buy: Amount;
}

/**
 * Full app-data response from the orderbook app-data endpoint.
 */
export interface AppDataObject {
    /**
     * Full serialized app-data payload.
     */
    fullAppData: string;
}

/**
 * Full quote cost breakdown.
 */
export interface Costs<T> {
    /**
     * Network fee component.
     */
    networkFee: NetworkFee<T>;
    /**
     * Partner fee component.
     */
    partnerFee: FeeComponent<T>;
    /**
     * Protocol fee component.
     */
    protocolFee: FeeComponent<T>;
}

/**
 * Fully resolved quote result produced by trading quote helpers.
 */
export interface QuoteResults {
    /**
     * Effective trade parameters after SDK defaults and advanced settings were applied.
     */
    tradeParameters: TradeParams;
    /**
     * Suggested slippage in basis points after SDK or custom-provider resolution.
     */
    suggestedSlippageBps: number;
    /**
     * Fee and amount breakdown derived from the orderbook quote.
     *
     * Spelled with the explicit `<Amount>` type argument on the TypeScript
     * boundary: the native field uses the `T = Amount` default of
     * [`QuoteAmountsAndCosts`], but TypeScript generics carry no default, so a
     * bare reference to the emitted `QuoteAmountsAndCosts<T>` would not
     * type-check.
     */
    amountsAndCosts: QuoteAmountsAndCosts<Amount>;
    /**
     * Unsigned order payload produced for signing or on-chain submission.
     */
    orderToSign: OrderData;
    /**
     * Raw orderbook quote response.
     */
    quoteResponse: OrderQuoteResponse;
    /**
     * App-data document, serialized payload, and digest used by the quote flow.
     */
    appDataInfo: TradingAppDataInfo;
    /**
     * Originating orderbook runtime binding captured by the quote flow.
     *
     * Quote-derived posting requires this binding to match the submission-time
     * orderbook runtime. It is omitted from serialization when `None` and
     * defaults back to `None` when absent, so a `QuoteResults` whose binding was
     * not carried through — rehydrated from storage, or rebuilt without it —
     * fails closed on resubmission with `TradingError::MissingQuoteOrderbookBinding`
     * rather than posting against an unverified runtime. A faithful round-trip
     * preserves a `Some` binding; the gate enforces runtime-authority match, not
     * quote freshness (the quote\'s own expiry governs that).
     */
    orderbookBinding?: OrderbookBinding;
    /**
     * Typed order-facing envelope kept for consumers while signers use the
     * lower-level `TypedDataPayload` seam internally.
     *
     * Spelled as the concrete `TypedDataEnvelope<OrderData>` rather than the
     * `OrderTypedData` alias so the generated TypeScript boundary references
     * the emitted `TypedDataEnvelope<OrderData>` declaration; the alias is a
     * transparent synonym, so native construction and reads are unchanged.
     */
    orderTypedData: TypedDataEnvelope<OrderData>;
}

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
 * Generic fee component represented by amount and basis points.
 */
export interface FeeComponent<T> {
    /**
     * Fee amount.
     */
    amount: T;
    /**
     * Fee in basis points.
     */
    bps: number;
}

/**
 * Generic sell/buy amount pair.
 */
export interface Amounts<T> {
    /**
     * Sell-side amount.
     */
    sellAmount: T;
    /**
     * Buy-side amount.
     */
    buyAmount: T;
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
 * Jitter strategy accepted by JS client constructors.
 */
export type JitterStrategyConfig = "none" | "full" | "equal" | "decorrelated";

/**
 * Limit-order request accepted by posting and signing helpers.
 */
export interface LimitTradeParams {
    /**
     * Order kind.
     */
    kind: OrderKind;
    /**
     * Optional owner override. Signer address becomes the fallback in signer-backed flows.
     */
    owner?: Address;
    /**
     * Sell-token address.
     */
    sellToken: Address;
    /**
     * Buy-token address.
     */
    buyToken: Address;
    /**
     * Sell amount before transformations.
     */
    sellAmount: Amount;
    /**
     * Buy amount before transformations.
     */
    buyAmount: Amount;
    /**
     * Optional quote id required by some flows such as `EthFlow` posting.
     */
    quoteId?: number;
    /**
     * Optional environment override for endpoint and contract resolution.
     */
    env?: CowEnv;
    /**
     * Optional settlement contract overrides keyed by chain id.
     */
    settlementContractOverride?: Record<string, string>;
    /**
     * Optional `EthFlow` contract overrides keyed by chain id.
     */
    ethFlowContractOverride?: Record<string, string>;
    /**
     * Whether partial fills are allowed. Defaults to `false` when omitted on the input boundary.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source. Defaults to `erc20` when omitted on the input boundary.
     */
    sellTokenBalance?: SellTokenSource;
    /**
     * Buy-token balance destination. Defaults to `erc20` when omitted on the input boundary.
     */
    buyTokenBalance?: BuyTokenDestination;
    /**
     * Optional explicit slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: Address;
    /**
     * Optional relative validity duration in seconds.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata merged into app-data and fee calculations.
     */
    partnerFee?: PartnerFee;
}

/**
 * Native-currency sell transaction bundle.
 */
export interface BuiltSellNativeCurrencyTx {
    /**
     * Deterministic order UID.
     */
    orderUid: string;
    /**
     * Transaction request to submit.
     */
    transaction: TransactionRequest;
    /**
     * Unsigned order encoded by the transaction.
     */
    orderToSign: OrderData;
    /**
     * Effective order owner.
     */
    from: string;
}

/**
 * Native-price response from `/api/v1/token/{token}/native_price`.
 */
export interface NativePriceResponse {
    /**
     * Token price quoted in the chain\'s native asset.
     */
    price: number;
}

/**
 * Nested auction snapshot inside solver-competition responses.
 */
export interface CompetitionAuction {
    /**
     * Order UIDs participating in the competition.
     */
    orders?: OrderUid[];
    /**
     * Clearing prices keyed by token address.
     */
    prices?: Record<string, string>;
}

/**
 * Network-fee amounts expressed in both quote currencies.
 */
export interface NetworkFee<T> {
    /**
     * Network fee expressed in sell-token units.
     */
    amountInSellCurrency: T;
    /**
     * Network fee expressed in buy-token units.
     */
    amountInBuyCurrency: T;
}

/**
 * On-chain order placement metadata returned by the orderbook for orders that
 * originated from an on-chain submission path.
 */
export interface OnchainOrderData {
    /**
     * Sender address associated with the on-chain placement.
     */
    sender: Address;
    /**
     * Placement error emitted by services, when on-chain placement failed.
     */
    placementError?: string;
}

/**
 * One typed partner-fee policy object.
 */
export type PartnerFeePolicy = { volumeBps: number; recipient: Address } | { surplusBps: number; maxVolumeBps: number; recipient: Address } | { priceImprovementBps: number; maxVolumeBps: number; recipient: Address };

/**
 * Optional pre and post interactions attached to an order response.
 */
export interface OrderInteractions {
    /**
     * Interactions executed before the order\'s trade.
     */
    pre?: InteractionData[];
    /**
     * Interactions executed after the order\'s trade.
     */
    post?: InteractionData[];
}

/**
 * Order class surfaced by the orderbook API.
 */
export type OrderClass = "market" | "limit" | "liquidity";

/**
 * Order lifecycle status returned by the orderbook API.
 */
export type OrderStatus = "presignaturePending" | "open" | "fulfilled" | "cancelled" | "expired";

/**
 * Order transaction helper parameters.
 */
export interface OrderTraderParams {
    /**
     * Target order UID.
     */
    orderUid: string;
    /**
     * Optional chain-id override.
     */
    chainId?: number;
    /**
     * Optional environment override.
     */
    env?: string;
    /**
     * Optional settlement-contract overrides keyed by chain id.
     *
     * Typed as `Record` rather than `Map` because the runtime
     * serializer emits a plain JavaScript object for `BTreeMap`
     * fields; the override aligns the declaration with the runtime.
     */
    settlementContractOverride?: Record<string, string>;
    /**
     * Optional `EthFlow` contract overrides keyed by chain id.
     *
     * Typed as `Record` rather than `Map` for the same runtime
     * alignment reason as `settlement_contract_override`.
     */
    ethFlowContractOverride?: Record<string, string>;
}

/**
 * Orderbook order response DTO.
 *
 * This response includes status, owner, uid, execution totals, and `EthFlow`
 * metadata that are not part of the user-domain signing order or contract ABI
 * hashing payload. It is one of two order-shaped types: the signing and
 * EIP-712 hashing pivot is `cow_sdk_core::OrderData`, and this is the
 * orderbook record. Use [`Order::signing_order`] to project a fetched order
 * back into the `cow_sdk_core::OrderData` for client-side digest or UID
 * re-derivation; it fails closed for `EthFlow` orders, whose response fields
 * are rewritten for display.
 */
export interface Order {
    /**
     * Sell-token address.
     */
    sellToken: Address;
    /**
     * Buy-token address.
     */
    buyToken: Address;
    /**
     * Optional receiver override.
     */
    receiver?: Address;
    /**
     * Sell amount in the upstream decimal-string wire shape.
     */
    sellAmount: Amount;
    /**
     * Buy amount in the upstream decimal-string wire shape.
     */
    buyAmount: Amount;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo: number;
    /**
     * App-data hash attached to the order.
     */
    appData: AppDataHash;
    /**
     * Optional app-data hash echoed for debugging by the orderbook.
     */
    appDataHash?: AppDataHash;
    /**
     * Order-level fee echoed on the orderbook response; always `\"0\"` in
     * practice because services rejects non-zero order-level fees.
     *
     * Stored under the upstream wire name `feeAmount` so deserialization
     * preserves services-schema parity; the value is not exposed on the
     * public Rust surface.
     *
     * Always present in the orderbook response (services rejects a non-zero
     * order-level fee but still echoes the `\"0\"` field), so the wire field is
     * required rather than defaulted: a response missing it is malformed.
     */
    feeAmount: Amount;
    /**
     * Strict balance-check flag accepted by services when the order was created.
     */
    fullBalanceCheck?: boolean;
    /**
     * Order kind.
     */
    kind: OrderKind;
    /**
     * Whether partial fills are allowed. Always serialized on the response, so
     * the wire field is required rather than defaulted.
     */
    partiallyFillable: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: SellTokenSource;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: BuyTokenDestination;
    /**
     * Signature scheme used for `signature`. Always serialized on the
     * response, so the wire field is required rather than defaulted.
     */
    signingScheme: SigningScheme;
    /**
     * Raw signature string.
     */
    signature: string;
    /**
     * Effective owner field returned by the API, when present.
     */
    from?: Address;
    /**
     * Quote id used when the order originated from a quote.
     */
    quoteId?: number;
    /**
     * Order class. Always serialized on the response, so the wire field is
     * required rather than defaulted.
     */
    class: OrderClass;
    /**
     * Canonical owner surfaced by the orderbook response.
     */
    owner: Address;
    /**
     * Order UID.
     */
    uid: OrderUid;
    /**
     * Creation timestamp string returned by the API. Always serialized on the
     * response, so the wire field is required rather than defaulted; the
     * `creationTime` alias is retained for the legacy response key.
     */
    creationDate: string;
    /**
     * Executed sell amount. Always serialized on the response, so the wire
     * field is required rather than defaulted.
     */
    executedSellAmount: Amount;
    /**
     * Executed sell amount before fees. Always serialized on the response, so
     * the wire field is required rather than defaulted.
     */
    executedSellAmountBeforeFees: Amount;
    /**
     * Executed buy amount. Stays `\"0\"` on the wire until the order\'s first
     * fill, rather than being absent, so the wire field is required rather
     * than defaulted.
     */
    executedBuyAmount: Amount;
    /**
     * Executed fee component, when provided.
     */
    executedFee?: Amount;
    /**
     * Deprecated legacy fee value some orderbook responses still emit on
     * older order payloads alongside [`executed_fee`].
     *
     * Surfaced as a read-only sibling so consumers that need the legacy
     * summation can compute it explicitly as
     * `executed_fee + executed_fee_amount`. New code should prefer
     * [`executed_fee`]; [`total_fee`] intentionally does not fold this
     * field in.
     *
     * [`executed_fee`]: Order::executed_fee
     * [`total_fee`]: Order::total_fee
     */
    executedFeeAmount?: Amount;
    /**
     * Token in which the executed fee was captured, when returned.
     */
    executedFeeToken?: Address;
    /**
     * Whether the order was invalidated by the protocol.
     *
     * Kept defaulted: although the services schema lists it as required, the
     * `EthFlow` response shape omits it (see the `sample_ethflow_order_json`
     * fixture), so the field must remain absent-able on the wire.
     */
    invalidated?: boolean;
    /**
     * Order lifecycle status. Always serialized on the response, so the wire
     * field is required rather than defaulted.
     */
    status: OrderStatus;
    /**
     * Whether services classified the order as a liquidity order.
     */
    isLiquidityOrder?: boolean;
    /**
     * On-chain user for `EthFlow`-style orders.
     */
    onchainUser?: Address;
    /**
     * `EthFlow`-specific metadata.
     */
    ethflowData?: EthflowData;
    /**
     * On-chain placement metadata, when services returns it.
     */
    onchainOrderData?: OnchainOrderData;
    /**
     * Full app-data payload, when services returns it.
     */
    fullAppData?: string;
    /**
     * Settlement contract address against which the order was signed.
     */
    settlementContract: Address;
    /**
     * Stored quote metadata for quote-linked orders.
     */
    quote?: StoredOrderQuote;
    /**
     * Optional pre and post interactions associated with the order.
     */
    interactions?: OrderInteractions;
    /**
     * Total fee normalized by the SDK transform layer.
     *
     * Kept defaulted: this is an SDK-synthesized field with no services-schema
     * counterpart, so an inbound orderbook response never carries it and it
     * must remain absent-able on the wire.
     */
    totalFee?: Amount;
}

/**
 * Orderbook order-creation input.
 */
export interface OrderCreation {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Optional receiver.
     */
    receiver?: string;
    /**
     * Sell amount.
     */
    sellAmount: string;
    /**
     * Buy amount.
     */
    buyAmount: string;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo: number;
    /**
     * Inline app-data payload.
     */
    appData?: string;
    /**
     * App-data hash.
     */
    appDataHash?: string;
    /**
     * Order-level fee amount. The orderbook accepts only zero.
     */
    feeAmount?: string;
    /**
     * Strict balance-check flag.
     */
    fullBalanceCheck?: boolean;
    /**
     * Order side.
     */
    kind: OrderKind;
    /**
     * Whether partial fills are allowed.
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
    /**
     * Signature scheme.
     */
    signingScheme: string;
    /**
     * Raw signature.
     */
    signature: string;
    /**
     * Effective owner.
     */
    from: string;
    /**
     * Optional quote id.
     */
    quoteId?: number;
}

/**
 * Orderbook quote request input.
 */
export interface OrderQuoteRequest {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Optional explicit receiver.
     */
    receiver?: string;
    /**
     * Quote owner.
     */
    from: string;
    /**
     * Quote side.
     */
    kind: OrderKind;
    /**
     * Sell amount before fee for sell quotes.
     */
    sellAmountBeforeFee?: string;
    /**
     * Buy amount after fee for buy quotes.
     */
    buyAmountAfterFee?: string;
    /**
     * Relative validity duration in seconds.
     */
    validFor?: number;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Inline app-data payload.
     */
    appData?: string;
    /**
     * App-data hash.
     */
    appDataHash?: string;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: SellTokenSource;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: BuyTokenDestination;
    /**
     * Quote-quality mode.
     */
    priceQuality?: string;
    /**
     * Expected signing scheme.
     */
    signingScheme?: string;
    /**
     * Whether the eventual order is expected to be on-chain.
     */
    onchainOrder?: boolean;
    /**
     * Optional verification gas limit.
     */
    verificationGasLimit?: number;
    /**
     * Optional request timeout in milliseconds.
     */
    timeout?: number;
}

/**
 * Pagination options shared by orderbook list helpers.
 */
export interface PaginationOptions {
    /**
     * Pagination offset.
     */
    offset?: number;
    /**
     * Pagination limit.
     */
    limit?: number;
}

/**
 * Quote metadata stored with an order response when an order was created from
 * a quote.
 */
export interface StoredOrderQuote {
    /**
     * Estimated gas units required to execute the quoted trade.
     */
    gasAmount: string;
    /**
     * Estimated gas price at quote time, in wei per gas unit.
     */
    gasPrice: string;
    /**
     * Sell-token price in native-token atoms per sell-token atom.
     */
    sellTokenPrice: string;
    /**
     * Quoted sell amount.
     */
    sellAmount: Amount;
    /**
     * Quoted buy amount.
     */
    buyAmount: Amount;
    /**
     * Estimated network fee in sell-token atoms.
     */
    feeAmount: Amount;
    /**
     * Solver address that provided the quote.
     */
    solver: Address;
    /**
     * Whether the quote was verified through simulation.
     */
    verified: boolean;
    /**
     * Additional services-provided quote metadata, when present.
     */
    metadata?: Value;
}

/**
 * Quote order data returned by the orderbook API.
 *
 * This mirrors the orderbook `OrderParameters` schema — the order
 * parameters payload returned inside a `/quote` response — and is named
 * `QuoteData` for that role (see ADR 0058). It is a wire DTO, not the
 * user-domain signing order (`cow_sdk_core::OrderData`), which is also the
 * contract EIP-712 hashing input. It accepts the orderbook\'s full-app-data
 * echo shape and resolves that into the app-data hash used by downstream
 * order creation.
 */
export interface QuoteData {
    /**
     * Sell-token address.
     */
    sellToken: Address;
    /**
     * Buy-token address.
     */
    buyToken: Address;
    /**
     * Optional receiver override.
     */
    receiver?: Address;
    /**
     * Sell amount in the upstream decimal-string wire shape.
     */
    sellAmount: Amount;
    /**
     * Buy amount in the upstream decimal-string wire shape.
     */
    buyAmount: Amount;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo: number;
    /**
     * Effective app-data hash derived from the orderbook response.
     */
    appData: AppDataHash;
    /**
     * Explicit app-data hash echoed alongside full app data, present only
     * when the orderbook response carried both forms. Mirrors the optional
     * `OrderParameters.appDataHash` wire field.
     */
    appDataHash?: AppDataHash;
    /**
     * Network-cost amount echoed by the orderbook `/quote` response.
     *
     * Stored under the upstream wire name `feeAmount` so the deterministic
     * JSON schema stays aligned with the services contract; consumers read
     * the value through [`QuoteData::network_cost_amount`] and configure it
     * through [`QuoteData::with_network_cost_amount`].
     */
    feeAmount: Amount;
    /**
     * Order kind.
     */
    kind: OrderKind;
    /**
     * Whether partial fills are allowed. Always serialized on the quote
     * response, so the wire field is required rather than defaulted; inbound
     * deserialization still tolerates its absence through the wire shim below.
     */
    partiallyFillable: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: SellTokenSource;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: BuyTokenDestination;
    /**
     * Estimated gas units for the quoted trade, in the upstream
     * decimal-string wire shape. Read-only quote estimate populated from the
     * orderbook `/quote` response (ADR 0021); empty for a locally constructed
     * quote. Read through [`QuoteData::gas_amount`].
     */
    gasAmount?: string;
    /**
     * Estimated gas price at quote time (wei per gas unit), in the upstream
     * decimal-string wire shape. Read-only quote estimate (ADR 0021); read
     * through [`QuoteData::gas_price`].
     */
    gasPrice?: string;
    /**
     * Sell-token price in native-token atoms per sell-token atom, in the
     * upstream decimal-string wire shape. Read-only quote estimate
     * (ADR 0021); read through [`QuoteData::sell_token_price`].
     */
    sellTokenPrice?: string;
    /**
     * Signing scheme for the quoted order. Mirrors
     * `OrderParameters.signingScheme`, which defaults to `eip712`. Read-only
     * quote field (ADR 0021); read through [`QuoteData::signing_scheme`].
     */
    signingScheme?: SigningScheme;
}

/**
 * Quote response DTO returned by `/api/v1/quote`.
 */
export interface OrderQuoteResponse {
    /**
     * Resolved quote payload.
     */
    quote: QuoteData;
    /**
     * Effective owner used for the quote, when returned by the API.
     */
    from?: Address;
    /**
     * Quote price/fee expiry as the orderbook\'s ISO-8601 UTC string (for
     * example `2026-04-28T10:00:00Z`), exposed losslessly.
     *
     * cow-rs intentionally takes no datetime dependency; parse this with your
     * preferred datetime crate (`chrono::DateTime::parse_from_rfc3339`,
     * `time::OffsetDateTime::parse`, ...) when a typed value is needed. This is
     * when the quoted price and fee expire; the eventual order\'s validity is
     * the [`QuoteData::valid_to`] UNIX epoch on `quote`.
     */
    expiration: string;
    /**
     * Quote identifier used when submitting the corresponding order.
     */
    id?: number;
    /**
     * Whether the quote was verified by the orderbook.
     */
    verified: boolean;
    /**
     * Optional protocol fee basis points for the quote.
     */
    protocolFeeBps?: string;
}

/**
 * Rate-limiter bucket scope accepted by JS client constructors.
 */
export type LimiterScopeConfig = "global" | "perHost";

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
 * Request-rate limiter override accepted by JS client constructors.
 */
export interface RequestRateLimiterConfig {
    /**
     * Request tokens granted per interval. Zero disables limiting.
     */
    tokensPerInterval?: number;
    /**
     * Limiter interval in milliseconds.
     */
    intervalMs?: number;
    /**
     * Bucket scope.
     */
    scope?: LimiterScopeConfig;
}

/**
 * Result returned after submitting a trade or transaction-producing flow.
 */
export interface OrderPostingResult {
    /**
     * Final order UID.
     */
    orderId: OrderUid;
    /**
     * Settlement transaction hash when the flow submits an on-chain
     * transaction directly (32-byte `0x`-prefixed hex string).
     *
     * Spelled as a viem-compatible `0x`-prefixed hex string on the TypeScript
     * boundary: the native `TransactionHash` alias of `Hash32` is not emitted as
     * a declaration, so the override pins the protocol-canonical `0x`-prefixed
     * hex wire form (the same idiom the orderbook `Trade.tx_hash` field uses).
     */
    txHash?: `0x${string}`;
    /**
     * Signature scheme used for the posted order.
     */
    signingScheme: SigningScheme;
    /**
     * Signature payload sent to the orderbook, or empty string for transaction-only flows.
     */
    signature: string;
    /**
     * Unsigned order payload used for signing or transaction generation.
     */
    orderToSign: OrderData;
}

/**
 * Retry-policy override accepted by JS client constructors.
 */
export interface RetryPolicyConfig {
    /**
     * Maximum attempts, including the initial request.
     */
    maxAttempts?: number;
    /**
     * Base exponential-backoff delay in milliseconds.
     */
    baseDelayMs?: number;
    /**
     * Maximum exponential-backoff delay in milliseconds.
     */
    maxDelayMs?: number;
}

/**
 * Runtime binding captured from an orderbook client for quote-derived workflows.
 */
export interface OrderbookBinding {
    /**
     * Chain id fixed by the orderbook client. Rendered as `number` on the
     * TypeScript boundary because [`CoreSupportedChainId`] serializes as the
     * numeric chain id.
     */
    chainId: number;
    /**
     * Environment fixed by the orderbook client. Spelled `CowEnv` on the
     * TypeScript boundary; the orderbook crate imports it under the
     * `CoreCowEnv` alias, which is not the emitted declaration name.
     */
    env: CowEnv;
    /**
     * Resolved base URL used by the orderbook client when it is available.
     */
    resolvedBaseUrl?: string;
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
 * Settlement candidate nested inside solver-competition responses.
 */
export interface SolverSettlement {
    /**
     * Address the solver used to execute the settlement on-chain.
     */
    solverAddress: Address;
    /**
     * Settlement score.
     */
    score: Amount;
    /**
     * Position of this solution in the competition ranking.
     */
    ranking: number;
    /**
     * Clearing prices keyed by token address.
     */
    clearingPrices?: Record<string, string>;
    /**
     * Orders touched by this solution.
     */
    orders?: SolverCompetitionOrder[];
    /**
     * Whether this solution won the right to be executed.
     */
    isWinner: boolean;
    /**
     * Whether this solution was filtered out by the competition rules.
     */
    filteredOut: boolean;
    /**
     * Reference score for this solution, when available.
     */
    referenceScore?: Amount;
    /**
     * Transaction in which the solution was executed on-chain, when available.
     */
    txHash?: `0x${string}`;
}

/**
 * Signature scheme encoded in orderbook wire DTOs.
 */
export type SigningScheme = "eip712" | "ethsign" | "eip1271" | "presign";

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
 * Signed order-cancellation DTO.
 */
export interface SignedCancellations {
    /**
     * Order UIDs to cancel.
     */
    orderUids: string[];
    /**
     * Cancellation signature.
     */
    signature: string;
    /**
     * ECDSA signing scheme.
     */
    signingScheme: string;
}

/**
 * Smart-contract interaction payload used by order pre and post hooks.
 */
export interface InteractionData {
    /**
     * Contract address targeted by the interaction.
     */
    target: Address;
    /**
     * Native token value sent with the interaction.
     */
    value: Amount;
    /**
     * Hex-encoded calldata forwarded to `target`.
     */
    callData: string;
}

/**
 * Solver execution entry nested inside competition-status responses.
 */
export interface SolverExecution {
    /**
     * Solver identifier or address rendered by the API.
     */
    solver: string;
    /**
     * Executed amounts for this solver path, when present.
     */
    executedAmounts?: ExecutedAmounts;
}

/**
 * Solver-competition response returned by the orderbook.
 */
export interface SolverCompetitionResponse {
    /**
     * Identifier of the auction this competition is for.
     */
    auctionId: number;
    /**
     * Block the auction started on.
     */
    auctionStartBlock: number;
    /**
     * Block deadline by which the auction must be settled.
     */
    auctionDeadlineBlock: number;
    /**
     * Transaction hashes for the winning solutions of this competition.
     */
    transactionHashes?: string[];
    /**
     * Reference score for each winning solver, keyed by solver address.
     */
    referenceScores?: Record<string, string>;
    /**
     * Auction snapshot for the competition.
     */
    auction: CompetitionAuction;
    /**
     * Settlement candidates submitted by solvers.
     */
    solutions?: SolverSettlement[];
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
 * Stepwise quote amounts and cost components across the quote lifecycle.
 */
export interface QuoteAmountsAndCosts<T> {
    /**
     * Whether the source quote was sell-sided.
     */
    isSell: boolean;
    /**
     * Cost breakdown for the quote.
     */
    costs: Costs<T>;
    /**
     * Amounts before all fees.
     */
    beforeAllFees: Amounts<T>;
    /**
     * Amounts before network costs.
     */
    beforeNetworkCosts: Amounts<T>;
    /**
     * Amounts after protocol fees.
     */
    afterProtocolFees: Amounts<T>;
    /**
     * Amounts after network costs.
     */
    afterNetworkCosts: Amounts<T>;
    /**
     * Amounts after partner fees.
     */
    afterPartnerFees: Amounts<T>;
    /**
     * Amounts after slippage.
     */
    afterSlippage: Amounts<T>;
    /**
     * Amounts that should be signed.
     */
    amountsToSign: Amounts<T>;
}

/**
 * Supported `CoW` deployment environments.
 *
 * Downstream crates should include a wildcard arm when matching so future
 * deployment environments remain semver-compatible.
 */
export type CowEnv = "prod" | "staging";

/**
 * Swap-style trade request accepted by quote and post helpers.
 */
export interface TradeParams {
    /**
     * Order kind.
     */
    kind: OrderKind;
    /**
     * Optional owner override. Signer address becomes the fallback in signer-backed flows.
     */
    owner?: Address;
    /**
     * Sell-token address.
     */
    sellToken: Address;
    /**
     * Buy-token address.
     */
    buyToken: Address;
    /**
     * Amount interpreted according to `kind`.
     */
    amount: Amount;
    /**
     * Optional environment override for endpoint and contract resolution.
     */
    env?: CowEnv;
    /**
     * Optional settlement contract overrides keyed by chain id. Typed as
     * `Record` rather than `Map` on the TypeScript boundary because the runtime
     * serializer emits a plain JavaScript object for the `BTreeMap`.
     */
    settlementContractOverride?: Record<string, string>;
    /**
     * Optional `EthFlow` contract overrides keyed by chain id.
     */
    ethFlowContractOverride?: Record<string, string>;
    /**
     * Whether partial fills are allowed. Defaults to `false` when omitted on the
     * input boundary; always serialized on a resolved `QuoteResults.tradeParameters`.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source. Defaults to `erc20` when omitted on the input
     * boundary; always serialized through quote and post flows.
     */
    sellTokenBalance?: SellTokenSource;
    /**
     * Buy-token balance destination. Defaults to `erc20` when omitted on the input
     * boundary; always serialized through quote and post flows.
     */
    buyTokenBalance?: BuyTokenDestination;
    /**
     * Optional explicit slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: Address;
    /**
     * Optional relative validity duration in seconds.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata merged into app-data and fee calculations.
     */
    partnerFee?: PartnerFee;
}

/**
 * Total-surplus response from `/api/v1/users/{owner}/total_surplus`.
 */
export interface TotalSurplus {
    /**
     * Total surplus value in the upstream decimal-string wire shape,
     * denominated in the chain\'s native-token base units (wei, 18 decimals).
     */
    totalSurplus?: Amount;
}

/**
 * Trade DTO returned by the orderbook trades endpoint.
 */
export interface Trade {
    /**
     * Block number containing the trade event.
     */
    blockNumber: number;
    /**
     * Log index within the block.
     */
    logIndex: number;
    /**
     * Order UID associated with the trade.
     */
    orderUid: OrderUid;
    /**
     * Owner address.
     */
    owner: Address;
    /**
     * Sell-token address.
     */
    sellToken: Address;
    /**
     * Buy-token address.
     */
    buyToken: Address;
    /**
     * Executed sell amount in the upstream decimal-string wire shape.
     */
    sellAmount: Amount;
    /**
     * Executed sell amount before fees. Always serialized on the trade
     * response, so the wire field is required rather than defaulted.
     */
    sellAmountBeforeFees: Amount;
    /**
     * Executed buy amount in the upstream decimal-string wire shape.
     */
    buyAmount: Amount;
    /**
     * Protocol fees executed as part of the trade, when services returns them.
     */
    executedProtocolFees?: ExecutedProtocolFee[];
    /**
     * Settlement transaction hash.
     */
    txHash: `0x${string}`;
}

/**
 * Trades query accepted by `OrderBookClient.getTrades`.
 */
export interface GetTradesRequest {
    /**
     * Owner filter. Set exactly one of `owner` or `orderUid`.
     */
    owner?: string;
    /**
     * Order UID filter. Set exactly one of `owner` or `orderUid`.
     */
    orderUid?: string;
    /**
     * Pagination offset.
     */
    offset?: number;
    /**
     * Pagination limit.
     */
    limit?: number;
}

/**
 * Transaction request shape used across signer and provider traits.
 */
export interface TransactionRequest {
    /**
     * Destination address for the transaction.
     */
    to?: Address;
    /**
     * Hex-encoded calldata payload.
     */
    data?: HexData;
    /**
     * Native token value to transfer.
     */
    value?: Amount;
    /**
     * Optional gas limit override.
     */
    gasLimit?: Amount;
}

/**
 * Transport-policy override accepted by JS client constructors.
 */
export interface TransportPolicyConfig {
    /**
     * Retry-policy override.
     */
    retryPolicy?: RetryPolicyConfig;
    /**
     * Rate-limiter override.
     */
    requestRateLimiter?: RequestRateLimiterConfig;
    /**
     * Retry jitter override.
     */
    jitterStrategy?: JitterStrategyConfig;
    /**
     * Optional transport user-agent value.
     */
    userAgent?: string;
}

/**
 * Typed contract-read request used by runtime-neutral providers.
 */
export interface ContractCall {
    /**
     * Target contract address.
     */
    address: Address;
    /**
     * ABI method name to invoke.
     */
    method: string;
    /**
     * JSON ABI fragment describing the contract or function.
     */
    abiJson: string;
    /**
     * JSON-encoded function arguments.
     */
    argsJson: string;
}

/**
 * Typed partner-fee metadata accepted by app-data and trading helpers.
 */
export type PartnerFee = PartnerFeePolicy | PartnerFeePolicy[];

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
 * `EthFlow`-specific orderbook metadata.
 */
export interface EthflowData {
    /**
     * Transaction in which the order was refunded, when present.
     */
    refundTxHash?: `0x${string}`;
    /**
     * User-facing validity timestamp for the `EthFlow` order.
     */
    userValidTo: number;
}


/**
 * IPFS app-data client backed by an explicitly configured HTTP transport.
 *
 * Construct this client when JavaScript needs to fetch app-data documents by
 * CID or app-data hash while preserving SDK retry, timeout, and cancellation
 * behavior.
 */
export class IpfsClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches and parses an app-data document by CID.
     *
     * The CID is resolved through the configured gateway and transport. The
     * returned document is normalized into the SDK app-data DTO shape.
     *
     * @param cid Canonical IPFS CID for the app-data document.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the app-data document.
     * @throws CowError for invalid CID, transport failure, timeout, or parse failure.
     */
    fetchAppDataFromCid(cid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataDocument>>;
    /**
     * Fetches and parses an app-data document by app-data hash.
     *
     * The helper converts the app-data hash to the canonical CID before
     * fetching through the configured gateway.
     *
     * @param appDataHex App-data hash as a `0x`-prefixed hex string.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the app-data document.
     * @throws CowError for invalid hash, transport failure, timeout, or parse failure.
     */
    fetchAppDataFromHex(appDataHex: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataDocument>>;
    /**
     * Creates an IPFS app-data client from a single config object.
     *
     * The config must include `transport`. Optional `ipfsUri` overrides the
     * default gateway base, while timeout, signal, and policy fields become
     * defaults for method calls.
     *
     * @param config IPFS client configuration.
     * @throws CowError when transport, policy, timeout, or gateway config is invalid.
     */
    constructor(config: IpfsClientConfig);
}

/**
 * Orderbook client backed by an explicitly configured HTTP transport.
 *
 * Construct this client when JavaScript needs direct access to quote,
 * submission, lookup, trade, native-price, app-data, and cancellation orderbook
 * endpoints. The client owns one callback registration and releases raw wasm
 * resources through the facade `dispose()` method.
 */
export class OrderBookClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Submits signed off-chain order cancellations.
     *
     * Build the signed cancellation payload with one of the cancellation
     * signing helpers, then submit it through the same orderbook runtime
     * configuration used for order operations.
     *
     * @param signed Signed cancellation payload.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing `{ cancelled: true }` on success.
     * @throws CowError for invalid UID, signature, transport failure, or timeout.
     */
    cancelOrders(signed: SignedCancellations, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ cancelled: true }>>;
    /**
     * Fetches the full app-data document registered for an app-data hash.
     *
     * Use this to retrieve the canonical app-data payload the orderbook holds
     * for a given hash, for example to display or re-verify a document
     * referenced by an order.
     *
     * @param appDataHash App-data hash as a `0x`-prefixed 32-byte hex string.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the app-data document.
     * @throws CowError for an invalid hash, transport failure, or timeout.
     */
    getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataObject>>;
    /**
     * Fetches a token's native price from the orderbook API.
     *
     * The token must be an EVM address. The returned value follows the
     * orderbook native-price response shape.
     *
     * @param token Token address to price.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing native price data.
     * @throws CowError for invalid token address, transport failure, or timeout.
     */
    getNativePrice(token: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<NativePriceResponse>>;
    /**
     * Fetches one order by its canonical order UID.
     *
     * The UID must be the full 56-byte CoW order UID encoded as a `0x`-prefixed
     * string. The response is returned in the orderbook wire DTO shape.
     *
     * @param orderUid Full order UID to look up.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the order response.
     * @throws CowError for invalid UID, not-found responses, transport failure, or timeout.
     */
    getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order>>;
    /**
     * Fetches the live competition status for one order.
     *
     * Returns the order's status in the current or most recent solver
     * competition, including any per-solver executed amounts the service
     * reports. The UID must be the full 56-byte CoW order UID.
     *
     * @param orderUid Full order UID to look up.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the competition status.
     * @throws CowError for invalid UID, not-found responses, transport failure, or timeout.
     */
    getOrderCompetitionStatus(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<CompetitionOrderStatus>>;
    /**
     * Builds the orderbook API URL (`/api/v1/orders/{uid}`) for a UID without
     * any network call.
     *
     * This is the canonical machine-readable order handle, not the human-facing
     * CoW Explorer page; build the explorer URL in the application.
     *
     * @param orderUid Full order UID to link to.
     * @returns A versioned envelope containing the orderbook API URL for the order.
     * @throws CowError for an invalid UID or an unresolved base URL.
     */
    getOrderLink(orderUid: string): WasmEnvelope<string>;
    /**
     * Fetches an order by UID, falling back across environments on a 404.
     *
     * @param orderUid Full order UID to look up.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the order response.
     * @throws CowError for invalid UID, not-found responses, transport failure, or timeout.
     */
    getOrderMultiEnv(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order>>;
    /**
     * Fetches orders owned by an address with optional pagination.
     *
     * The owner address is validated before the request is dispatched. The
     * response preserves the typed orderbook order shape. When `pagination` is
     * omitted the request sends the upstream default `limit` of 1000, so an
     * account with more orders is truncated unless an explicit page is set.
     *
     * @param owner Owner address to query.
     * @param pagination Optional offset and limit; defaults to `limit` 1000 when omitted.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching orders.
     * @throws CowError for invalid owner, transport failure, timeout, or cancellation.
     */
    getOrders(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order[]>>;
    /**
     * Fetches a price quote from the orderbook API.
     *
     * The request is converted to the typed orderbook quote request and sent
     * through the configured transport. Per-call options can override the
     * constructor timeout or attach an `AbortSignal`.
     *
     * This returns the raw `OrderQuoteResponse`, distinct from
     * `TradingClient.getQuote`, which returns the richer `QuoteResults`
     * carrying `orderToSign` and `amountsAndCosts` for posting.
     *
     * @param request Quote request DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the raw quote response.
     * @throws CowError for invalid input, transport failure, timeout, or cancellation.
     */
    getQuote(request: OrderQuoteRequest, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderQuoteResponse>>;
    /**
     * Fetches the solver-competition result for an auction.
     *
     * Returns the solver competition the protocol ran for the auction: the
     * winning solvers, their scores and rankings, the auction snapshot, and the
     * per-solver settlements, in the upstream wire shape. Targets the v2
     * `/api/v2/solver_competition/{auctionId}` route.
     *
     * @param auctionId Auction id to look up (a non-negative integer).
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the solver-competition response.
     * @throws CowError for an out-of-range id, not-found responses, transport failure, or timeout.
     */
    getSolverCompetition(auctionId: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<SolverCompetitionResponse>>;
    /**
     * Fetches the solver-competition result by settlement transaction hash.
     *
     * Like `getSolverCompetition`, keyed by the settlement transaction hash
     * rather than the auction id. Targets the v2
     * `/api/v2/solver_competition/by_tx_hash/{txHash}` route.
     *
     * @param txHash Settlement transaction hash as a `0x`-prefixed 32-byte hex string.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the solver-competition response.
     * @throws CowError for an invalid hash, not-found responses, transport failure, or timeout.
     */
    getSolverCompetitionByTxHash(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<SolverCompetitionResponse>>;
    /**
     * Fetches the total accumulated surplus for an account.
     *
     * Returns the lifetime surplus the protocol has captured for the owner
     * across its settled orders, in the upstream decimal-string wire shape.
     * The value is denominated in the chain's native-token base units (wei,
     * 18 decimals), not USD or sell-token atoms.
     *
     * @param owner Owner address to query.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the total-surplus response in native-token wei.
     * @throws CowError for invalid owner, transport failure, or timeout.
     */
    getTotalSurplus(owner: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<TotalSurplus>>;
    /**
     * Fetches trades for exactly one owner address or order UID.
     *
     * The query must set one of `owner` or `orderUid`, not both. Optional
     * pagination fields are forwarded to the orderbook request.
     *
     * @param query Trade query DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching trades.
     * @throws CowError when the query is ambiguous or transport fails.
     */
    getTrades(query: GetTradesRequest, options?: SdkClientOptions | null): Promise<WasmEnvelope<Trade[]>>;
    /**
     * Fetches the orders contained in a settlement transaction.
     *
     * @param txHash Settlement transaction hash.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the settled orders.
     * @throws CowError for an invalid hash, transport failure, timeout, or cancellation.
     */
    getTxOrders(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<Order[]>>;
    /**
     * Fetches the orderbook service version string.
     *
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the service version string.
     * @throws CowError for transport failure, timeout, or cancellation.
     */
    getVersion(options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Creates an orderbook client from a single config object.
     *
     * The config must include `chainId` and `transport`. The optional
     * `timeoutMs`, `signal`, and `transportPolicy` fields become defaults for
     * calls made through this client unless a method call overrides them.
     *
     * @param config Orderbook client configuration.
     * @throws CowError when the chain, environment, transport, or policy is invalid.
     */
    constructor(config: OrderBookClientConfig);
    /**
     * Submits a signed order to the orderbook.
     *
     * The signed DTO normally comes from a signing helper in the same package.
     * The SDK reconstructs the typed order creation payload and returns the
     * order UID assigned by the orderbook service.
     *
     * @param signed Signed order DTO including typed data, signature, owner, and scheme.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the submitted order UID.
     * @throws CowError for invalid signatures, transport failure, timeout, or rejection.
     */
    sendOrder(signed: SignedOrder, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Submits a raw order-creation payload to the orderbook.
     *
     * Use this method when the host already has a complete orderbook
     * `OrderCreation` shape and does not need the facade to reconstruct it
     * from a signed-order DTO.
     *
     * @param input Raw order-creation DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the submitted order UID.
     * @throws CowError for malformed input, transport failure, timeout, or rejection.
     */
    sendOrderCreation(input: OrderCreation, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Uploads the full app-data JSON for a content-addressed app-data hash.
     *
     * The SDK enforces the content-addressed-write invariant: the keccak-256
     * digest of `fullAppData` must equal `appDataHash`, or the call rejects
     * before any network request. Serialize `fullAppData` with the canonical
     * app-data writer so the digest matches.
     *
     * @param appDataHash App-data hash as a `0x`-prefixed 32-byte hex string.
     * @param fullAppData Canonically serialized app-data JSON payload.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing `{ uploaded: true }` on success.
     * @throws CowError for a hash mismatch, invalid hash, transport failure, or timeout.
     */
    uploadAppData(appDataHash: string, fullAppData: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ uploaded: true }>>;
}

/**
 * Read-only subgraph client backed by an explicitly configured transport.
 *
 * Construct this client when JavaScript needs protocol totals, recent volume,
 * or custom GraphQL query execution through the same transport and policy
 * model as the orderbook clients.
 */
export class SubgraphClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches recent daily volume rows.
     *
     * The `days` value controls how many recent daily buckets the subgraph
     * query requests.
     *
     * @param days Number of daily buckets to fetch.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing daily volume rows.
     * @throws CowError for invalid query shape, transport failure, or timeout.
     */
    getLastDaysVolume(days: number, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches recent hourly volume rows.
     *
     * The `hours` value controls how many recent hourly buckets the subgraph
     * query requests.
     *
     * @param hours Number of hourly buckets to fetch.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing hourly volume rows.
     * @throws CowError for invalid query shape, transport failure, or timeout.
     */
    getLastHoursVolume(hours: number, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches aggregate protocol totals from the subgraph.
     *
     * The request uses the client's configured chain, API key, transport, and
     * transport policy.
     *
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing aggregate totals.
     * @throws CowError for transport, cancellation, timeout, or subgraph errors.
     */
    getTotals(options?: SdkClientOptions | null): Promise<any>;
    /**
     * Creates a subgraph client from a single config object.
     *
     * The config must include `chainId`, `apiKey`, and `transport`. Optional
     * timeout, signal, and policy fields become client defaults for later
     * method calls.
     *
     * @param config Subgraph client configuration.
     * @throws CowError when the chain, API key, transport, or policy is invalid.
     */
    constructor(config: SubgraphClientConfig);
    /**
     * Runs a caller-provided GraphQL query against the configured subgraph.
     *
     * Use this method when the built-in totals or volume helpers are too
     * narrow. `variables` and `operationName` are forwarded when provided.
     *
     * @param query Raw GraphQL document to execute.
     * @param variables Optional GraphQL variables object.
     * @param operationName Optional operation name for a multi-operation document.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the JSON GraphQL response.
     * @throws CowError for invalid variables, transport, timeout, cancellation, or GraphQL errors.
     */
    runQuery(query: string, variables: Value, operationName?: string | null, options?: SdkClientOptions | null): Promise<any>;
}

/**
 * High-level trading client backed by an explicitly configured orderbook.
 *
 * Construct this client when JavaScript needs quote, sign, post, allowance,
 * and native-sell helper workflows rather than direct orderbook calls. The
 * client keeps app-code, chain, environment, transport, and policy defaults.
 */
export class TradingClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Builds the ERC-20 approval transaction for the CoW Protocol vault relayer.
     *
     * The SDK encodes the unsigned `approve` transaction; the JavaScript host
     * owns submission through its own wallet. This completes the
     * read-allowance-then-approve path alongside `getCowProtocolAllowance`.
     *
     * @param params Approval parameters DTO (token, amount, optional vault-relayer override).
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the unsigned approval transaction request.
     * @throws CowError when the token, amount, or vault-relayer override is invalid.
     */
    buildApprovalTx(params: ApprovalParams, options?: SdkClientOptions | null): Promise<WasmEnvelope<TransactionRequest>>;
    /**
     * Builds the transaction for a native-currency sell order.
     *
     * The helper validates that the order sells the native-token sentinel,
     * resolves the EthFlow deployment, and returns a transaction request for
     * the host wallet to submit.
     *
     * @param order Unsigned native-sell order DTO.
     * @param quoteId Quote identifier returned by the orderbook.
     * @param from Transaction sender address.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing order UID and transaction request.
     * @throws CowError when the order, chain, deployment, or sender is invalid.
     */
    buildSellNativeCurrencyTx(order: OrderData, quoteId: number, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<BuiltSellNativeCurrencyTx>>;
    /**
     * Builds the native-currency sell transaction directly from a quote result.
     *
     * This is the native-sell sibling of `postSwapOrderFromQuote`: it consumes
     * the `QuoteResults` that `getQuote` returns for a native-currency sell
     * and derives the EthFlow transaction without the host reconstructing the
     * order or extracting the quote id. The quote must have been requested with
     * the native-token sentinel as the sell token and must carry the quote id
     * the orderbook returns for EthFlow submission.
     *
     * @param quoteResults Quote result DTO returned by `getQuote` for a native sell.
     * @param from Transaction sender address.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing order UID and transaction request.
     * @throws CowError when the quote is not a native-currency sell, lacks a quote id, or the chain, deployment, or sender is invalid.
     */
    buildSellNativeCurrencyTxFromQuote(quoteResults: QuoteResults, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<BuiltSellNativeCurrencyTx>>;
    /**
     * Builds the transaction that unwraps the wrapped-native token back into
     * native currency (for example WETH into ETH) on this client's chain.
     *
     * `withdraw` burns the caller's own wrapped-native balance, so no token
     * approval is required. Submit the returned request with the host wallet.
     *
     * @param amount Amount of the wrapped-native token to unwrap, in wei as a decimal string.
     * @returns A versioned envelope containing the unsigned unwrap transaction request.
     * @throws CowError when the chain is unsupported or the amount is invalid.
     */
    buildUnwrapTx(amount: string): WasmEnvelope<TransactionRequest>;
    /**
     * Builds the transaction that wraps native currency into its wrapped-native
     * token (for example ETH into WETH) on this client's chain.
     *
     * The target wrapped-native address is resolved from the chain; submit the
     * returned request with the host wallet. Selling native currency through CoW
     * Protocol does not require a manual wrap — the eth-flow path wraps on-chain
     * during order creation — so use this for standalone wrap and treasury flows.
     *
     * @param amount Amount of native currency to wrap, in wei as a decimal string.
     * @returns A versioned envelope containing the unsigned wrap transaction request.
     * @throws CowError when the chain is unsupported or the amount is invalid.
     */
    buildWrapTx(amount: string): WasmEnvelope<TransactionRequest>;
    /**
     * Reads CoW Protocol allowance through a read-only contract callback.
     *
     * The SDK builds the contract call while the JavaScript host performs the
     * actual chain read. Use this when a TypeScript runtime owns the RPC
     * provider. The vault-relayer spender is resolved per chain and environment
     * unless overridden in the parameters. The callback must return the
     * ABI-decoded `uint256` allowance as a decimal string or JSON number — for
     * example viem's `readContract` result passed through `String(value)` — not
     * a raw `0x`-hex `eth_call` payload.
     *
     * @param params Allowance parameters DTO.
     * @param readContractCallback Callback that executes the read-only call and returns the ABI-decoded allowance.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the allowance amount as a decimal string.
     * @throws CowError for invalid parameters, callback failure, timeout, or cancellation.
     */
    getCowProtocolAllowance(params: AllowanceParams, readContractCallback: ContractReadCallback, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Fetches a quote without signing or submitting an order.
     *
     * Use this method when a host wants to preview the quote response before
     * asking a wallet to sign or before constructing a post request. Set
     * `owner` on the swap parameters: quote-only flows resolve no signer, so a
     * missing owner surfaces as an error rather than defaulting to an account.
     *
     * This returns the rich `QuoteResults` carrying `orderToSign` and
     * `amountsAndCosts` for posting, distinct from `OrderBookClient.getQuote`,
     * which returns the raw orderbook `OrderQuoteResponse`.
     *
     * @param params Swap parameters DTO; set `owner` for quote-only flows.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the rich quote results.
     * @throws CowError for a missing owner, invalid parameters, transport failure, timeout, or cancellation.
     */
    getQuote(params: TradeParams, options?: SdkClientOptions | null): Promise<WasmEnvelope<QuoteResults>>;
    /**
     * Creates a trading client from a single config object.
     *
     * The config must include `chainId`, `appCode`, and `transport`. Optional
     * environment, API key, timeout, signal, and transport policy fields become
     * defaults for all trading methods. When constructed through the TypeScript
     * facade, an omitted `transport` defaults to the runtime global `fetch`;
     * that default is a facade affordance, so the raw constructor documented
     * here requires the transport explicitly.
     *
     * @param config Trading client configuration.
     * @throws CowError when chain, app-code, environment, transport, or policy validation fails.
     */
    constructor(config: TradingClientConfig);
    /**
     * Signs and posts a limit order through a typed-data callback.
     *
     * This helper follows the native limit-order trading path and lets the SDK
     * build, sign, and submit the order using the configured orderbook.
     *
     * @param params Limit-order parameters DTO.
     * @param owner Owner address to bind to the order when absent from params.
     * @param signerCallback Callback that signs the typed-data envelope.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws CowError for invalid input, wallet failure, timeout, or rejection.
     */
    postLimitOrder(params: LimitTradeParams, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResult>>;
    /**
     * Quotes, signs, and posts a swap order through a typed-data callback.
     *
     * The SDK fetches a quote, builds the order to sign, invokes the callback
     * with the EIP-712 envelope, posts the signed order, and returns posting
     * output from the trading workflow.
     *
     * @param params Swap parameters DTO.
     * @param owner Owner address to bind to the order.
     * @param signerCallback Callback that signs the typed-data envelope.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws CowError for invalid input, quote failure, wallet failure, timeout, or rejection.
     */
    postSwapOrder(params: TradeParams, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResult>>;
    /**
     * Signs and posts a previously quoted swap order.
     *
     * Use this method when a host has already called `getQuote` and wants to
     * reuse that quote result for posting without requesting a new quote.
     *
     * @param quoteResults Quote result DTO returned by `getQuote`.
     * @param owner Owner address to bind to the order.
     * @param signerCallback Callback that signs the typed-data envelope.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws CowError for invalid quote data, wallet failure, timeout, or rejection.
     */
    postSwapOrderFromQuote(quoteResults: QuoteResults, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResult>>;
    /**
     * Quotes and posts a swap order with a custom EIP-1271 signature callback.
     *
     * Use this method when a smart-account runtime owns final contract
     * signature production. The SDK still quotes the swap, builds typed data,
     * posts the signed order, and returns posting output.
     *
     * @param params Swap parameters DTO.
     * @param owner Smart-account owner address.
     * @param customCallback Callback that returns the final EIP-1271 signature.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws CowError for invalid input, quote failure, callback failure, timeout, or rejection.
     */
    postSwapOrderWithEip1271(params: TradeParams, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResult>>;
}

/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init(): void;

/**
 * Builds a normalized app-data document without deriving storage metadata.
 *
 * This helper is useful when a host wants to inspect or modify the canonical
 * document shape before separately deriving app-data information.
 *
 * @param doc App-data document input accepted by the SDK schema.
 * @returns A versioned envelope containing the normalized document.
 * @throws CowError when the input cannot be normalized.
 */
export function appDataDoc(doc: AppDataParams): WasmEnvelope<AppDataDocument>;

/**
 * Converts a `0x`-prefixed app-data hash into the canonical IPFS CID.
 *
 * The conversion is pure and uses the same app-data multicodec and multihash
 * rules as the Rust app-data crate.
 *
 * @param appDataHex App-data hash as a `0x`-prefixed hex string.
 * @returns A versioned envelope containing the CID string.
 * @throws CowError when the hash is malformed.
 */
export function appDataHexToCid(appDataHex: string): WasmEnvelope<string>;

/**
 * Builds app-data content and returns its deterministic hash and CID.
 *
 * Use this when a JavaScript host wants the SDK to construct the canonical
 * document and expose the values needed for order submission and storage.
 *
 * @param doc App-data document input accepted by the SDK schema.
 * @returns A versioned envelope containing document, hash, CID, and hex data.
 * @throws CowError when the document cannot be normalized or hashed.
 */
export function appDataInfo(doc: AppDataParams): WasmEnvelope<AppDataInfo>;

/**
 * Builds a settlement cancellation transaction for an order UID.
 *
 * The returned transaction request targets the Settlement contract and encodes
 * `invalidateOrder(bytes)`. The host wallet remains responsible for submitting
 * and observing the transaction.
 *
 * @param params Order UID, chain, environment, and optional deployment override.
 * @returns A versioned envelope containing the transaction request DTO.
 * @throws CowError when the chain, deployment, or order UID is invalid.
 */
export function buildCancelOrderTx(params: OrderTraderParams): WasmEnvelope<TransactionRequest>;

/**
 * Builds a settlement pre-sign transaction for an order UID.
 *
 * The returned transaction request targets the Settlement contract and encodes
 * `setPreSignature(bytes,bool)` with the order UID and `true` flag. The host
 * wallet remains responsible for transaction submission.
 *
 * @param params Order UID, chain, environment, and optional deployment override.
 * @returns A versioned envelope containing the transaction request DTO.
 * @throws CowError when the chain, deployment, or order UID is invalid.
 */
export function buildPresignTx(params: OrderTraderParams): WasmEnvelope<TransactionRequest>;

/**
 * Converts a canonical IPFS CID into a `0x`-prefixed app-data hash.
 *
 * Use this helper when an order or metadata path starts from a CID but the
 * orderbook request needs the app-data hash form.
 *
 * @param cid Canonical CID string for an app-data document.
 * @returns A versioned envelope containing the `0x`-prefixed hash.
 * @throws CowError when the CID does not match the supported app-data shape.
 */
export function cidToAppDataHex(cid: string): WasmEnvelope<string>;

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
 * Signs a cancellation digest through an explicit `eth_sign` callback.
 *
 * The SDK computes the canonical cancellation digest for the provided UIDs and
 * passes it to the digest signer callback as a `0x`-prefixed string.
 *
 * @param orderUids One or more full order UIDs to cancel.
 * @param chainId EVM chain id used for the cancellation digest.
 * @param digestSigner Callback that signs the digest string.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing signed cancellations.
 * @throws CowError for empty input, invalid UID, callback failure, or timeout.
 */
export function signCancellationEthSignDigest(orderUids: string[], chainId: number, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellations>>;

/**
 * Signs cancellation typed data through a typed-data callback.
 *
 * The SDK builds the batch cancellation EIP-712 payload for the provided order
 * UIDs and asks the callback to sign it. The response can be submitted through
 * `OrderBookClient.cancelOrders`.
 *
 * @param orderUids One or more full order UIDs to cancel.
 * @param chainId EVM chain id used for the cancellation domain.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing signed cancellations.
 * @throws CowError for empty input, invalid UID, callback failure, or timeout.
 */
export function signCancellationWithTypedDataSigner(orderUids: string[], chainId: number, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellations>>;

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
 * Validates an app-data document against the typed metadata contract.
 *
 * Validation is local and deterministic. The result reports whether the
 * document conforms and includes validation details without uploading data.
 *
 * @param doc App-data document input to validate.
 * @returns A versioned envelope containing the validation result.
 * @throws CowError when the input cannot be converted into a document.
 */
export function validateAppDataDoc(doc: AppDataParams): WasmEnvelope<ValidationResult>;

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
