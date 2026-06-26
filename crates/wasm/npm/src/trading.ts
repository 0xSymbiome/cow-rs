import * as raw from "./raw/trading.js";
import {
  assertActive,
  callAsync,
  callSync,
  defaultsFrom,
  disposeRaw,
  mergeOptions,
  mergeSigningOptions,
  translateClientConfig,
  type ClientDefaults
} from "./internal.js";
import type { WasmEnvelope } from "./envelope.js";
import type { CommonClientConfig, SdkClientOptions, SigningOptions, TradingClientConfig } from "./options.js";
import type {
  ContractReadCallback,
  CustomEip1271Callback,
  DigestSignerCallback,
  TypedDataSignerCallback
} from "./callbacks.js";

export type OrderBookClientConfig = CommonClientConfig;

let initialized: Promise<void> | undefined;

/**
 * Initialize the wasm module, idempotently, once per module instance.
 *
 * On the `web` build — Cloudflare Workers, Deno, Vercel Edge, and no-bundler
 * browsers — the host owns module instantiation, so it must call this once with
 * the compiled module (Workers pass the `CompiledWasm` binding) or its URL/bytes
 * (Deno and browsers). On the `bundler` and `nodejs` builds the host
 * bundler/runtime instantiates the module on import, so this resolves
 * immediately and the argument is ignored — calling it is optional and harmless,
 * which keeps one call shape working across every target.
 */
export async function initialize(module?: WebAssembly.Module | raw.InitInput): Promise<void> {
  if (!initialized) {
    initialized = raw.initializeRaw({ module_or_path: module }).then(() => undefined);
  }
  await initialized;
}

export default initialize;

export class OrderBookClient {
  #inner: InstanceType<typeof raw.RawOrderBookClient>;
  #defaults: ClientDefaults;
  #disposed = false;

