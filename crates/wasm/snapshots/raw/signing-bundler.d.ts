/* tslint:disable */
/* eslint-disable */

export interface WalletConfig {
    timeoutMs?: number;
}

export interface SigningOptions extends SdkClientOptions {
    walletConfig?: WalletConfig;
}

export type TypedDataSignerCallback = (
envelope: TypedDataEnvelopeDto,
) => Promise<string> | string;

export type Eip1193RequestCallback = (
request: { method: string; params?: unknown[] },
) => Promise<unknown> | unknown;

export type DigestSignerCallback = (
digest: string,
) => Promise<string> | string;

export type CowEip1271SignCallback = (
request: CowEip1271SignRequest,
) => Promise<string> | string;

export type CustomEip1271Callback = CowEip1271SignCallback;



export type Value = unknown;
export type SdkError = WasmError;

export interface SdkClientOptions {
    timeoutMs?: number;
    signal?: AbortSignal;
}


/**
 * A decoded `GPv2Settlement` (or inherited `GPv2Signing`) event.
 *
 * Mirrors `cow_sdk_contracts::SettlementEvent`. Addresses and the order UID
 * are lowercase `0x`-prefixed hex; amounts are base-10 atom strings; the
 * interaction `selector` is a `0x`-prefixed 4-byte hex string. The `kind`
 * discriminator distinguishes the variants.
 */
export type SettlementEventDto = { kind: "trade"; owner: string; sellToken: string; buyToken: string; sellAmount: string; buyAmount: string; feeAmount: string; orderUid: string } | { kind: "interaction"; target: string; value: string; selector: string } | { kind: "settlement"; solver: string } | { kind: "orderInvalidated"; owner: string; orderUid: string } | { kind: "preSignature"; owner: string; orderUid: string; signed: boolean };

/**
 * A decoded eth-flow on-chain order lifecycle event.
 *
 * Mirrors `cow_sdk_contracts::EthFlowEvent`. The placement `order` reuses the
 * canonical [`OrderInput`] shape (its `validTo` is the on-chain clamped value;
 * the trader\'s real expiry travels in the opaque `data` trailer). `signature`
 * and `data` are `0x`-prefixed hex strings carrying the raw on-chain signature
 * payload and the opaque trailing data field; addresses and the order UID are
 * lowercase `0x`-prefixed hex. The `kind` discriminator distinguishes the
 * variants.
 */
export type EthFlowEventDto = { kind: "orderPlacement"; sender: string; order: OrderInput; signingScheme: string; signature: string; data: string } | { kind: "orderInvalidation"; orderUid: string } | { kind: "orderRefund"; orderUid: string; refunder: string };

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
     * Original order input.
     */
    order: OrderInput;
    /**
     * Typed-data envelope.
     */
    typedData: TypedDataEnvelopeDto;
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
 */
export interface DeploymentAddressesDto {
    /**
     * Settlement contract.
     */
    settlement: string;
    /**
     * Vault relayer contract.
     */
    vaultRelayer: string;
    /**
     * EthFlow contract.
     */
    ethFlow: string;
}

/**
 * EIP-1193 request DTO.
 */
export interface Eip1193Request {
    /**
     * Provider method.
     */
    method: string;
    /**
     * Provider params.
     */
    params?: Value[];
}

/**
 * Explicit raw GraphQL query input.
 */
export interface SubgraphQueryInput {
    /**
     * Raw GraphQL document.
     */
    query: string;
    /**
     * Optional GraphQL variables.
     */
    variables?: Value;
    /**
     * Optional operation name.
     */
    operationName?: string;
}

/**
 * Generated order UID output.
 */
