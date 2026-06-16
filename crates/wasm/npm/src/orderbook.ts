import * as raw from "./raw/orderbook.js";
import {
  assertActive,
  callAsync,
  callSync,
  defaultsFrom,
  disposeRaw,
  mergeOptions,
  translateClientConfig,
  type ClientDefaults
} from "./internal.js";
import type { WasmEnvelope } from "./envelope.js";
import type { CommonClientConfig, SdkClientOptions, SigningOptions } from "./options.js";
import type {
  CustomEip1271Callback,
  DigestSignerCallback,
  Eip1193RequestCallback,
  TypedDataSignerCallback
} from "./callbacks.js";

export interface OrderBookClientConfig extends CommonClientConfig {}

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
    signed: raw.SignedCancellationsInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<{ cancelled: true }>> {
    return this.#call((client, merged) => client.cancelOrders(signed, merged), options);
  }

  async getNativePrice(
    token: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.NativePriceResponseDto>> {
    return this.#call((client, merged) => client.getNativePrice(token, merged), options);
  }

  async getOrder(
    orderUid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.OrderDto>> {
    return this.#call((client, merged) => client.getOrder(orderUid, merged), options);
  }

  async getOrders(
    owner: string,
    pagination?: raw.PaginationOptions | null,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.OrderDto[]>> {
    return this.#call((client, merged) => client.getOrders(owner, pagination ?? null, merged), options);
  }

  async getOrderMultiEnv(
    orderUid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.OrderDto>> {
    return this.#call((client, merged) => client.getOrderMultiEnv(orderUid, merged), options);
  }

  async getTxOrders(
    txHash: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.OrderDto[]>> {
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
    request: raw.OrderQuoteRequestInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.OrderQuoteResponseDto>> {
    return this.#call((client, merged) => client.getQuote(request, merged), options);
  }

  async getTrades(
    query: raw.TradesQueryInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.TradeDto[]>> {
    return this.#call((client, merged) => client.getTrades(query, merged), options);
  }

  async getOrderCompetitionStatus(
    orderUid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.CompetitionOrderStatusDto>> {
    return this.#call(
      (client, merged) => client.getOrderCompetitionStatus(orderUid, merged),
      options
    );
  }

  async getTotalSurplus(
    owner: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.TotalSurplusDto>> {
    return this.#call((client, merged) => client.getTotalSurplus(owner, merged), options);
  }

  async getAppData(
    appDataHash: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.AppDataObjectDto>> {
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
    signed: raw.SignedOrderDto,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<string>> {
    return this.#call((client, merged) => client.sendOrder(signed, merged), options);
  }

  async sendOrderCreation(
    input: raw.OrderCreationInput,
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

export function buildCancelOrderTx(
  params: raw.OrderTraderParametersInput
): WasmEnvelope<raw.TransactionRequestDto> {
  return callSync(() => raw.buildCancelOrderTx(params));
}

export function buildPresignTx(
  params: raw.OrderTraderParametersInput
): WasmEnvelope<raw.TransactionRequestDto> {
  return callSync(() => raw.buildPresignTx(params));
}

export function computeOrderUid(
  input: raw.OrderInput,
  chainId: number,
  owner: string
): WasmEnvelope<raw.GeneratedOrderUidDto> {
  return callSync(() => raw.computeOrderUid(input, chainId, owner));
}

export function decodeEthFlowLog(log: raw.EventLogInput): WasmEnvelope<raw.EthFlowEventDto> {
  return callSync(() => raw.decodeEthFlowLog(log));
}

export function decodeSettlementLog(log: raw.EventLogInput): WasmEnvelope<raw.SettlementEventDto> {
  return callSync(() => raw.decodeSettlementLog(log));
}

export function deploymentAddresses(
  chainId: number,
  env?: string | null
): WasmEnvelope<raw.DeploymentAddressesDto> {
  return callSync(() => raw.deploymentAddresses(chainId, env ?? null));
}

export function domainSeparator(chainId: number): string {
  return callSync(() => raw.domainSeparator(chainId));
}

export function eip1271SignaturePayload(
  input: raw.OrderInput,
  ecdsaSignature: string
): WasmEnvelope<string> {
  return callSync(() => raw.eip1271SignaturePayload(input, ecdsaSignature));
}

export function orderTypedData(
  input: raw.OrderInput,
  chainId: number
): WasmEnvelope<raw.TypedDataEnvelopeDto> {
  return callSync(() => raw.orderTypedData(input, chainId));
}

export function signCancellationEthSignDigest(
  orderUids: string[],
  chainId: number,
  digestSigner: DigestSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedCancellationsInput>> {
  return callAsync(() =>
    raw.signCancellationEthSignDigest(orderUids, chainId, digestSigner, options ?? null)
  );
}

export function signCancellationWithEip1193(
  orderUids: string[],
  chainId: number,
  owner: string,
  requestCallback: Eip1193RequestCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedCancellationsInput>> {
  return callAsync(() =>
    raw.signCancellationWithEip1193(orderUids, chainId, owner, requestCallback, options ?? null)
  );
}

export function signCancellationWithTypedDataSigner(
  orderUids: string[],
  chainId: number,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedCancellationsInput>> {
  return callAsync(() =>
    raw.signCancellationWithTypedDataSigner(orderUids, chainId, typedDataSigner, options ?? null)
  );
}

export function signOrderEthSignDigest(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  digestSigner: DigestSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderEthSignDigest(input, chainId, owner, digestSigner, options ?? null)
  );
}

export function signOrderWithCustomEip1271(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  customCallback: CustomEip1271Callback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithCustomEip1271(input, chainId, owner, customCallback, options ?? null)
  );
}

export function signOrderWithEip1193(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  requestCallback: Eip1193RequestCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithEip1193(input, chainId, owner, requestCallback, options ?? null)
  );
}

export function signOrderWithEip1271(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithEip1271(input, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function signOrderWithTypedDataSigner(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithTypedDataSigner(input, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function supportedChainIds(): Uint32Array {
  return callSync(() => raw.supportedChainIds());
}

export function wasmVersion(): string {
  return callSync(() => raw.wasmVersion());
}

export type {
  AppDataObjectDto,
  CompetitionOrderStatusDto,
  CompetitionOrderStatusKindDto,
  CowEip1271SignRequest,
  DeploymentAddressesDto,
  Eip1193Request,
  EthFlowEventDto,
  EthflowDataDto,
  EventLogInput,
  ExecutedAmountsDto,
  ExecutedProtocolFeeDto,
  GeneratedOrderUidDto,
  InteractionDataDto,
  NativePriceResponseDto,
  OnchainOrderDataDto,
  OrderClassDto,
  OrderCreationInput,
  OrderDto,
  OrderInput,
  OrderInteractionsDto,
  OrderKindDto,
  OrderQuoteRequestInput,
  OrderQuoteResponseDto,
  OrderStatusDto,
  OrderTraderParametersInput,
  PaginationOptions,
  QuoteDataDto,
  SettlementEventDto,
  SignedCancellationsInput,
  SignedOrderDto,
  SigningSchemeDto,
  SolverExecutionDto,
  StoredOrderQuoteDto,
  TokenBalanceDto,
  TotalSurplusDto,
  TradeDto,
  TradesQueryInput,
  TransactionRequestDto,
  TypedDataDomainDto,
  TypedDataEnvelopeDto,
  TypedDataFieldDto
} from "./raw/orderbook.js";
export type {
  CowEip1271SignCallback,
  CowEnv,
  CustomEip1271Callback,
  DigestSignerCallback,
  Eip1193RequestCallback,
  TypedDataSignerCallback
} from "./callbacks.js";
export type { OrderBookRejectionCategory, CowError } from "./errors.js";
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
  TransportPolicyConfig,
  WalletConfig
} from "./options.js";
