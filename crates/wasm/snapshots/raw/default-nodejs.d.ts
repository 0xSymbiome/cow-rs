/// <reference lib="esnext.disposable" />
/* tslint:disable */
/* eslint-disable */

export interface IpfsClientConfig {
    ipfsUri?: string | null;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface OrderBookClientConfig {
    chainId: number;
    env?: string | null;
    apiKey?: string | null;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface SubgraphClientConfig {
    chainId: number;
    apiKey: string;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface TradingClientConfig {
    chainId: number;
    env?: string | null;
    appCode: string;
    apiKey?: string | null;
    transport: HttpTransportConfig;
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



export type ContractReadCallback = (
request: ContractCallDto,
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
| { kind: "fetch"; fetch?: typeof globalThis.fetch }
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
 * A single pre/post interaction attached to an order, mirroring
 * `cow_sdk_orderbook::InteractionData`.
 */
export interface InteractionDataDto {
    /**
     * Contract address targeted by the interaction.
     */
    target: string;
    /**
     * Native token value sent with the interaction.
     */
    value: string;
    /**
     * Hex-encoded calldata forwarded to `target`.
     */
    callData: string;
}

/**
 * Allowance helper parameters.
 */
export interface AllowanceParametersInput {
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
 * App-data document input.
 */
export interface AppDataDocInput {
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
export interface AppDataDocDto {
    /**
     * App-data document.
     */
    document: Value;
}

/**
 * App-data document, serialized payload, and digest used by the quote flow,
 * mirroring `cow_sdk_trading::TradingAppDataInfo`.
 */
export interface TradingAppDataInfoDto {
    /**
     * Parsed app-data document (arbitrary JSON).
     */
    doc: Value;
    /**
     * Canonically serialized app-data payload.
     */
    fullAppData: string;
    /**
     * Keccak-256 digest used in protocol order payloads.
     */
    appDataKeccak256: string;
}

/**
 * App-data info output.
 */
export interface AppDataInfoDto {
    /**
     * CID representation.
     */
    cid: string;
    /**
     * Deterministic app-data content.
     */
    appDataContent: string;
    /**
     * App-data hash.
     */
    appDataHex: string;
}

/**
 * App-data validation result.
 */
export interface ValidationResultDto {
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
 * Contract-read callback request.
 */
export interface ContractCallDto {
    /**
     * Target contract address.
     */
    address: string;
    /**
     * ABI method name.
     */
    method: string;
    /**
     * JSON ABI fragment.
     */
    abiJson: string;
    /**
     * JSON-encoded function arguments.
     */
    argsJson: string;
}

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
 * Effective trade parameters after SDK defaults and advanced settings were
 * applied, mirroring `cow_sdk_trading::TradeParams`.
 */
export interface TradeParametersDto {
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Optional owner override.
     */
    owner?: string;
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Amount interpreted according to `kind`.
     */
    amount: string;
    /**
     * Optional environment override.
     */
    env?: CowEnvDto;
    /**
     * Optional settlement-contract overrides keyed by chain id. Typed as
     * `Record` rather than `Map` for runtime alignment (the wire form is a
     * plain JSON object).
     */
    settlementContractOverride?: Record<string, string>;
    /**
     * Optional `EthFlow`-contract overrides keyed by chain id.
     */
    ethFlowContractOverride?: Record<string, string>;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance: TokenBalanceDto;
    /**
     * Optional explicit slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Optional relative validity duration in seconds.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata.
     */
    partnerFee?: PartnerFeeDto;
}

/**
 * Executed protocol-fee component of a trade, mirroring
 * `cow_sdk_orderbook::ExecutedProtocolFee`.
 */
export interface ExecutedProtocolFeeDto {
    /**
     * Fee policy that produced this fee, when services returns it (arbitrary
     * JSON mirroring the upstream policy shape).
     */
    policy?: Value;
    /**
     * Fee amount taken.
     */
    amount?: string;
    /**
     * Token in which the fee was taken.
     */
    token?: string;
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
 * Fee component represented by an amount and a basis-point value.
 */
export interface FeeComponentDto {
    /**
     * Fee amount.
     */
    amount: string;
    /**
     * Fee in basis points.
     */
    bps: number;
}

/**
 * Full app-data document returned by the orderbook app-data endpoint.
 */
export interface AppDataObjectDto {
    /**
     * Full serialized app-data payload.
     */
    fullAppData: string;
}

/**
 * Full quote cost breakdown.
 */
export interface CostsDto {
    /**
     * Network-fee component.
     */
    networkFee: NetworkFeeDto;
    /**
     * Partner-fee component.
     */
    partnerFee: FeeComponentDto;
    /**
     * Protocol-fee component.
     */
    protocolFee: FeeComponentDto;
}

/**
 * Fully resolved quote result produced by trading quote helpers, mirroring
 * `cow_sdk_trading::QuoteResults`.
 *
 * Returned by `getQuote` and accepted by `postSwapOrderFromQuote` unchanged.
 */
export interface QuoteResultsDto {
    /**
     * Effective trade parameters after SDK defaults and advanced settings.
     */
    tradeParameters: TradeParametersDto;
    /**
     * Suggested slippage in basis points after resolution.
     */
    suggestedSlippageBps: number;
    /**
     * Fee and amount breakdown derived from the orderbook quote.
     */
    amountsAndCosts: QuoteAmountsAndCostsDto;
    /**
     * Unsigned order payload produced for signing or on-chain submission.
     */
    orderToSign: OrderDataDto;
    /**
     * Raw orderbook quote response.
     */
    quoteResponse: OrderQuoteResponseDto;
    /**
     * App-data document, serialized payload, and digest used by the quote flow.
     */
    appDataInfo: TradingAppDataInfoDto;
    /**
     * Originating orderbook runtime binding captured by the quote flow.
     */
    orderbookBinding?: OrderBookRuntimeBindingDto;
    /**
     * Typed order-facing EIP-712 envelope kept for consumers.
     */
    orderTypedData: TypedDataEnvelopeDto;
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
 * Jitter strategy accepted by JS client constructors.
 */
export type JitterStrategyConfig = "none" | "full" | "equal" | "decorrelated";

/**
 * Limit-order parameters accepted by trading posting helpers.
 */
export interface LimitTradeParametersInput {
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Optional owner override.
     */
    owner?: string;
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Sell amount before transformations.
     */
    sellAmount: string;
    /**
     * Buy amount before transformations.
     */
    buyAmount: string;
    /**
     * Optional quote id.
     */
    quoteId?: number;
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
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: TokenBalanceDto;
    /**
     * Optional slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Optional relative validity duration.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata.
     */
    partnerFee?: PartnerFeeInput;
}

/**
 * Native-currency sell transaction bundle.
 */
export interface BuiltSellNativeCurrencyTxDto {
    /**
     * Deterministic order UID.
     */
    orderUid: string;
    /**
     * Transaction request to submit.
     */
    transaction: TransactionRequestDto;
    /**
     * Unsigned order encoded by the transaction.
     */
    orderToSign: OrderInput;
    /**
     * Effective order owner.
     */
    from: string;
}

/**
 * Native-price response from the orderbook native-price endpoint, mirroring
 * `cow_sdk_orderbook::NativePriceResponse`.
 */
export interface NativePriceResponseDto {
    /**
     * Token price quoted in the chain\'s native asset.
     */
    price: number;
}

/**
 * Network-fee amounts expressed in both quote currencies.
 */
export interface NetworkFeeDto {
    /**
     * Network fee expressed in sell-token units.
     */
    amountInSellCurrency: string;
    /**
     * Network fee expressed in buy-token units.
     */
    amountInBuyCurrency: string;
}

/**
 * On-chain placement metadata, mirroring
 * `cow_sdk_orderbook::OnchainOrderData`.
 */
export interface OnchainOrderDataDto {
    /**
     * Sender address associated with the on-chain placement.
     */
    sender: string;
    /**
     * Placement error emitted by services, when on-chain placement failed.
     */
    placementError?: string;
}

/**
 * One typed partner-fee policy object, mirroring
 * `cow_sdk_app_data::PartnerFeePolicy`.
 */
export type PartnerFeePolicyDto = { volumeBps: number; recipient: string } | { surplusBps: number; maxVolumeBps: number; recipient: string } | { priceImprovementBps: number; maxVolumeBps: number; recipient: string };

/**
 * Order class surfaced by the orderbook API, mirroring
 * `cow_sdk_orderbook::OrderClass`.
 */
export type OrderClassDto = "market" | "limit" | "liquidity";

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
 * Order lifecycle status returned by the orderbook API, mirroring
 * `cow_sdk_orderbook::OrderStatus`.
 */
export type OrderStatusDto = "presignaturePending" | "open" | "fulfilled" | "cancelled" | "expired";

/**
 * Order returned by the orderbook order endpoints, mirroring
 * `cow_sdk_orderbook::Order` (the enriched order shape, with the normalized
 * `totalFee` folded in).
 */
export interface OrderDto {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Sell amount in the upstream decimal-string wire shape.
     */
    sellAmount: string;
    /**
     * Buy amount in the upstream decimal-string wire shape.
     */
    buyAmount: string;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo: number;
    /**
     * App-data hash attached to the order.
     */
    appData: string;
    /**
     * Optional app-data hash echoed for debugging by the orderbook.
     */
    appDataHash?: string;
    /**
     * Order-level fee echoed on the orderbook response; always `\"0\"` in
     * practice because services rejects non-zero order-level fees.
     */
    feeAmount: string;
    /**
     * Strict balance-check flag, present only when the order was created with
     * it set.
     */
    fullBalanceCheck?: boolean;
    /**
     * Order kind.
     */
    kind: OrderKindDto;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance: TokenBalanceDto;
    /**
     * Signature scheme used for `signature`.
     */
    signingScheme: SigningSchemeDto;
    /**
     * Raw signature string.
     */
    signature: string;
    /**
     * Effective owner field returned by the API, when present.
     */
    from?: string;
    /**
     * Quote id used when the order originated from a quote.
     */
    quoteId?: number;
    /**
     * Order class.
     */
    class: OrderClassDto;
    /**
     * Canonical owner surfaced by the orderbook response.
     */
    owner: string;
    /**
     * Order UID.
     */
    uid: string;
    /**
     * Creation timestamp string returned by the API.
     */
    creationDate?: string;
    /**
     * Executed sell amount.
     */
    executedSellAmount?: string;
    /**
     * Executed sell amount before fees.
     */
    executedSellAmountBeforeFees?: string;
    /**
     * Executed buy amount.
     */
    executedBuyAmount?: string;
    /**
     * Executed fee component, when provided.
     */
    executedFee?: string;
    /**
     * Deprecated legacy executed-fee value, present on older order payloads.
     */
    executedFeeAmount?: string;
    /**
     * Token in which the executed fee was captured, when returned.
     */
    executedFeeToken?: string;
    /**
     * Whether the order was invalidated by the protocol.
     */
    invalidated?: boolean;
    /**
     * Order lifecycle status.
     */
    status: OrderStatusDto;
    /**
     * Whether services classified the order as a liquidity order.
     */
    isLiquidityOrder?: boolean;
    /**
     * On-chain user for `EthFlow`-style orders.
     */
    onchainUser?: string;
    /**
     * `EthFlow`-specific metadata.
     */
    ethflowData?: EthflowDataDto;
    /**
     * On-chain placement metadata, when services returns it.
     */
    onchainOrderData?: OnchainOrderDataDto;
    /**
     * Full app-data payload, when services returns it.
     */
    fullAppData?: string;
    /**
     * Settlement contract address against which the order was signed.
     */
    settlementContract: string;
    /**
     * Stored quote metadata for quote-linked orders.
     */
    quote?: StoredOrderQuoteDto;
    /**
     * Optional pre and post interactions associated with the order.
     */
    interactions?: OrderInteractionsDto;
    /**
     * Total fee normalized by the SDK transform layer.
     */
    totalFee?: string;
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
 * Orderbook order-creation input.
 */
export interface OrderCreationInput {
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
    kind: OrderKindDto;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: TokenBalanceDto;
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
export interface OrderQuoteRequestInput {
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
    kind: OrderKindDto;
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
     * Whether partial fills are allowed.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: TokenBalanceDto;
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
 * Originating orderbook runtime binding captured by the quote flow, mirroring
 * `cow_sdk_orderbook::OrderbookRuntimeBinding`.
 */
export interface OrderBookRuntimeBindingDto {
    /**
     * Chain id fixed by the orderbook client.
     */
    chainId: number;
    /**
     * Environment fixed by the orderbook client.
     */
    env: CowEnvDto;
    /**
     * Resolved base URL used by the orderbook client when available.
     */
    resolvedBaseUrl?: string;
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
 * Partner-fee input accepted by trading swap parameters.
 */
export type PartnerFeeInput = PartnerFeePolicyInput | PartnerFeePolicyInput[];

/**
 * Partner-fee metadata, mirroring `cow_sdk_app_data::PartnerFee`.
 */
export type PartnerFeeDto = PartnerFeePolicyDto | PartnerFeePolicyDto[];

/**
 * Partner-fee policy input for trading swap parameters.
 */
export interface PartnerFeePolicyInput {
    /**
     * Volume fee in basis points.
     */
    volumeBps?: number;
    /**
     * Surplus fee in basis points.
     */
    surplusBps?: number;
    /**
     * Price-improvement fee in basis points.
     */
    priceImprovementBps?: number;
    /**
     * Maximum volume fee in basis points.
     */
    maxVolumeBps?: number;
    /**
     * Fee recipient address.
     */
    recipient: string;
}

/**
 * Pre/post interactions associated with an order, mirroring
 * `cow_sdk_orderbook::OrderInteractions`.
 */
export interface OrderInteractionsDto {
    /**
     * Interactions executed before the order\'s trade.
     */
    pre?: InteractionDataDto[];
    /**
     * Interactions executed after the order\'s trade.
     */
    post?: InteractionDataDto[];
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
 * Raw orderbook quote response, mirroring
 * `cow_sdk_orderbook::OrderQuoteResponse`.
 */
export interface OrderQuoteResponseDto {
    /**
     * Resolved quote payload.
     */
    quote: QuoteDataDto;
    /**
     * Effective owner used for the quote, when returned by the API.
     */
    from?: string;
    /**
     * Quote price/fee expiry as an ISO-8601 UTC string.
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
 * Resolved quote payload echoed by the orderbook `/quote` response, mirroring
 * `cow_sdk_orderbook::QuoteData`.
 */
export interface QuoteDataDto {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Sell amount in the upstream decimal-string wire shape.
     */
    sellAmount: string;
    /**
     * Buy amount in the upstream decimal-string wire shape.
     */
    buyAmount: string;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo: number;
    /**
     * Effective app-data hash derived from the orderbook response.
     */
    appData: string;
    /**
     * Explicit app-data hash echoed alongside full app data, when present.
     */
    appDataHash?: string;
    /**
     * Network-cost amount echoed by the orderbook `/quote` response.
     */
    feeAmount: string;
    /**
     * Order kind.
     */
    kind: OrderKindDto;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance: TokenBalanceDto;
    /**
     * Estimated gas units for the quoted trade; empty for a locally
     * constructed quote.
     */
    gasAmount?: string;
    /**
     * Estimated gas price at quote time (wei per gas unit).
     */
    gasPrice?: string;
    /**
     * Sell-token price in native-token atoms per sell-token atom.
     */
    sellTokenPrice?: string;
    /**
     * Signing scheme for the quoted order.
     */
    signingScheme: SigningSchemeDto;
}

/**
 * Result returned by a managed trading submission
 * (`cow_sdk_trading::OrderPostingResult`).
 */
export interface OrderPostingResultDto {
    /**
     * Final order UID.
     */
    orderId: string;
    /**
     * Transaction hash when the flow submitted an on-chain transaction directly.
     */
    txHash?: string;
    /**
     * Signature scheme used for the posted order.
     */
    signingScheme: SigningSchemeDto;
    /**
     * Signature payload sent to the orderbook, or an empty string for
     * transaction-only flows.
     */
    signature: string;
    /**
     * Unsigned order payload used for signing or transaction generation.
     */
    orderToSign: OrderDataDto;
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
 * Sell/buy amount pair at one quote stage.
 */
export interface AmountsDto {
    /**
     * Sell-side amount.
     */
    sellAmount: string;
    /**
     * Buy-side amount.
     */
    buyAmount: string;
}

/**
 * Signature scheme carried on posted and returned orders, mirroring
 * `cow_sdk_orderbook::SigningScheme`, whose wire form is the lowercased
 * variant name.
 */
export type SigningSchemeDto = "eip712" | "ethsign" | "eip1271" | "presign";

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
 * Stepwise quote amounts and cost components across the quote lifecycle.
 */
export interface QuoteAmountsAndCostsDto {
    /**
     * Whether the source quote was sell-sided.
     */
    isSell: boolean;
    /**
     * Cost breakdown for the quote.
     */
    costs: CostsDto;
    /**
     * Amounts before all fees.
     */
    beforeAllFees: AmountsDto;
    /**
     * Amounts before network costs.
     */
    beforeNetworkCosts: AmountsDto;
    /**
     * Amounts after protocol fees.
     */
    afterProtocolFees: AmountsDto;
    /**
     * Amounts after network costs.
     */
    afterNetworkCosts: AmountsDto;
    /**
     * Amounts after partner fees.
     */
    afterPartnerFees: AmountsDto;
    /**
     * Amounts after slippage.
     */
    afterSlippage: AmountsDto;
    /**
     * Amounts that should be signed.
     */
    amountsToSign: AmountsDto;
}

/**
 * Stored quote metadata for quote-linked orders, mirroring
 * `cow_sdk_orderbook::StoredOrderQuote`.
 */
export interface StoredOrderQuoteDto {
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
    sellAmount: string;
    /**
     * Quoted buy amount.
     */
    buyAmount: string;
    /**
     * Estimated network fee in sell-token atoms.
     */
    feeAmount: string;
    /**
     * Solver address that provided the quote.
     */
    solver: string;
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
 * Token-balance mode accepted by wasm order inputs.
 */
export type TokenBalanceDto = "erc20" | "external" | "internal";

/**
 * Trade returned by the orderbook trades endpoint, mirroring
 * `cow_sdk_orderbook::Trade`.
 */
export interface TradeDto {
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
    orderUid: string;
    /**
     * Owner address.
     */
    owner: string;
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Executed sell amount in the upstream decimal-string wire shape.
     */
    sellAmount: string;
    /**
     * Executed sell amount before fees.
     */
    sellAmountBeforeFees?: string;
    /**
     * Executed buy amount in the upstream decimal-string wire shape.
     */
    buyAmount: string;
    /**
     * Protocol fees executed as part of the trade, when services returns them.
     */
    executedProtocolFees?: ExecutedProtocolFeeDto[];
    /**
     * Settlement transaction hash.
     */
    txHash: string | undefined;
}

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
 * Trading swap-parameter input.
 */
export interface SwapParametersInput {
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Optional owner override.
     */
    owner?: string;
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Amount interpreted according to `kind`.
     */
    amount: string;
    /**
     * Optional environment override.
     */
    env?: string;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: TokenBalanceDto;
    /**
     * Optional slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Optional relative validity duration.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata.
     */
    partnerFee?: PartnerFeeInput;
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
    /**
     * Enables or disables transport tracing integration.
     */
    tracingEnabled?: boolean;
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
 * Unsigned order payload (`cow_sdk_core::OrderData`) returned by managed
 * trading flows.
 *
 * Mirrors the signed-order field set. Unlike [`OrderInput`], the `receiver`
 * is always a concrete address because managed flows resolve it before
 * signing, so every field is present on the wire.
 */
export interface OrderDataDto {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Receiver of the bought tokens.
     */
    receiver: string;
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
     * Partial-fill flag.
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
 * `CoW` Protocol environment, mirroring `cow_sdk_core::CowEnv`.
 */
export type CowEnvDto = "prod" | "staging";

/**
 * `EthFlow`-specific order metadata, mirroring
 * `cow_sdk_orderbook::EthflowData`.
 */
export interface EthflowDataDto {
    /**
     * Transaction in which the order was refunded, when present.
     */
    refundTxHash?: string;
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
    fetchAppDataFromCid(cid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataDocDto>>;
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
    fetchAppDataFromHex(appDataHex: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataDocDto>>;
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
    cancelOrders(signed: SignedCancellationsInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ cancelled: true }>>;
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
    getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataObjectDto>>;
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
    getNativePrice(token: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<NativePriceResponseDto>>;
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
    getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderDto>>;
    /**
     * Fetches orders owned by an address with optional pagination.
     *
     * The owner address is validated before the request is dispatched. The
     * response preserves the typed orderbook order shape.
     *
     * @param owner Owner address to query.
     * @param pagination Optional offset and limit.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching orders.
     * @throws CowError for invalid owner, transport failure, timeout, or cancellation.
     */
    getOrders(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderDto[]>>;
    /**
     * Fetches orders owned by an address.
     *
     * This compatibility method is equivalent to `getOrders` and accepts the
     * same pagination options. New TypeScript code can use `getOrders`.
     *
     * @param owner Owner address to query.
     * @param pagination Optional offset and limit.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching orders.
     * @throws CowError for invalid owner, transport failure, timeout, or cancellation.
     */
    getOrdersByOwner(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderDto[]>>;
    /**
     * Fetches a price quote from the orderbook API.
     *
     * The request is converted to the typed orderbook quote request and sent
     * through the configured transport. Per-call options can override the
     * constructor timeout or attach an `AbortSignal`.
     *
     * @param request Quote request DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the quote response.
     * @throws CowError for invalid input, transport failure, timeout, or cancellation.
     */
    getQuote(request: OrderQuoteRequestInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<OrderQuoteResponseDto>>;
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
    getTrades(query: TradesQueryInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<TradeDto[]>>;
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
    sendOrder(signed: SignedOrderDto, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
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
    sendOrderCreation(input: OrderCreationInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
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
     * narrow. Variables and operation name are forwarded when present.
     *
     * @param request GraphQL query, variables, and optional operation name.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the JSON GraphQL response.
     * @throws CowError for transport, timeout, cancellation, or GraphQL errors.
     */
    runQuery(request: SubgraphQueryInput, options?: SdkClientOptions | null): Promise<any>;
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
    buildSellNativeCurrencyTx(order: OrderInput, quoteId: number, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<BuiltSellNativeCurrencyTxDto>>;
    /**
     * Reads CoW Protocol allowance through a read-only contract callback.
     *
     * The SDK builds the contract call while the JavaScript host performs the
     * actual chain read. Use this when a TypeScript runtime owns the RPC
     * provider.
     *
     * @param params Allowance parameters DTO.
     * @param readContractCallback Callback that executes the read-only call.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the allowance amount string.
     * @throws CowError for invalid parameters, callback failure, timeout, or cancellation.
     */
    getCowProtocolAllowance(params: AllowanceParametersInput, readContractCallback: ContractReadCallback, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Fetches a quote without signing or submitting an order.
     *
     * Use this method when a host wants to preview the quote response before
     * asking a wallet to sign or before constructing a post request.
     *
     * @param params Swap parameters DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing quote results.
     * @throws CowError for invalid parameters, transport failure, timeout, or cancellation.
     */
    getQuote(params: SwapParametersInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<QuoteResultsDto>>;
    /**
     * Creates a trading client from a single config object.
     *
     * The config must include `chainId`, `appCode`, and `transport`. Optional
     * environment, API key, timeout, signal, and transport policy fields become
     * defaults for all trading methods.
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
    postLimitOrder(params: LimitTradeParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResultDto>>;
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
    postSwapOrder(params: SwapParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResultDto>>;
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
    postSwapOrderFromQuote(quoteResults: QuoteResultsDto, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResultDto>>;
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
    postSwapOrderWithEip1271(params: SwapParametersInput, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<OrderPostingResultDto>>;
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
export function appDataDoc(doc: AppDataDocInput): WasmEnvelope<AppDataDocDto>;

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
export function appDataInfo(doc: AppDataDocInput): WasmEnvelope<AppDataInfoDto>;

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
export function buildCancelOrderTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>;

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
export function buildPresignTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>;

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
 * @param input Unsigned order fields to hash and pack.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Order owner address included in the UID suffix.
 * @returns A versioned envelope with `orderUid` and `orderDigest`.
 * @throws CowError when the order, owner, or chain id is invalid.
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
 * @throws CowError when the log is malformed or its topic set matches no known
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
 * @throws CowError when the log is malformed or its topic set matches no known
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
 * @throws CowError when the chain or environment is unsupported.
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
 * @throws CowError when the chain is not supported.
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
 * @throws CowError when the order or signature is invalid.
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
 * @throws CowError when order parsing or chain validation fails.
 */
export function orderTypedData(input: OrderInput, chainId: number): WasmEnvelope<TypedDataEnvelopeDto>;

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
export function signCancellationEthSignDigest(orderUids: string[], chainId: number, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs cancellation typed data through an EIP-1193 callback.
 *
 * The callback receives an `eth_signTypedData_v4` request object. Use this
 * helper when an injected wallet or wallet client owns typed-data signing.
 *
 * @param orderUids One or more full order UIDs to cancel.
 * @param chainId EVM chain id used for the cancellation domain.
 * @param owner Owner address included in the EIP-1193 request parameters.
 * @param requestCallback Callback that executes the EIP-1193 request.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing signed cancellations.
 * @throws CowError for invalid input, wallet failure, timeout, or cancellation.
 */
export function signCancellationWithEip1193(orderUids: string[], chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

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
export function signCancellationWithTypedDataSigner(orderUids: string[], chainId: number, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

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
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
 * @throws CowError for invalid input, wallet failure, timeout, or cancellation.
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
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
 * @throws CowError for invalid input, callback failure, timeout, or cancellation.
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
 * Validates an app-data document against the typed metadata contract.
 *
 * Validation is local and deterministic. The result reports whether the
 * document conforms and includes validation details without uploading data.
 *
 * @param doc App-data document input to validate.
 * @returns A versioned envelope containing the validation result.
 * @throws CowError when the input cannot be converted into a document.
 */
export function validateAppDataDoc(doc: AppDataDocInput): WasmEnvelope<ValidationResultDto>;

/**
 * Returns the version of the wasm package runtime.
 *
 * The value comes from the Rust package metadata used to build the wasm
 * artifact and can be included in diagnostics or compatibility checks.
 *
 * @returns The semantic version string for this wasm build.
 */
export function wasmVersion(): string;
