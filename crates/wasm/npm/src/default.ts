import * as raw from "./raw/default.js";
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
import type {
  CommonClientConfig,
  IpfsClientConfig,
  SdkClientOptions,
  SigningOptions,
  SubgraphClientConfig,
  TradingClientConfig
} from "./options.js";
import type {
  ContractReadCallback,
  CustomEip1271Callback,
  DigestSignerCallback,
  Eip1193RequestCallback,
  TypedDataSignerCallback
} from "./callbacks.js";

export interface OrderBookClientConfig extends CommonClientConfig {}

export class IpfsClient {
  #inner: InstanceType<typeof raw.RawIpfsClient>;
  #defaults: ClientDefaults;
  #disposed = false;

  constructor(config: IpfsClientConfig) {
    try {
      this.#defaults = defaultsFrom(config);
      this.#inner = new raw.RawIpfsClient(
        translateClientConfig(config) as unknown as raw.IpfsClientConfig
      );
    } catch (error) {
      throw callSync(() => {
        throw error;
      });
    }
  }

  async fetchAppDataFromCid(
    cid: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.AppDataDocDto>> {
    return this.#call((client, merged) => client.fetchAppDataFromCid(cid, merged), options);
  }

  async fetchAppDataFromHex(
    appDataHex: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.AppDataDocDto>> {
    return this.#call((client, merged) => client.fetchAppDataFromHex(appDataHex, merged), options);
  }

  dispose(): void {
    if (!this.#disposed) {
      disposeRaw(this.#inner);
      this.#disposed = true;
    }
  }

  #call<T>(
    operation: (
      client: InstanceType<typeof raw.RawIpfsClient>,
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
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getNativePrice(token, merged), options);
  }

  async getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getOrder(orderUid, merged), options);
  }

  async getOrders(
    owner: string,
    pagination?: raw.PaginationOptions | null,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getOrders(owner, pagination ?? null, merged), options);
  }

  async getOrdersByOwner(
    owner: string,
    pagination?: raw.PaginationOptions | null,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call(
      (client, merged) => client.getOrdersByOwner(owner, pagination ?? null, merged),
      options
    );
  }

  async getQuote(
    request: raw.OrderQuoteRequestInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getQuote(request, merged), options);
  }

  async getTrades(
    query: raw.TradesQueryInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getTrades(query, merged), options);
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

export class SubgraphClient {
  #inner: InstanceType<typeof raw.RawSubgraphClient>;
  #defaults: ClientDefaults;
  #disposed = false;

