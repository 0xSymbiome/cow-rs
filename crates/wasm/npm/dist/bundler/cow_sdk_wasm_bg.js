/**
 * IPFS client backed by an explicitly configured HTTP transport.
 */
export class IpfsClient {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IpfsClientFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ipfsclient_free(ptr, 0);
    }
    /**
     * Fetches and parses an app-data document by CID.
     * @param {string} cid
     * @param {SdkClientOptions | null} [options]
     * @returns {WasmEnvelope<AppDataDocDto>}
     */
    fetchAppDataFromCid(cid, options) {
        const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclient_fetchAppDataFromCid(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches and parses an app-data document by app-data hash.
     * @param {string} appDataHex
     * @param {SdkClientOptions | null} [options]
     * @returns {WasmEnvelope<AppDataDocDto>}
     */
    fetchAppDataFromHex(appDataHex, options) {
        const ptr0 = passStringToWasm0(appDataHex, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclient_fetchAppDataFromHex(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Creates an IPFS client from a single config object.
     * @param {IpfsClientConfig} config
     */
    constructor(config) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.ipfsclient_new(retptr, addHeapObject(config));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            IpfsClientFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) IpfsClient.prototype[Symbol.dispose] = IpfsClient.prototype.free;

/**
 * Orderbook client backed by an explicitly configured HTTP transport.
 */
export class OrderBookClient {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OrderBookClientFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_orderbookclient_free(ptr, 0);
    }
    /**
     * Cancels orders through a signed cancellation payload.
     * @param {SignedCancellationsInput} signed
     * @param {SdkClientOptions | null} [options]
     * @returns {WasmEnvelope<{ cancelled: true }>}
     */
    cancelOrders(signed, options) {
        const ret = wasm.orderbookclient_cancelOrders(this.__wbg_ptr, addHeapObject(signed), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches a token's native price.
     * @param {string} token
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getNativePrice(token, options) {
        const ptr0 = passStringToWasm0(token, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getNativePrice(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches an order by UID.
     * @param {string} orderUid
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getOrder(orderUid, options) {
        const ptr0 = passStringToWasm0(orderUid, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getOrder(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches orders owned by an address.
     * @param {string} owner
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getOrdersByOwner(owner, options) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getOrdersByOwner(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches a quote.
     * @param {OrderQuoteRequestInput} request
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getQuote(request, options) {
        const ret = wasm.orderbookclient_getQuote(this.__wbg_ptr, addHeapObject(request), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches trades for an order UID.
     * @param {string} orderUid
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getTrades(orderUid, options) {
        const ptr0 = passStringToWasm0(orderUid, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getTrades(this.__wbg_ptr, ptr0, len0, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Creates an orderbook client from a single config object.
     * @param {OrderBookClientConfig} config
     */
    constructor(config) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.orderbookclient_new(retptr, addHeapObject(config));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            OrderBookClientFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Submits a signed order.
     * @param {SignedOrderDto} signed
     * @param {SdkClientOptions | null} [options]
     * @returns {WasmEnvelope<string>}
     */
    sendOrder(signed, options) {
        const ret = wasm.orderbookclient_sendOrder(this.__wbg_ptr, addHeapObject(signed), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Submits a raw order-creation payload.
     * @param {OrderCreationInput} input
     * @param {SdkClientOptions | null} [options]
     * @returns {WasmEnvelope<string>}
     */
    sendOrderCreation(input, options) {
        const ret = wasm.orderbookclient_sendOrderCreation(this.__wbg_ptr, addHeapObject(input), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
}
if (Symbol.dispose) OrderBookClient.prototype[Symbol.dispose] = OrderBookClient.prototype.free;

/**
 * Subgraph client backed by an explicitly configured HTTP transport.
 */
export class SubgraphClient {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SubgraphClientFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_subgraphclient_free(ptr, 0);
    }
    /**
     * Fetches daily volume rows.
     * @param {number} days
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getLastDaysVolume(days, options) {
        const ret = wasm.subgraphclient_getLastDaysVolume(this.__wbg_ptr, days, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches hourly volume rows.
     * @param {number} hours
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getLastHoursVolume(hours, options) {
        const ret = wasm.subgraphclient_getLastHoursVolume(this.__wbg_ptr, hours, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Fetches aggregate totals.
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getTotals(options) {
        const ret = wasm.subgraphclient_getTotals(this.__wbg_ptr, isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Creates a subgraph client from a single config object.
     * @param {SubgraphClientConfig} config
     */
    constructor(config) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.subgraphclient_new(retptr, addHeapObject(config));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            SubgraphClientFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Runs a raw GraphQL query.
     * @param {SubgraphQueryInput} request
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    runQuery(request, options) {
        const ret = wasm.subgraphclient_runQuery(this.__wbg_ptr, addHeapObject(request), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
}
if (Symbol.dispose) SubgraphClient.prototype[Symbol.dispose] = SubgraphClient.prototype.free;

/**
 * Trading facade backed by an explicitly configured HTTP transport.
 */
export class TradingClient {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TradingClientFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_tradingclient_free(ptr, 0);
    }
    /**
     * Fetches a quote without submitting an order.
     * @param {SwapParametersInput} params
     * @param {SdkClientOptions | null} [options]
     * @returns {Promise<any>}
     */
    getQuote(params, options) {
        const ret = wasm.tradingclient_getQuote(this.__wbg_ptr, addHeapObject(params), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Creates a trading client from a single config object.
     * @param {TradingClientConfig} config
     */
    constructor(config) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.tradingclient_new(retptr, addHeapObject(config));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            TradingClientFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Quotes, signs, and posts a swap order through a typed-data callback.
     * @param {SwapParametersInput} params
     * @param {string} owner
     * @param {TypedDataSignerCallback} signerCallback
     * @param {SigningOptions | null} [options]
     * @returns {Promise<any>}
     */
    postSwapOrder(params, owner, signerCallback, options) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclient_postSwapOrder(this.__wbg_ptr, addHeapObject(params), ptr0, len0, addHeapObject(signerCallback), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
    /**
     * Quotes and posts a swap order with a custom EIP-1271 signature callback.
     * @param {SwapParametersInput} params
     * @param {string} owner
     * @param {CustomEip1271Callback} customCallback
     * @param {SigningOptions | null} [options]
     * @returns {Promise<any>}
     */
    postSwapOrderWithEip1271(params, owner, customCallback, options) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclient_postSwapOrderWithEip1271(this.__wbg_ptr, addHeapObject(params), ptr0, len0, addHeapObject(customCallback), isLikeNone(options) ? 0 : addHeapObject(options));
        return takeObject(ret);
    }
}
if (Symbol.dispose) TradingClient.prototype[Symbol.dispose] = TradingClient.prototype.free;

/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init() {
    wasm.__cow_sdk_wasm_init();
}

/**
 * Builds an app-data document without hashing it.
 * @param {AppDataDocInput} doc
 * @returns {WasmEnvelope<AppDataDocDto>}
 */
export function appDataDoc(doc) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.appDataDoc(retptr, addHeapObject(doc));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Converts an app-data hash to an IPFS CID.
 * @param {string} appDataHex
 * @returns {WasmEnvelope<string>}
 */
export function appDataHexToCid(appDataHex) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(appDataHex, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.appDataHexToCid(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Returns deterministic app-data content, hash, and CID.
 * @param {AppDataDocInput} doc
 * @returns {WasmEnvelope<AppDataInfoDto>}
 */
export function appDataInfo(doc) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.appDataInfo(retptr, addHeapObject(doc));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Converts an IPFS CID to an app-data hash.
 * @param {string} cid
 * @returns {WasmEnvelope<string>}
 */
export function cidToAppDataHex(cid) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.cidToAppDataHex(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Computes the compact order UID and digest.
 * @param {OrderInput} input
 * @param {number} chainId
 * @param {string} owner
 * @returns {WasmEnvelope<GeneratedOrderUidDto>}
 */
export function computeOrderUid(input, chainId, owner) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.computeOrderUid(retptr, addHeapObject(input), chainId, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Returns canonical deployment addresses for a chain and environment.
 * @param {number} chainId
 * @param {string | null} [env]
 * @returns {WasmEnvelope<DeploymentAddressesDto>}
 */
export function deploymentAddresses(chainId, env) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        var len0 = WASM_VECTOR_LEN;
        wasm.deploymentAddresses(retptr, chainId, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Computes the EIP-712 domain separator for a supported chain.
 * @param {number} chainId
 * @returns {string}
 */
export function domainSeparator(chainId) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.domainSeparator(retptr, chainId);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        var ptr1 = r0;
        var len1 = r1;
        if (r3) {
            ptr1 = 0; len1 = 0;
            throw takeObject(r2);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA signature.
 * @param {OrderInput} input
 * @param {string} ecdsaSignature
 * @returns {WasmEnvelope<string>}
 */
export function eip1271SignaturePayload(input, ecdsaSignature) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(ecdsaSignature, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.eip1271SignaturePayload(retptr, addHeapObject(input), ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Builds signer-facing order typed data.
 * @param {OrderInput} input
 * @param {number} chainId
 * @returns {WasmEnvelope<TypedDataEnvelopeDto>}
 */
export function orderTypedData(input, chainId) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.orderTypedData(retptr, addHeapObject(input), chainId);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Signs a cancellation digest through an explicit `eth_sign` callback.
 * @param {string[]} orderUids
 * @param {number} chainId
 * @param {DigestSignerCallback} digestSigner
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedCancellationsInput>}
 */
export function signCancellationEthSignDigest(orderUids, chainId, digestSigner, options) {
    const ptr0 = passArrayJsValueToWasm0(orderUids, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signCancellationEthSignDigest(ptr0, len0, chainId, addHeapObject(digestSigner), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs cancellation typed data through an EIP-1193 callback.
 * @param {string[]} orderUids
 * @param {number} chainId
 * @param {string} owner
 * @param {Eip1193RequestCallback} requestCallback
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedCancellationsInput>}
 */
export function signCancellationWithEip1193(orderUids, chainId, owner, requestCallback, options) {
    const ptr0 = passArrayJsValueToWasm0(orderUids, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.signCancellationWithEip1193(ptr0, len0, chainId, ptr1, len1, addHeapObject(requestCallback), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs cancellation typed data through a typed-data callback.
 * @param {string[]} orderUids
 * @param {number} chainId
 * @param {TypedDataSignerCallback} typedDataSigner
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedCancellationsInput>}
 */
export function signCancellationWithTypedDataSigner(orderUids, chainId, typedDataSigner, options) {
    const ptr0 = passArrayJsValueToWasm0(orderUids, wasm.__wbindgen_export);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signCancellationWithTypedDataSigner(ptr0, len0, chainId, addHeapObject(typedDataSigner), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 * @param {OrderInput} input
 * @param {number} chainId
 * @param {string} owner
 * @param {DigestSignerCallback} digestSigner
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedOrderDto>}
 */
export function signOrderEthSignDigest(input, chainId, owner, digestSigner, options) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderEthSignDigest(addHeapObject(input), chainId, ptr0, len0, addHeapObject(digestSigner), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs an order through a custom EIP-1271 callback.
 * @param {OrderInput} input
 * @param {number} chainId
 * @param {string} owner
 * @param {CustomEip1271Callback} customCallback
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedOrderDto>}
 */
export function signOrderWithCustomEip1271(input, chainId, owner, customCallback, options) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithCustomEip1271(addHeapObject(input), chainId, ptr0, len0, addHeapObject(customCallback), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs an order through an EIP-1193 request callback.
 * @param {OrderInput} input
 * @param {number} chainId
 * @param {string} owner
 * @param {Eip1193RequestCallback} requestCallback
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedOrderDto>}
 */
export function signOrderWithEip1193(input, chainId, owner, requestCallback, options) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithEip1193(addHeapObject(input), chainId, ptr0, len0, addHeapObject(requestCallback), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 * @param {OrderInput} input
 * @param {number} chainId
 * @param {string} owner
 * @param {TypedDataSignerCallback} typedDataSigner
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedOrderDto>}
 */
export function signOrderWithEip1271(input, chainId, owner, typedDataSigner, options) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithEip1271(addHeapObject(input), chainId, ptr0, len0, addHeapObject(typedDataSigner), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Signs an order through a typed-data callback.
 * @param {OrderInput} input
 * @param {number} chainId
 * @param {string} owner
 * @param {TypedDataSignerCallback} typedDataSigner
 * @param {SigningOptions | null} [options]
 * @returns {WasmEnvelope<SignedOrderDto>}
 */
export function signOrderWithTypedDataSigner(input, chainId, owner, typedDataSigner, options) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithTypedDataSigner(addHeapObject(input), chainId, ptr0, len0, addHeapObject(typedDataSigner), isLikeNone(options) ? 0 : addHeapObject(options));
    return takeObject(ret);
}

/**
 * Returns supported EVM chain ids.
 * @returns {Uint32Array}
 */
export function supportedChainIds() {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.supportedChainIds(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var v1 = getArrayU32FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export4(r0, r1 * 4, 4);
        return v1;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Validates an app-data document against the embedded schemas.
 * @param {AppDataDocInput} doc
 * @returns {WasmEnvelope<ValidationResultDto>}
 */
export function validateAppDataDoc(doc) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.validateAppDataDoc(retptr, addHeapObject(doc));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Returns the wasm crate version.
 * @returns {string}
 */
export function wasmVersion() {
    let deferred1_0;
    let deferred1_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.wasmVersion(retptr);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred1_0 = r0;
        deferred1_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
    }
}
export function __wbg_Error_960c155d3d49e4c2(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return addHeapObject(ret);
}
export function __wbg_Number_32bf70a599af1d4b(arg0) {
    const ret = Number(getObject(arg0));
    return ret;
}
export function __wbg_String_8564e559799eccda(arg0, arg1) {
    const ret = String(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_bigint_get_as_i64_3d3aba5d616c6a51(arg0, arg1) {
    const v = getObject(arg1);
    const ret = typeof(v) === 'bigint' ? v : undefined;
    getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
}
export function __wbg___wbindgen_boolean_get_6ea149f0a8dcc5ff(arg0) {
    const v = getObject(arg0);
    const ret = typeof(v) === 'boolean' ? v : undefined;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
}
export function __wbg___wbindgen_debug_string_ab4b34d23d6778bd(arg0, arg1) {
    const ret = debugString(getObject(arg1));
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_in_a5d8b22e52b24dd1(arg0, arg1) {
    const ret = getObject(arg0) in getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_is_bigint_ec25c7f91b4d9e93(arg0) {
    const ret = typeof(getObject(arg0)) === 'bigint';
    return ret;
}
export function __wbg___wbindgen_is_function_3baa9db1a987f47d(arg0) {
    const ret = typeof(getObject(arg0)) === 'function';
    return ret;
}
export function __wbg___wbindgen_is_null_52ff4ec04186736f(arg0) {
    const ret = getObject(arg0) === null;
    return ret;
}
export function __wbg___wbindgen_is_object_63322ec0cd6ea4ef(arg0) {
    const val = getObject(arg0);
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
}
export function __wbg___wbindgen_is_string_6df3bf7ef1164ed3(arg0) {
    const ret = typeof(getObject(arg0)) === 'string';
    return ret;
}
export function __wbg___wbindgen_is_undefined_29a43b4d42920abd(arg0) {
    const ret = getObject(arg0) === undefined;
    return ret;
}
export function __wbg___wbindgen_jsval_eq_d3465d8a07697228(arg0, arg1) {
    const ret = getObject(arg0) === getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_jsval_loose_eq_cac3565e89b4134c(arg0, arg1) {
    const ret = getObject(arg0) == getObject(arg1);
    return ret;
}
export function __wbg___wbindgen_number_get_c7f42aed0525c451(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
}
export function __wbg___wbindgen_string_get_7ed5322991caaec5(arg0, arg1) {
    const obj = getObject(arg1);
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_throw_6b64449b9b9ed33c(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
}
export function __wbg__wbg_cb_unref_b46c9b5a9f08ec37(arg0) {
    getObject(arg0)._wbg_cb_unref();
}
export function __wbg_abort_79db88f743c3efd7(arg0) {
    getObject(arg0).abort();
}
export function __wbg_aborted_072e5543dce0fc05(arg0) {
    const ret = getObject(arg0).aborted;
    return ret;
}
export function __wbg_addEventListener_8176dab41b09531c() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).addEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments); }
export function __wbg_call_14b169f759b26747() { return handleError(function (arg0, arg1) {
    const ret = getObject(arg0).call(getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_call_a24592a6f349a97e() { return handleError(function (arg0, arg1, arg2) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_call_bb28efe6b2f55b86() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = getObject(arg0).call(getObject(arg1), getObject(arg2), getObject(arg3));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_clearTimeout_3629d6209dfcc46e(arg0) {
    const ret = clearTimeout(takeObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_clearTimeout_a5b2d1f832c8c5b6(arg0) {
    globalThis.clearTimeout(getObject(arg0));
}
export function __wbg_done_9158f7cc8751ba32(arg0) {
    const ret = getObject(arg0).done;
    return ret;
}
export function __wbg_entries_e0b73aa8571ddb56(arg0) {
    const ret = Object.entries(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_error_a6fa202b58aa1cd3(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
    }
}
export function __wbg_from_0dbf29f09e7fb200(arg0) {
    const ret = Array.from(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_getRandomValues_3f44b700395062e5() { return handleError(function (arg0, arg1) {
    globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
}, arguments); }
export function __wbg_get_1affdbdd5573b16a() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_get_6011fa3a58f61074() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(getObject(arg0), getObject(arg1));
    return addHeapObject(ret);
}, arguments); }
export function __wbg_get_8360291721e2339f(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
}
export function __wbg_get_unchecked_17f53dad852b9588(arg0, arg1) {
    const ret = getObject(arg0)[arg1 >>> 0];
    return addHeapObject(ret);
}
export function __wbg_get_with_ref_key_6412cf3094599694(arg0, arg1) {
    const ret = getObject(arg0)[getObject(arg1)];
    return addHeapObject(ret);
}
export function __wbg_headers_6022deb4e576fb8e(arg0) {
    const ret = getObject(arg0).headers;
    return addHeapObject(ret);
}
export function __wbg_instanceof_AbortSignal_6e9e35050fcd6a9a(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof AbortSignal;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_ArrayBuffer_7c8433c6ed14ffe3(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Map_1b76fd4635be43eb(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Map;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Response_9b2d111407865ff2(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Response;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_instanceof_Uint8Array_152ba1f289edcf3f(arg0) {
    let result;
    try {
        result = getObject(arg0) instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
}
export function __wbg_isArray_c3109d14ffc06469(arg0) {
    const ret = Array.isArray(getObject(arg0));
    return ret;
}
export function __wbg_isSafeInteger_4fc213d1989d6d2a(arg0) {
    const ret = Number.isSafeInteger(getObject(arg0));
    return ret;
}
export function __wbg_iterator_013bc09ec998c2a7() {
    const ret = Symbol.iterator;
    return addHeapObject(ret);
}
export function __wbg_length_3d4ecd04bd8d22f1(arg0) {
    const ret = getObject(arg0).length;
    return ret;
}
export function __wbg_length_9f1775224cf1d815(arg0) {
    const ret = getObject(arg0).length;
    return ret;
}
export function __wbg_new_036bd6cd9cea9e73(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wasm_bindgen_func_elem_11349(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = 0;
    }
}
export function __wbg_new_0c7403db6e782f19(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_new_227d7c05414eb861() {
    const ret = new Error();
    return addHeapObject(ret);
}
export function __wbg_new_34d45cc8e36aaead() {
    const ret = new Map();
    return addHeapObject(ret);
}
export function __wbg_new_682678e2f47e32bc() {
    const ret = new Array();
    return addHeapObject(ret);
}
export function __wbg_new_aa8d0fa9762c29bd() {
    const ret = new Object();
    return addHeapObject(ret);
}
export function __wbg_new_d3704878df906b51() {
    const ret = new globalThis.AbortController();
    return addHeapObject(ret);
}
export function __wbg_new_typed_323f37fd55ab048d(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wasm_bindgen_func_elem_11349(a, state0.b, arg0, arg1);
            } finally {
                state0.a = a;
            }
        };
        const ret = new Promise(cb0);
        return addHeapObject(ret);
    } finally {
        state0.a = 0;
    }
}
export function __wbg_next_0340c4ae324393c3() { return handleError(function (arg0) {
    const ret = getObject(arg0).next();
    return addHeapObject(ret);
}, arguments); }
export function __wbg_next_7646edaa39458ef7(arg0) {
    const ret = getObject(arg0).next;
    return addHeapObject(ret);
}
export function __wbg_now_a9b7df1cbee90986() {
    const ret = Date.now();
    return ret;
}
export function __wbg_now_e7c6795a7f81e10f(arg0) {
    const ret = getObject(arg0).now();
    return ret;
}
export function __wbg_performance_3fcf6e32a7e1ed0a(arg0) {
    const ret = getObject(arg0).performance;
    return addHeapObject(ret);
}
export function __wbg_prototypesetcall_a6b02eb00b0f4ce2(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
}
export function __wbg_push_471a5b068a5295f6(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
}
export function __wbg_queueMicrotask_5d15a957e6aa920e(arg0) {
    queueMicrotask(getObject(arg0));
}
export function __wbg_queueMicrotask_f8819e5ffc402f36(arg0) {
    const ret = getObject(arg0).queueMicrotask;
    return addHeapObject(ret);
}
export function __wbg_race_04dca79de55bb877(arg0) {
    const ret = Promise.race(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_removeEventListener_7bdf07404d9b24bd() { return handleError(function (arg0, arg1, arg2, arg3) {
    getObject(arg0).removeEventListener(getStringFromWasm0(arg1, arg2), getObject(arg3));
}, arguments); }
export function __wbg_resolve_e6c466bc1052f16c(arg0) {
    const ret = Promise.resolve(getObject(arg0));
    return addHeapObject(ret);
}
export function __wbg_setTimeout_3b32677b3fda46e8(arg0, arg1) {
    const ret = globalThis.setTimeout(getObject(arg0), arg1 >>> 0);
    return addHeapObject(ret);
}
export function __wbg_setTimeout_56bcdccbad22fd44() { return handleError(function (arg0, arg1) {
    const ret = setTimeout(getObject(arg0), arg1);
    return addHeapObject(ret);
}, arguments); }
export function __wbg_set_022bee52d0b05b19() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
    return ret;
}, arguments); }
export function __wbg_set_3bf1de9fab0cd644(arg0, arg1, arg2) {
    getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
}
export function __wbg_set_6be42768c690e380(arg0, arg1, arg2) {
    getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
}
export function __wbg_set_fde2cec06c23692b(arg0, arg1, arg2) {
    const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}
export function __wbg_signal_b74e34a36211c513(arg0) {
    const ret = getObject(arg0).signal;
    return addHeapObject(ret);
}
export function __wbg_stack_3b0d974bbf31e44f(arg0, arg1) {
    const ret = getObject(arg1).stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg_static_accessor_GLOBAL_8cfadc87a297ca02() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}
export function __wbg_static_accessor_GLOBAL_THIS_602256ae5c8f42cf() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}
export function __wbg_static_accessor_SELF_e445c1c7484aecc3() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}
export function __wbg_static_accessor_WINDOW_f20e8576ef1e0f17() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addHeapObject(ret);
}
export function __wbg_statusText_a0c2afa453245983(arg0, arg1) {
    const ret = getObject(arg1).statusText;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg_status_43e0d2f15b22d69f(arg0) {
    const ret = getObject(arg0).status;
    return ret;
}
export function __wbg_text_595ef75535aa25c1() { return handleError(function (arg0) {
    const ret = getObject(arg0).text();
    return addHeapObject(ret);
}, arguments); }
export function __wbg_then_792e0c862b060889(arg0, arg1, arg2) {
    const ret = getObject(arg0).then(getObject(arg1), getObject(arg2));
    return addHeapObject(ret);
}
export function __wbg_then_8e16ee11f05e4827(arg0, arg1) {
    const ret = getObject(arg0).then(getObject(arg1));
    return addHeapObject(ret);
}
export function __wbg_value_ee3a06f4579184fa(arg0) {
    const ret = getObject(arg0).value;
    return addHeapObject(ret);
}
export function __wbg_warn_3cc416af27dbdc02(arg0) {
    console.warn(getObject(arg0));
}
export function __wbindgen_cast_0000000000000001(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 2232, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_11341);
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000002(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 6, ret: Externref, inner_ret: Some(Externref) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_1158);
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000003(arg0, arg1) {
    // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 2225, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
    const ret = makeMutClosure(arg0, arg1, __wasm_bindgen_func_elem_11262);
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000004(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000005(arg0) {
    // Cast intrinsic for `I64 -> Externref`.
    const ret = arg0;
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000006(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
}
export function __wbindgen_cast_0000000000000007(arg0) {
    // Cast intrinsic for `U64 -> Externref`.
    const ret = BigInt.asUintN(64, arg0);
    return addHeapObject(ret);
}
export function __wbindgen_object_clone_ref(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
}
export function __wbindgen_object_drop_ref(arg0) {
    takeObject(arg0);
}
function __wasm_bindgen_func_elem_11262(arg0, arg1) {
    wasm.__wasm_bindgen_func_elem_11262(arg0, arg1);
}

function __wasm_bindgen_func_elem_1158(arg0, arg1, arg2) {
    const ret = wasm.__wasm_bindgen_func_elem_1158(arg0, arg1, addHeapObject(arg2));
    return takeObject(ret);
}

function __wasm_bindgen_func_elem_11341(arg0, arg1, arg2) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.__wasm_bindgen_func_elem_11341(retptr, arg0, arg1, addHeapObject(arg2));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

function __wasm_bindgen_func_elem_11349(arg0, arg1, arg2, arg3) {
    wasm.__wasm_bindgen_func_elem_11349(arg0, arg1, addHeapObject(arg2), addHeapObject(arg3));
}

const IpfsClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ipfsclient_free(ptr >>> 0, 1));
const OrderBookClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_orderbookclient_free(ptr >>> 0, 1));
const SubgraphClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_subgraphclient_free(ptr >>> 0, 1));
const TradingClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_tradingclient_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_export5(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, f) {
    const state = { a: arg0, b: arg1, cnt: 1 };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            wasm.__wbindgen_export5(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addHeapObject(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;


let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}