export interface GeneratedOrderUidDto {
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
 * Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
 *
 * The wire form is the protocol-canonical `0x`-prefixed 66-character
 * lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
 * [`alloy_primitives::B256`] and forwards `Display`/`Serialize`/
 * `Deserialize` to the inner alloy type, whose canonical defaults already
 * emit the cow lowercase wire form.
 */
export type Hash32 = string;

/**
 * JS-visible typed error envelope for every wasm export.
 */
export type WasmError = { kind: "invalidInput"; schemaVersion: SchemaVersion; message: string; field?: string } | { kind: "unknownEnumValue"; schemaVersion: SchemaVersion; message: string; field: string; value: string } | { kind: "unsupportedChain"; schemaVersion: SchemaVersion; message: string; chainId: number } | { kind: "walletRequest"; schemaVersion: SchemaVersion; method: string; code?: number; message: string; data?: Value } | { kind: "walletTimeout"; schemaVersion: SchemaVersion; message: string; timeoutMs: number } | { kind: "transport"; schemaVersion: SchemaVersion; class: string; message: string; status?: number; headers?: [string, string][]; body?: string } | { kind: "orderbook"; schemaVersion: SchemaVersion; code?: string; category?: OrderBookRejectionCategoryDto; message: string; retryable?: boolean; retryAfterMs?: number } | { kind: "subgraph"; schemaVersion: SchemaVersion; message: string } | { kind: "signing"; schemaVersion: SchemaVersion; message: string } | { kind: "appData"; schemaVersion: SchemaVersion; class?: string; message: string } | { kind: "forbiddenInteraction"; schemaVersion: SchemaVersion; message: string; target: string; reason: string } | { kind: "cancelled"; schemaVersion: SchemaVersion; message: string } | { kind: "internal"; schemaVersion: SchemaVersion; message: string } | { kind: "__unknown"; schemaVersion: SchemaVersion; message: string; raw: Value };

/**
 * Order input shared by signing and UID exports.
 */
export interface OrderInput {
    /**
     * Sell token address.
     */
    sellToken: string;
    /**
     * Buy token address.
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
     * Valid-to timestamp.
     */
    validTo: number;
    /**
     * App-data hash.
     */
    appData: string;
    /**
     * Fee amount.
     */
    feeAmount: string;
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Partial fill flag.
     */
    partiallyFillable: boolean;
    /**
     * Sell balance source.
     */
    sellTokenBalance: TokenBalanceDto;
    /**
     * Buy balance destination.
     */
    buyTokenBalance: TokenBalanceDto;
}

/**
 * Order side accepted by wasm order inputs.
 */
export type OrderKindDto = "sell" | "buy";

/**
 * Order transaction helper parameters.
 */
export interface OrderTraderParametersInput {
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
 * Raw EVM event log accepted by the on-chain event decoders.
 *
 * `topics` carries the indexed log topics as `0x`-prefixed 32-byte hex
 * strings with topic-0 (the event signature hash) first; `data` is the
 * ABI-encoded non-indexed payload as a `0x`-prefixed hex string (`\"0x\"` for an
 * empty payload).
 */
export interface EventLogInput {
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
 * Signed order DTO returned by wallet callback exports.
 */
export interface SignedOrderDto {
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
    typedData: TypedDataEnvelopeDto;
    /**
     * Optional quote id.
     */
    quoteId?: number;
}

/**
 * Signed order-cancellation DTO.
 */
export interface SignedCancellationsInput {
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
 * Token-balance mode accepted by wasm order inputs.
 */
export type TokenBalanceDto = "erc20" | "external" | "internal";

/**
 * Trades query accepted by `OrderBookClient.getTrades`.
 */
export interface TradesQueryInput {
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
 * Transaction request DTO returned by transaction builders.
 */
export interface TransactionRequestDto {
    /**
     * Destination address.
     */
    to?: string;
    /**
     * Hex-encoded calldata.
     */
    data?: string;
    /**
     * Native value.
     */
    value?: string;
    /**
     * Gas limit.
     */
    gasLimit?: string;
}

/**
 * Typed-data domain DTO.
 */
export interface TypedDataDomainDto {
    /**
     * Domain name.
     */
    name: string;
    /**
     * Domain version.
     */
    version: string;
    /**
     * Chain id.
     */
    chainId: number;
    /**
     * Verifying contract.
     */
    verifyingContract: string;
}

/**
 * Typed-data envelope DTO.
 */
export interface TypedDataEnvelopeDto {
    /**
     * Domain metadata.
     */
    domain: TypedDataDomainDto;
    /**
     * Primary type.
     */
    primaryType: string;
    /**
     * Type map.
     *
     * Typed as `Record` because the runtime serializer
     * (`serde_wasm_bindgen::Serializer::json_compatible`) emits a
     * plain JavaScript object for `BTreeMap` fields. The override
     * aligns the generated TypeScript declaration with the runtime
     * shape so the declared type matches the value the wasm boundary
     * emits byte-for-byte.
     */
    types: Record<string, TypedDataFieldDto[]>;
    /**
     * Parsed message body.
     */
    message: Value;
}

/**
 * Typed-data field DTO.
 */
export interface TypedDataFieldDto {
    /**
     * Field name.
     */
    name: string;
    /**
     * Solidity field type.
     */
    type: string;
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
export type AppDataHash = string;

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
export type Address = string;

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
export type OrderUid = string;

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
export type HexData = string;

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
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init(): void;

/**
 * Computes the canonical order UID and order digest for an unsigned order.
 *
 * The UID combines the EIP-712 order digest, owner address, and validity
 * timestamp using the same packing rules as the native Rust SDK.
 *
 * @param input Unsigned order fields to hash and pack.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Order owner address included in the UID suffix.
 * @returns A versioned envelope with `orderUid` and `orderDigest`.
 * @throws SdkError when the order, owner, or chain id is invalid.
 */
export function computeOrderUid(input: OrderInput, chainId: number, owner: string): WasmEnvelope<GeneratedOrderUidDto>;

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
 * @throws SdkError when the log is malformed or its topic set matches no known
 * eth-flow lifecycle event.
 */
export function decodeEthFlowLog(log: EventLogInput): WasmEnvelope<EthFlowEventDto>;

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
 * @throws SdkError when the log is malformed or its topic set matches no known
 * settlement event.
 */
export function decodeSettlementLog(log: EventLogInput): WasmEnvelope<SettlementEventDto>;

/**
 * Returns canonical CoW Protocol deployment addresses for a chain.
 *
 * The optional environment selects production or staging deployment data. When
 * omitted, the helper uses the SDK default environment.
 *
 * @param chainId EVM chain id to resolve.
 * @param env Optional CoW environment name, such as `prod` or `staging`.
 * @returns Settlement, VaultRelayer, EthFlow, and AllowListAuth addresses.
 * @throws SdkError when the chain or environment is unsupported.
 */
export function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<DeploymentAddressesDto>;

/**
 * Computes the CoW Protocol EIP-712 domain separator for a supported chain.
 *
 * Use this helper when a JavaScript host needs to compare the domain hash used
 * by the Rust SDK with another signing stack. The input is an EVM chain id,
 * not a CoW environment selector.
 *
 * @param chainId EVM chain id supported by the deployment registry.
 * @returns The `0x`-prefixed 32-byte domain separator.
 * @throws SdkError when the chain is not supported.
 */
export function domainSeparator(chainId: number): string;

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA order signature.
 *
 * Use this pure helper when a smart-account flow already has the wrapped ECDSA
 * signature and needs the contract-signature payload bytes expected by CoW
 * Protocol order submission.
 *
 * @param input Unsigned order used to derive the EIP-1271 payload.
 * @param ecdsaSignature Wrapped ECDSA signature as a `0x`-prefixed string.
 * @returns A versioned envelope containing the encoded EIP-1271 payload.
 * @throws SdkError when the order or signature is invalid.
 */
export function eip1271SignaturePayload(input: OrderInput, ecdsaSignature: string): WasmEnvelope<string>;

/**
 * Builds signer-facing EIP-712 typed data for an unsigned order.
 *
 * The returned envelope contains the domain, type map, primary type, and
 * order message that wallet libraries expect for EIP-712 signing. It is
 * deterministic for the provided order and chain id.
 *
 * @param input Unsigned order fields using the facade order DTO shape.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @returns A versioned envelope containing typed-data DTO fields.
 * @throws SdkError when order parsing or chain validation fails.
 */
export function orderTypedData(input: OrderInput, chainId: number): WasmEnvelope<TypedDataEnvelopeDto>;

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 *
 * The SDK computes the canonical order digest, passes the digest as a
 * `0x`-prefixed string to the callback, normalizes the signature, and returns
 * an `ethsign` signed-order DTO.
 *
 * @param input Unsigned order fields to sign.
 * @param chainId EVM chain id used for the digest.
 * @param owner Owner address used in the generated order UID.
 * @param digestSigner Callback that signs the digest string.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderEthSignDigest(input: OrderInput, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through a custom EIP-1271 callback.
 *
 * Use this method when the JavaScript host owns the smart-account or
 * account-abstraction client and can return the final contract signature
 * directly. The SDK still builds typed data and the deterministic order UID.
 *
 * @param input Unsigned order to sign.
 * @param chainId EVM chain id for the EIP-712 domain.
 * @param owner Smart-account owner address used in the generated order UID.
 * @param customCallback Callback that returns the final EIP-1271 signature.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed-order DTO.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithCustomEip1271(input: OrderInput, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through an EIP-1193 request callback.
 *
 * The callback receives an `eth_signTypedData_v4` request object with owner
 * address and serialized typed data. This is the bridge for injected wallets
 * and wallet-client libraries that expose an EIP-1193-style request function.
 *
 * @param input Unsigned order fields to sign.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Owner address used in the wallet request and order UID.
 * @param requestCallback Callback that executes the EIP-1193 request.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws SdkError for invalid input, wallet failure, timeout, or cancellation.
 */
export function signOrderWithEip1193(input: OrderInput, chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 *
 * The SDK sends the EIP-712 envelope to the provided typed-data callback,
 * then converts the returned ECDSA signature into the CoW EIP-1271 payload.
 * Per-call options may attach cancellation and wallet timeout settings.
 *
 * @param input Unsigned order to sign.
 * @param chainId EVM chain id for the EIP-712 domain.
 * @param owner Smart-account owner address used in the generated order UID.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed-order DTO.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithEip1271(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through a typed-data callback.
 *
 * The SDK builds the EIP-712 typed-data envelope, passes it to the callback,
 * normalizes the returned ECDSA signature, and returns the signed-order DTO
 * with the canonical order UID and digest.
 *
 * @param input Unsigned order fields to sign.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Owner address used in the generated order UID.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithTypedDataSigner(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

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