  constructor(config: SubgraphClientConfig) {
    try {
      this.#defaults = defaultsFrom(config);
      this.#inner = new raw.RawSubgraphClient(
        translateClientConfig(config) as unknown as raw.SubgraphClientConfig
      );
    } catch (error) {
      throw callSync(() => {
        throw error;
      });
    }
  }

  async getLastDaysVolume(days: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getLastDaysVolume(days, merged), options);
  }

  async getLastHoursVolume(hours: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getLastHoursVolume(hours, merged), options);
  }

  async getTotals(options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getTotals(merged), options);
  }

  async runQuery(
    request: raw.SubgraphQueryInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.runQuery(request, merged), options);
  }

  dispose(): void {
    if (!this.#disposed) {
      disposeRaw(this.#inner);
      this.#disposed = true;
    }
  }

  #call<T>(
    operation: (
      client: InstanceType<typeof raw.RawSubgraphClient>,
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
    order: raw.OrderInput,
    quoteId: bigint,
    from: string,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<raw.BuiltSellNativeCurrencyTxDto>> {
    return this.#call(
      (client, merged) => client.buildSellNativeCurrencyTx(order, quoteId, from, merged),
      options
    );
  }

  async getCowProtocolAllowance(
    params: raw.AllowanceParametersInput,
    readContractCallback: ContractReadCallback,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<string>> {
    return this.#call(
      (client, merged) => client.getCowProtocolAllowance(params, readContractCallback, merged),
      options
    );
  }

  async getQuote(
    params: raw.SwapParametersInput,
    options?: SdkClientOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#call((client, merged) => client.getQuote(params, merged), options);
  }

  async postLimitOrder(
    params: raw.LimitTradeParametersInput,
    owner: string,
    signerCallback: TypedDataSignerCallback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#callSigning((client) =>
      client.postLimitOrder(params, owner, signerCallback, options ?? null)
    );
  }

  async postSwapOrder(
    params: raw.SwapParametersInput,
    owner: string,
    signerCallback: TypedDataSignerCallback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#callSigning((client) =>
      client.postSwapOrder(params, owner, signerCallback, options ?? null)
    );
  }

  async postSwapOrderFromQuote(
    quoteResults: raw.QuoteResultsInput,
    owner: string,
    signerCallback: TypedDataSignerCallback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#callSigning((client) =>
      client.postSwapOrderFromQuote(quoteResults, owner, signerCallback, options ?? null)
    );
  }

  async postSwapOrderWithEip1271(
    params: raw.SwapParametersInput,
    owner: string,
    customCallback: CustomEip1271Callback,
    options?: SigningOptions | null
  ): Promise<WasmEnvelope<unknown>> {
    return this.#callSigning((client) =>
      client.postSwapOrderWithEip1271(params, owner, customCallback, options ?? null)
    );
  }

  dispose(): void {
    if (!this.#disposed) {
      disposeRaw(this.#inner);
      this.#disposed = true;
    }
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
    operation: (client: InstanceType<typeof raw.RawTradingClient>) => Promise<T>
  ): Promise<T> {
    return callAsync(() => {
      assertActive(this.#disposed);
      return operation(this.#inner);
    });
  }
}

export function appDataDoc(doc: raw.AppDataDocInput): WasmEnvelope<raw.AppDataDocDto> {
  return callSync(() => raw.appDataDoc(doc));
}

export function appDataHexToCid(appDataHex: string): WasmEnvelope<string> {
  return callSync(() => raw.appDataHexToCid(appDataHex));
}

export function appDataInfo(doc: raw.AppDataDocInput): WasmEnvelope<raw.AppDataInfoDto> {
  return callSync(() => raw.appDataInfo(doc));
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

export function cidToAppDataHex(cid: string): WasmEnvelope<string> {
  return callSync(() => raw.cidToAppDataHex(cid));
}

export function computeOrderUid(
  input: raw.OrderInput,
  chainId: number,
  owner: string
): WasmEnvelope<raw.GeneratedOrderUidDto> {
  return callSync(() => raw.computeOrderUid(input, chainId, owner));
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

export function validateAppDataDoc(doc: raw.AppDataDocInput): WasmEnvelope<raw.ValidationResultDto> {
  return callSync(() => raw.validateAppDataDoc(doc));
}

export function wasmVersion(): string {
  return callSync(() => raw.wasmVersion());
}

export type {
  AllowanceParametersInput,
  AppDataDocDto,
  AppDataDocInput,
  AppDataInfoDto,
  BuiltSellNativeCurrencyTxDto,
  ContractCallDto,
  CowEip1271SignRequest,
  DeploymentAddressesDto,
  Eip1193Request,
  GeneratedOrderUidDto,
  LimitTradeParametersInput,
  OrderCreationInput,
  OrderInput,
  OrderKindDto,
  OrderQuoteRequestInput,
  OrderTraderParametersInput,
  PaginationOptions,
  PartnerFeeInput,
  PartnerFeePolicyInput,
  QuoteResponseRefInput,
  QuoteResultsInput,
  SignedCancellationsInput,
  SignedOrderDto,
  SubgraphQueryInput,
  SwapParametersInput,
  TokenBalanceDto,
  TradesQueryInput,
  TransactionRequestDto,
  TypedDataDomainDto,
  TypedDataEnvelopeDto,
  TypedDataFieldDto,
  ValidationResultDto
} from "./raw/default.js";
export type {
  ContractReadCallback,
  CowEip1271SignCallback,
  CustomEip1271Callback,
  DigestSignerCallback,
  Eip1193RequestCallback,
  TypedDataSignerCallback
} from "./callbacks.js";
export type { SdkError } from "./errors.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type {
  HttpTransportConfig,
  IpfsClientConfig,
  SdkClientOptions,
  SigningOptions,
  SubgraphClientConfig,
  TradingClientConfig,
  TransportPolicyConfig,
  WalletConfig
} from "./options.js";