  constructor(config: OrderBookClientConfig) {
    try {
      this.#defaults = defaultsFrom(config);
      this.#inner = new raw.RawOrderBookClient(
        translateClientConfig(config) as unknown as raw.OrderBookClientConfig
      );
    } catch (error) {
      throw callSync(() => {
        throw error;
      });
    }
  }

  async cancelOrders(
    signed: raw.SignedCancellations,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<{ cancelled: true }>> {
    return this.#call((client, merged) => client.cancelOrders(signed, merged), options);
  }

  async getNativePrice(
    token: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.NativePriceResponse>> {
    return this.#call((client, merged) => client.getNativePrice(token, merged), options);
  }

  async getOrder(
    orderUid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.Order>> {
    return this.#call((client, merged) => client.getOrder(orderUid, merged), options);
  }

  async getOrders(
    owner: string,
    pagination?: raw.PaginationOptions | null,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.Order[]>> {
    return this.#call((client, merged) => client.getOrders(owner, pagination ?? null, merged), options);
  }

  async getOrderMultiEnv(
    orderUid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.Order>> {
    return this.#call((client, merged) => client.getOrderMultiEnv(orderUid, merged), options);
  }

  async getTxOrders(
    txHash: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.Order[]>> {
    return this.#call((client, merged) => client.getTxOrders(txHash, merged), options);
  }

  async getVersion(options?: SdkClientOptions | null): Promise<WasmEnvelope<string>> {
    return this.#call((client, merged) => client.getVersion(merged), options);
  }

  getOrderLink(orderUid: string): WasmEnvelope<string> {
    assertActive(this.#disposed);
    return callSync(() => this.#inner.getOrderLink(orderUid));
  }

  async getQuote(
    request: raw.OrderQuoteRequest,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.OrderQuoteResponse>> {
    return this.#call((client, merged) => client.getQuote(request, merged), options);
  }

  async getTrades(
    query: raw.GetTradesRequest,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.Trade[]>> {
    return this.#call((client, merged) => client.getTrades(query, merged), options);
  }

  async getOrderCompetitionStatus(
    orderUid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.CompetitionOrderStatus>> {
    return this.#call(
      (client, merged) => client.getOrderCompetitionStatus(orderUid, merged),
      options
    );
  }

  async getTotalSurplus(
    owner: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.TotalSurplus>> {
    return this.#call((client, merged) => client.getTotalSurplus(owner, merged), options);
  }

  async getSolverCompetition(
    auctionId: number,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.SolverCompetitionResponse>> {
    return this.#call(
      (client, merged) => client.getSolverCompetition(auctionId, merged),
      options
    );
  }

  async getSolverCompetitionByTxHash(
    txHash: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.SolverCompetitionResponse>> {
    return this.#call(
      (client, merged) => client.getSolverCompetitionByTxHash(txHash, merged),
      options
    );
  }

  async getAppData(
    appDataHash: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.AppDataObject>> {
    return this.#call((client, merged) => client.getAppData(appDataHash, merged), options);
  }

  async uploadAppData(
    appDataHash: string,
    fullAppData: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<{ uploaded: true }>> {
    return this.#call(
      (client, merged) => client.uploadAppData(appDataHash, fullAppData, merged),
      options
    );
  }

  async sendOrder(
    signed: raw.SignedOrder,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<string>> {
    return this.#call((client, merged) => client.sendOrder(signed, merged), options);
  }

  async sendOrderCreation(
    input: raw.OrderCreation,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<string>> {
    return this.#call((client, merged) => client.sendOrderCreation(input, merged), options);
  }

  dispose(): void {
    if (!this.#disposed) {
      disposeRaw(this.#inner);
      this.#disposed = true;
    }
  }

  [Symbol.dispose](): void {
    this.dispose();
  }

  #call<T>(
    operation: (
      client: InstanceType<typeof raw.RawOrderBookClient>,
      options: SdkClientOptions | undefined
    ) => Promise<T>,
    options?: SdkClientOptions | null
  ): Promise<T> {
    return callAsync(() => {
      assertActive(this.#disposed);
      return operation(this.#inner, mergeOptions(this.#defaults, options));
    });
  }
}

export class TradingClient {
  #inner: InstanceType<typeof raw.RawTradingClient>;
  #defaults: ClientDefaults;
  #disposed = false;

  constructor(config: TradingClientConfig) {
    try {
      this.#defaults = defaultsFrom(config);
      this.#inner = new raw.RawTradingClient(
        translateClientConfig(config) as unknown as raw.TradingClientConfig
      );
    } catch (error) {
      throw callSync(() => {
        throw error;
      });
    }
  }

  async buildSellNativeCurrencyTx(
    order: raw.OrderData,
    quoteId: number,
    from: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.BuiltSellNativeCurrencyTx>> {
    return this.#call(
      (client, merged) => client.buildSellNativeCurrencyTx(order, quoteId, from, merged),
      options
    );
  }

  async buildSellNativeCurrencyTxFromQuote(
    quoteResults: raw.QuoteResults,
    from: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.BuiltSellNativeCurrencyTx>> {
    return this.#call(
      (client, merged) => client.buildSellNativeCurrencyTxFromQuote(quoteResults, from, merged),
      options
    );
  }

  async getCowProtocolAllowance(
    params: raw.AllowanceParams,
    readContractCallback: ContractReadCallback,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<string>> {
    return this.#call(
      (client, merged) => client.getCowProtocolAllowance(params, readContractCallback, merged),
      options
    );
  }

  async buildApprovalTx(
    params: raw.ApprovalParams,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.TransactionRequest>> {
    return this.#call((client, merged) => client.buildApprovalTx(params, merged), options);
  }

  buildWrapTx(amount: string): WasmEnvelope<raw.TransactionRequest> {
    assertActive(this.#disposed);
    return callSync(() => this.#inner.buildWrapTx(amount));
  }

  buildUnwrapTx(amount: string): WasmEnvelope<raw.TransactionRequest> {
    assertActive(this.#disposed);
    return callSync(() => this.#inner.buildUnwrapTx(amount));
  }

  async getQuote(
    params: raw.TradeParams,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.QuoteResults>> {
    return this.#call((client, merged) => client.getQuote(params, merged), options);
  }

  async postLimitOrder(
    params: raw.LimitTradeParams,
    owner: string,
    signerCallback: TypedDataSignerCallback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<raw.OrderPostingResult>> {
    return this.#callSigning(
      (client, merged) => client.postLimitOrder(params, owner, signerCallback, merged ?? null),
      options
    );
  }

  async postSwapOrder(
    params: raw.TradeParams,
    owner: string,
    signerCallback: TypedDataSignerCallback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<raw.OrderPostingResult>> {
    return this.#callSigning(
      (client, merged) => client.postSwapOrder(params, owner, signerCallback, merged ?? null),
      options
    );
  }

  async postSwapOrderFromQuote(
    quoteResults: raw.QuoteResults,
    owner: string,
    signerCallback: TypedDataSignerCallback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<raw.OrderPostingResult>> {
    return this.#callSigning(
      (client, merged) =>
        client.postSwapOrderFromQuote(quoteResults, owner, signerCallback, merged ?? null),
      options
    );
  }

  async postSwapOrderWithEip1271(
    params: raw.TradeParams,
    owner: string,
    customCallback: CustomEip1271Callback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<raw.OrderPostingResult>> {
    return this.#callSigning(
      (client, merged) => client.postSwapOrderWithEip1271(params, owner, customCallback, merged ?? null),
      options
    );
  }

  dispose(): void {
    if (!this.#disposed) {
      disposeRaw(this.#inner);
      this.#disposed = true;
    }
  }

  [Symbol.dispose](): void {
    this.dispose();
  }

  #call<T>(
    operation: (
      client: InstanceType<typeof raw.RawTradingClient>,
      options: SdkClientOptions | undefined
    ) => Promise<T>,
    options?: SdkClientOptions | null
  ): Promise<T> {
    return callAsync(() => {
      assertActive(this.#disposed);
      return operation(this.#inner, mergeOptions(this.#defaults, options));
    });
  }

  #callSigning<T>(
    operation: (
      client: InstanceType<typeof raw.RawTradingClient>,
      options: SigningOptions | undefined
    ) => Promise<T>,
    options?: SigningOptions | null
  ): Promise<T> {
    return callAsync(() => {
      assertActive(this.#disposed);
      return operation(this.#inner, mergeSigningOptions(this.#defaults, options));
    });
  }
}

export function appDataDoc(doc: raw.AppDataParams): WasmEnvelope<raw.AppDataDocument> {
  return callSync(() => raw.appDataDoc(doc));
}

export function appDataHexToCid(appDataHex: string): WasmEnvelope<string> {
  return callSync(() => raw.appDataHexToCid(appDataHex));
}

export function appDataInfo(doc: raw.AppDataParams): WasmEnvelope<raw.AppDataInfo> {
  return callSync(() => raw.appDataInfo(doc));
}

export function buildCancelOrderTx(
  params: raw.OrderTraderParams
): WasmEnvelope<raw.TransactionRequest> {
  return callSync(() => raw.buildCancelOrderTx(params));
}

export function buildPresignTx(
  params: raw.OrderTraderParams
): WasmEnvelope<raw.TransactionRequest> {
  return callSync(() => raw.buildPresignTx(params));
}

export function cidToAppDataHex(cid: string): WasmEnvelope<string> {
  return callSync(() => raw.cidToAppDataHex(cid));
}

export function computeOrderUid(
  order: raw.OrderData,
  chainId: number,
  owner: string
): WasmEnvelope<raw.GeneratedOrderUid> {
  return callSync(() => raw.computeOrderUid(order, chainId, owner));
}

export function decodeEthFlowLog(log: raw.EventLog): WasmEnvelope<raw.EthFlowEvent> {
  return callSync(() => raw.decodeEthFlowLog(log));
}

export function decodeSettlementLog(log: raw.EventLog): WasmEnvelope<raw.SettlementEvent> {
  return callSync(() => raw.decodeSettlementLog(log));
}

export function deploymentAddresses(
  chainId: number,
  env?: string | null
): WasmEnvelope<raw.DeploymentAddresses> {
  return callSync(() => raw.deploymentAddresses(chainId, env ?? null));
}

export function domainSeparator(chainId: number): WasmEnvelope<string> {
  return callSync(() => raw.domainSeparator(chainId));
}

export function eip1271SignaturePayload(
  order: raw.OrderData,
  ecdsaSignature: string
): WasmEnvelope<string> {
  return callSync(() => raw.eip1271SignaturePayload(order, ecdsaSignature));
}

export type TypedDataMessage = unknown;

export function orderTypedData(
  order: raw.OrderData,
  chainId: number
): WasmEnvelope<raw.TypedDataEnvelope<TypedDataMessage>> {
  return callSync(() => raw.orderTypedData(order, chainId));
}

export function signCancellationEthSignDigest(
  orderUids: string[],
  chainId: number,
  digestSigner: DigestSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedCancellations>> {
  return callAsync(() =>
    raw.signCancellationEthSignDigest(orderUids, chainId, digestSigner, options ?? null)
  );
}

export function signCancellationWithTypedDataSigner(
  orderUids: string[],
  chainId: number,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedCancellations>> {
  return callAsync(() =>
    raw.signCancellationWithTypedDataSigner(orderUids, chainId, typedDataSigner, options ?? null)
  );
}

export function signOrderEthSignDigest(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  digestSigner: DigestSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderEthSignDigest(order, chainId, owner, digestSigner, options ?? null)
  );
}

export function signOrderWithCustomEip1271(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  customCallback: CustomEip1271Callback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderWithCustomEip1271(order, chainId, owner, customCallback, options ?? null)
  );
}

export function signOrderWithEip1271(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderWithEip1271(order, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function signOrderWithTypedDataSigner(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderWithTypedDataSigner(order, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function supportedChainIds(): Uint32Array {
  return callSync(() => raw.supportedChainIds());
}

export function validateAppDataDoc(doc: raw.AppDataParams): WasmEnvelope<raw.ValidationResult> {
  return callSync(() => raw.validateAppDataDoc(doc));
}

export function wasmVersion(): string {
  return callSync(() => raw.wasmVersion());
}

export function wrappedNativeToken(chainId: number): WasmEnvelope<raw.WrappedNativeToken> {
  return callSync(() => raw.wrappedNativeToken(chainId));
}

export type {
  AllowanceParams,
  Amounts,
  AppDataDocument,
  AppDataParams,
  AppDataInfo,
  AppDataObject,
  ApprovalParams,
  BuiltSellNativeCurrencyTx,
  BuyTokenDestination,
  CompetitionAuction,
  CompetitionOrderStatus,
  CompetitionOrderStatusKind,
  ContractCall,
  Costs,
  CowEip1271SignRequest,
  DeploymentAddresses,
  EthflowData,
  EthFlowEvent,
  EventLog,
  ExecutedAmounts,
  ExecutedProtocolFee,
  FeeComponent,
  GeneratedOrderUid,
  InitInput,
  InteractionData,
  LimitTradeParams,
  NativePriceResponse,
  NetworkFee,
  OnchainOrderData,
  Order,
  OrderData,
  OrderbookBinding,
  OrderClass,
  OrderCreation,
  OrderInteractions,
  OrderKind,
  OrderPostingResult,
  OrderQuoteRequest,
  OrderQuoteResponse,
  OrderStatus,
  OrderTraderParams,
  PaginationOptions,
  PartnerFee,
  PartnerFeePolicy,
  QuoteAmountsAndCosts,
  QuoteData,
  QuoteResults,
  SellTokenSource,
  SettlementEvent,
  SignedCancellations,
  SignedOrder,
  SigningScheme,
  SolverCompetitionOrder,
  SolverCompetitionResponse,
  SolverExecution,
  SolverSettlement,
  StoredOrderQuote,
  TotalSurplus,
  Trade,
  TradeParams,
  GetTradesRequest,
  TradingAppDataInfo,
  TransactionRequest,
  TypedDataDomain,
  TypedDataEnvelope,
  TypedDataField,
  ValidationResult,
  WrappedNativeToken
} from "./raw/trading.js";
export type {
  ContractReadCallback,
  CowEnv,
  CustomEip1271Callback,
  DigestSignerCallback,
  TypedDataSignerCallback
} from "./callbacks.js";
export { CowError, isCowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "./errors.js";
export type { CowErrorData, OrderBookErrorType, OrderBookRejectionCategory } from "./errors.js";
export { withRetry } from "./retry.js";
export type { RetryOptions } from "./retry.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type {
  CowFetchCallback,
  CowFetchRequest,
  CowFetchResponse,
  HttpTransportConfig,
  JitterStrategyConfig,
  LimiterScopeConfig,
  RequestRateLimiterConfig,
  RetryPolicyConfig,
  SdkClientOptions,
  SigningOptions,
  TradingClientConfig,
  TransportPolicyConfig,
  WalletConfig
} from "./options.js";
