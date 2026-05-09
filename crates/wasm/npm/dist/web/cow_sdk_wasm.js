/* @ts-self-types="./cow_sdk_wasm.d.ts" */

/**
 * Disposable callback registry handle.
 */
export class FetchCallbackHandle {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(FetchCallbackHandle.prototype);
        obj.__wbg_ptr = ptr;
        FetchCallbackHandleFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FetchCallbackHandleFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fetchcallbackhandle_free(ptr, 0);
    }
    /**
     * Disposes this callback registration. Calling this more than once is harmless.
     */
    dispose() {
        wasm.fetchcallbackhandle_dispose(this.__wbg_ptr);
    }
    /**
     * Numeric callback id.
     * @returns {number}
     */
    get id() {
        const ret = wasm.fetchcallbackhandle_id(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) FetchCallbackHandle.prototype[Symbol.dispose] = FetchCallbackHandle.prototype.free;

/**
 * Adapter that lets app-data IPFS reads flow through an HTTP transport.
 */
export class HttpToIpfsAdapter {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(HttpToIpfsAdapter.prototype);
        obj.__wbg_ptr = ptr;
        HttpToIpfsAdapterFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        HttpToIpfsAdapterFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_httptoipfsadapter_free(ptr, 0);
    }
    /**
     * Fetches and parses an app-data document by CID.
     * @param {string} cid
     * @param {string | null} [ipfs_uri]
     * @returns {Promise<any>}
     */
    fetchAppDataFromCid(cid, ipfs_uri) {
        const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.httptoipfsadapter_fetchAppDataFromCid(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Fetches and parses an app-data document by app-data hash.
     * @param {string} app_data_hex
     * @param {string | null} [ipfs_uri]
     * @returns {Promise<any>}
     */
    fetchAppDataFromHex(app_data_hex, ipfs_uri) {
        const ptr0 = passStringToWasm0(app_data_hex, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.httptoipfsadapter_fetchAppDataFromHex(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Creates an adapter from an existing fetch-callback handle id.
     * @param {number} fetch_callback_id
     * @param {number | null} [timeout_ms]
     * @returns {HttpToIpfsAdapter}
     */
    static fromHandle(fetch_callback_id, timeout_ms) {
        const ret = wasm.httptoipfsadapter_fromHandle(fetch_callback_id, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return HttpToIpfsAdapter.__wrap(ret[0]);
    }
    /**
     * Creates an adapter that owns a registered fetch callback.
     * @param {Function} fetch_callback
     * @param {number | null} [timeout_ms]
     */
    constructor(fetch_callback, timeout_ms) {
        const ret = wasm.httptoipfsadapter_new(fetch_callback, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        HttpToIpfsAdapterFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) HttpToIpfsAdapter.prototype[Symbol.dispose] = HttpToIpfsAdapter.prototype.free;

/**
 * IPFS client backed by the browser fetch transport.
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
     * @returns {Promise<any>}
     */
    fetchAppDataFromCid(cid) {
        const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclient_fetchAppDataFromCid(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches and parses an app-data document by app-data hash.
     * @param {string} app_data_hex
     * @returns {Promise<any>}
     */
    fetchAppDataFromHex(app_data_hex) {
        const ptr0 = passStringToWasm0(app_data_hex, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclient_fetchAppDataFromHex(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Creates an IPFS client with the default browser fetch transport.
     * @param {string | null} [ipfs_uri]
     * @param {number | null} [timeout_ms]
     */
    constructor(ipfs_uri, timeout_ms) {
        var ptr0 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclient_new(ptr0, len0, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        IpfsClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) IpfsClient.prototype[Symbol.dispose] = IpfsClient.prototype.free;

/**
 * IPFS client backed by a JavaScript fetch callback.
 */
export class IpfsClientWithFetch {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(IpfsClientWithFetch.prototype);
        obj.__wbg_ptr = ptr;
        IpfsClientWithFetchFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        IpfsClientWithFetchFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_ipfsclientwithfetch_free(ptr, 0);
    }
    /**
     * Fetches and parses an app-data document by CID.
     * @param {string} cid
     * @returns {Promise<any>}
     */
    fetchAppDataFromCid(cid) {
        const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclientwithfetch_fetchAppDataFromCid(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches and parses an app-data document by app-data hash.
     * @param {string} app_data_hex
     * @returns {Promise<any>}
     */
    fetchAppDataFromHex(app_data_hex) {
        const ptr0 = passStringToWasm0(app_data_hex, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclientwithfetch_fetchAppDataFromHex(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Creates an IPFS client from an existing fetch-callback handle id.
     * @param {string | null | undefined} ipfs_uri
     * @param {number | null | undefined} timeout_ms
     * @param {number} fetch_callback_id
     * @returns {IpfsClientWithFetch}
     */
    static fromHandle(ipfs_uri, timeout_ms, fetch_callback_id) {
        var ptr0 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclientwithfetch_fromHandle(ptr0, len0, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0, fetch_callback_id);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return IpfsClientWithFetch.__wrap(ret[0]);
    }
    /**
     * Creates an IPFS client that owns a registered fetch callback.
     * @param {string | null | undefined} ipfs_uri
     * @param {number | null | undefined} timeout_ms
     * @param {Function} fetch_callback
     */
    constructor(ipfs_uri, timeout_ms, fetch_callback) {
        var ptr0 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.ipfsclientwithfetch_new(ptr0, len0, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0, fetch_callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        IpfsClientWithFetchFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) IpfsClientWithFetch.prototype[Symbol.dispose] = IpfsClientWithFetch.prototype.free;

/**
 * Orderbook client backed by the browser fetch transport.
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
     * @returns {Promise<any>}
     */
    cancelOrders(signed) {
        const ret = wasm.orderbookclient_cancelOrders(this.__wbg_ptr, signed);
        return ret;
    }
    /**
     * Fetches a token's native price.
     * @param {string} token
     * @returns {Promise<any>}
     */
    getNativePrice(token) {
        const ptr0 = passStringToWasm0(token, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getNativePrice(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches an order by UID.
     * @param {string} order_uid
     * @returns {Promise<any>}
     */
    getOrder(order_uid) {
        const ptr0 = passStringToWasm0(order_uid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getOrder(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches orders owned by an address.
     * @param {string} owner
     * @returns {Promise<any>}
     */
    getOrdersByOwner(owner) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getOrdersByOwner(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches a quote.
     * @param {OrderQuoteRequestInput} request
     * @returns {Promise<any>}
     */
    getQuote(request) {
        const ret = wasm.orderbookclient_getQuote(this.__wbg_ptr, request);
        return ret;
    }
    /**
     * Fetches trades for an order UID.
     * @param {string} order_uid
     * @returns {Promise<any>}
     */
    getTrades(order_uid) {
        const ptr0 = passStringToWasm0(order_uid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_getTrades(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Creates an orderbook client for a chain and environment.
     * @param {number} chain_id
     * @param {string | null} [env]
     */
    constructor(chain_id, env) {
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclient_new(chain_id, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        OrderBookClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Submits a signed order.
     * @param {SignedOrderDto} signed
     * @returns {Promise<string>}
     */
    sendOrder(signed) {
        const ret = wasm.orderbookclient_sendOrder(this.__wbg_ptr, signed);
        return ret;
    }
    /**
     * Submits a raw order-creation payload.
     * @param {OrderCreationInput} input
     * @returns {Promise<string>}
     */
    sendOrderCreation(input) {
        const ret = wasm.orderbookclient_sendOrderCreation(this.__wbg_ptr, input);
        return ret;
    }
}
if (Symbol.dispose) OrderBookClient.prototype[Symbol.dispose] = OrderBookClient.prototype.free;

/**
 * Orderbook client backed by a JavaScript fetch callback.
 */
export class OrderBookClientWithFetch {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(OrderBookClientWithFetch.prototype);
        obj.__wbg_ptr = ptr;
        OrderBookClientWithFetchFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OrderBookClientWithFetchFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_orderbookclientwithfetch_free(ptr, 0);
    }
    /**
     * Cancels orders through a signed cancellation payload.
     * @param {SignedCancellationsInput} signed
     * @returns {Promise<any>}
     */
    cancelOrders(signed) {
        const ret = wasm.orderbookclientwithfetch_cancelOrders(this.__wbg_ptr, signed);
        return ret;
    }
    /**
     * Creates an orderbook client from an existing fetch-callback handle id.
     * @param {number} chain_id
     * @param {string | null | undefined} env
     * @param {number} fetch_callback_id
     * @returns {OrderBookClientWithFetch}
     */
    static fromHandle(chain_id, env, fetch_callback_id) {
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclientwithfetch_fromHandle(chain_id, ptr0, len0, fetch_callback_id);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return OrderBookClientWithFetch.__wrap(ret[0]);
    }
    /**
     * Fetches a token's native price.
     * @param {string} token
     * @returns {Promise<any>}
     */
    getNativePrice(token) {
        const ptr0 = passStringToWasm0(token, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclientwithfetch_getNativePrice(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches an order by UID.
     * @param {string} order_uid
     * @returns {Promise<any>}
     */
    getOrder(order_uid) {
        const ptr0 = passStringToWasm0(order_uid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclientwithfetch_getOrder(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches orders owned by an address.
     * @param {string} owner
     * @returns {Promise<any>}
     */
    getOrdersByOwner(owner) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclientwithfetch_getOrdersByOwner(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Fetches a quote.
     * @param {OrderQuoteRequestInput} request
     * @returns {Promise<any>}
     */
    getQuote(request) {
        const ret = wasm.orderbookclientwithfetch_getQuote(this.__wbg_ptr, request);
        return ret;
    }
    /**
     * Fetches trades for an order UID.
     * @param {string} order_uid
     * @returns {Promise<any>}
     */
    getTrades(order_uid) {
        const ptr0 = passStringToWasm0(order_uid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclientwithfetch_getTrades(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Creates an orderbook client that owns a registered fetch callback.
     * @param {number} chain_id
     * @param {string | null | undefined} env
     * @param {Function} fetch_callback
     */
    constructor(chain_id, env, fetch_callback) {
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ret = wasm.orderbookclientwithfetch_new(chain_id, ptr0, len0, fetch_callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        OrderBookClientWithFetchFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Submits a signed order.
     * @param {SignedOrderDto} signed
     * @returns {Promise<string>}
     */
    sendOrder(signed) {
        const ret = wasm.orderbookclientwithfetch_sendOrder(this.__wbg_ptr, signed);
        return ret;
    }
    /**
     * Submits a raw order-creation payload.
     * @param {OrderCreationInput} input
     * @returns {Promise<string>}
     */
    sendOrderCreation(input) {
        const ret = wasm.orderbookclientwithfetch_sendOrderCreation(this.__wbg_ptr, input);
        return ret;
    }
}
if (Symbol.dispose) OrderBookClientWithFetch.prototype[Symbol.dispose] = OrderBookClientWithFetch.prototype.free;

/**
 * Subgraph client backed by the browser fetch transport.
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
     * @returns {Promise<any>}
     */
    getLastDaysVolume(days) {
        const ret = wasm.subgraphclient_getLastDaysVolume(this.__wbg_ptr, days);
        return ret;
    }
    /**
     * Fetches hourly volume rows.
     * @param {number} hours
     * @returns {Promise<any>}
     */
    getLastHoursVolume(hours) {
        const ret = wasm.subgraphclient_getLastHoursVolume(this.__wbg_ptr, hours);
        return ret;
    }
    /**
     * Fetches aggregate totals.
     * @returns {Promise<any>}
     */
    getTotals() {
        const ret = wasm.subgraphclient_getTotals(this.__wbg_ptr);
        return ret;
    }
    /**
     * Creates a subgraph client for a chain and Graph API key.
     * @param {number} chain_id
     * @param {string} api_key
     */
    constructor(chain_id, api_key) {
        const ptr0 = passStringToWasm0(api_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.subgraphclient_new(chain_id, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        SubgraphClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Runs a raw GraphQL query.
     * @param {SubgraphQueryInput} request
     * @returns {Promise<any>}
     */
    runQuery(request) {
        const ret = wasm.subgraphclient_runQuery(this.__wbg_ptr, request);
        return ret;
    }
}
if (Symbol.dispose) SubgraphClient.prototype[Symbol.dispose] = SubgraphClient.prototype.free;

/**
 * Subgraph client backed by a JavaScript fetch callback.
 */
export class SubgraphClientWithFetch {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SubgraphClientWithFetch.prototype);
        obj.__wbg_ptr = ptr;
        SubgraphClientWithFetchFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SubgraphClientWithFetchFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_subgraphclientwithfetch_free(ptr, 0);
    }
    /**
     * Creates a subgraph client from an existing fetch-callback handle id.
     * @param {number} chain_id
     * @param {string} api_key
     * @param {number} fetch_callback_id
     * @returns {SubgraphClientWithFetch}
     */
    static fromHandle(chain_id, api_key, fetch_callback_id) {
        const ptr0 = passStringToWasm0(api_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.subgraphclientwithfetch_fromHandle(chain_id, ptr0, len0, fetch_callback_id);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return SubgraphClientWithFetch.__wrap(ret[0]);
    }
    /**
     * Fetches daily volume rows.
     * @param {number} days
     * @returns {Promise<any>}
     */
    getLastDaysVolume(days) {
        const ret = wasm.subgraphclientwithfetch_getLastDaysVolume(this.__wbg_ptr, days);
        return ret;
    }
    /**
     * Fetches hourly volume rows.
     * @param {number} hours
     * @returns {Promise<any>}
     */
    getLastHoursVolume(hours) {
        const ret = wasm.subgraphclientwithfetch_getLastHoursVolume(this.__wbg_ptr, hours);
        return ret;
    }
    /**
     * Fetches aggregate totals.
     * @returns {Promise<any>}
     */
    getTotals() {
        const ret = wasm.subgraphclientwithfetch_getTotals(this.__wbg_ptr);
        return ret;
    }
    /**
     * Creates a subgraph client that owns a registered fetch callback.
     * @param {number} chain_id
     * @param {string} api_key
     * @param {Function} fetch_callback
     */
    constructor(chain_id, api_key, fetch_callback) {
        const ptr0 = passStringToWasm0(api_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.subgraphclientwithfetch_new(chain_id, ptr0, len0, fetch_callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        SubgraphClientWithFetchFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Runs a raw GraphQL query.
     * @param {SubgraphQueryInput} request
     * @returns {Promise<any>}
     */
    runQuery(request) {
        const ret = wasm.subgraphclientwithfetch_runQuery(this.__wbg_ptr, request);
        return ret;
    }
}
if (Symbol.dispose) SubgraphClientWithFetch.prototype[Symbol.dispose] = SubgraphClientWithFetch.prototype.free;

/**
 * Trading facade backed by the browser fetch transport.
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
     * @returns {Promise<any>}
     */
    getQuote(params) {
        const ret = wasm.tradingclient_getQuote(this.__wbg_ptr, params);
        return ret;
    }
    /**
     * Creates a trading client for a chain, environment, and app code.
     * @param {number} chain_id
     * @param {string | null | undefined} env
     * @param {string} app_code
     */
    constructor(chain_id, env, app_code) {
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(app_code, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclient_new(chain_id, ptr0, len0, ptr1, len1);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        TradingClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Quotes, signs, and posts a swap order through a typed-data callback.
     * @param {SwapParametersInput} params
     * @param {string} owner
     * @param {Function} signer_callback
     * @returns {Promise<any>}
     */
    postSwapOrder(params, owner, signer_callback) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclient_postSwapOrder(this.__wbg_ptr, params, ptr0, len0, signer_callback);
        return ret;
    }
    /**
     * Quotes and posts a swap order with a custom EIP-1271 signature callback.
     * @param {SwapParametersInput} params
     * @param {string} owner
     * @param {Function} custom_callback
     * @returns {Promise<any>}
     */
    postSwapOrderWithEip1271(params, owner, custom_callback) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclient_postSwapOrderWithEip1271(this.__wbg_ptr, params, ptr0, len0, custom_callback);
        return ret;
    }
}
if (Symbol.dispose) TradingClient.prototype[Symbol.dispose] = TradingClient.prototype.free;

/**
 * Trading facade backed by a JavaScript fetch callback.
 */
export class TradingClientWithFetch {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TradingClientWithFetch.prototype);
        obj.__wbg_ptr = ptr;
        TradingClientWithFetchFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TradingClientWithFetchFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_tradingclientwithfetch_free(ptr, 0);
    }
    /**
     * Creates a trading client from an existing fetch-callback handle id.
     * @param {number} chain_id
     * @param {string | null | undefined} env
     * @param {string} app_code
     * @param {number} fetch_callback_id
     * @returns {TradingClientWithFetch}
     */
    static fromHandle(chain_id, env, app_code, fetch_callback_id) {
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(app_code, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclientwithfetch_fromHandle(chain_id, ptr0, len0, ptr1, len1, fetch_callback_id);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return TradingClientWithFetch.__wrap(ret[0]);
    }
    /**
     * Fetches a quote without submitting an order.
     * @param {SwapParametersInput} params
     * @returns {Promise<any>}
     */
    getQuote(params) {
        const ret = wasm.tradingclientwithfetch_getQuote(this.__wbg_ptr, params);
        return ret;
    }
    /**
     * Creates a trading client that owns a registered fetch callback.
     * @param {number} chain_id
     * @param {string | null | undefined} env
     * @param {string} app_code
     * @param {Function} fetch_callback
     */
    constructor(chain_id, env, app_code, fetch_callback) {
        var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(app_code, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclientwithfetch_new(chain_id, ptr0, len0, ptr1, len1, fetch_callback);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        TradingClientWithFetchFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Quotes, signs, and posts a swap order through a typed-data callback.
     * @param {SwapParametersInput} params
     * @param {string} owner
     * @param {Function} signer_callback
     * @returns {Promise<any>}
     */
    postSwapOrder(params, owner, signer_callback) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclientwithfetch_postSwapOrder(this.__wbg_ptr, params, ptr0, len0, signer_callback);
        return ret;
    }
    /**
     * Quotes and posts a swap order with a custom EIP-1271 signature callback.
     * @param {SwapParametersInput} params
     * @param {string} owner
     * @param {Function} custom_callback
     * @returns {Promise<any>}
     */
    postSwapOrderWithEip1271(params, owner, custom_callback) {
        const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.tradingclientwithfetch_postSwapOrderWithEip1271(this.__wbg_ptr, params, ptr0, len0, custom_callback);
        return ret;
    }
}
if (Symbol.dispose) TradingClientWithFetch.prototype[Symbol.dispose] = TradingClientWithFetch.prototype.free;

/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init() {
    wasm.__cow_sdk_wasm_init();
}

/**
 * Builds an app-data document without hashing it.
 * @param {AppDataDocInput} doc
 * @returns {any}
 */
export function appDataDoc(doc) {
    const ret = wasm.appDataDoc(doc);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Converts an app-data hash to an IPFS CID.
 * @param {string} app_data_hex
 * @returns {string}
 */
export function appDataHexToCid(app_data_hex) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(app_data_hex, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.appDataHexToCid(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Returns deterministic app-data content, hash, and CID.
 * @param {AppDataDocInput} doc
 * @returns {any}
 */
export function appDataInfo(doc) {
    const ret = wasm.appDataInfo(doc);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Converts an IPFS CID to an app-data hash.
 * @param {string} cid
 * @returns {string}
 */
export function cidToAppDataHex(cid) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.cidToAppDataHex(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Computes the compact order UID and digest.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @param {string} owner
 * @returns {any}
 */
export function computeOrderUid(input, chain_id, owner) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.computeOrderUid(input, chain_id, ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Returns canonical deployment addresses for a chain and environment.
 * @param {number} chain_id
 * @param {string | null} [env]
 * @returns {any}
 */
export function deploymentAddresses(chain_id, env) {
    var ptr0 = isLikeNone(env) ? 0 : passStringToWasm0(env, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    const ret = wasm.deploymentAddresses(chain_id, ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Computes the EIP-712 domain separator for a supported chain.
 * @param {number} chain_id
 * @returns {string}
 */
export function domainSeparator(chain_id) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.domainSeparator(chain_id);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA signature.
 * @param {OrderInput} input
 * @param {string} ecdsa_signature
 * @returns {string}
 */
export function eip1271SignaturePayload(input, ecdsa_signature) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(ecdsa_signature, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.eip1271SignaturePayload(input, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * Fetches and parses an app-data document by CID.
 * @param {string} cid
 * @param {string | null} [ipfs_uri]
 * @param {number | null} [timeout_ms]
 * @returns {Promise<any>}
 */
export function fetchAppDataFromCid(cid, ipfs_uri, timeout_ms) {
    const ptr0 = passStringToWasm0(cid, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    const ret = wasm.fetchAppDataFromCid(ptr0, len0, ptr1, len1, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0);
    return ret;
}

/**
 * Fetches and parses an app-data document by app-data hash.
 * @param {string} app_data_hex
 * @param {string | null} [ipfs_uri]
 * @param {number | null} [timeout_ms]
 * @returns {Promise<any>}
 */
export function fetchAppDataFromHex(app_data_hex, ipfs_uri, timeout_ms) {
    const ptr0 = passStringToWasm0(app_data_hex, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(ipfs_uri) ? 0 : passStringToWasm0(ipfs_uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    const ret = wasm.fetchAppDataFromHex(ptr0, len0, ptr1, len1, isLikeNone(timeout_ms) ? 0x100000001 : (timeout_ms) >>> 0);
    return ret;
}

/**
 * Builds signer-facing order typed data.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @returns {any}
 */
export function orderTypedData(input, chain_id) {
    const ret = wasm.orderTypedData(input, chain_id);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Registers a JS fetch callback and returns a disposable handle.
 * @param {Function} callback
 * @returns {FetchCallbackHandle}
 */
export function registerFetchCallback(callback) {
    const ret = wasm.registerFetchCallback(callback);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return FetchCallbackHandle.__wrap(ret[0]);
}

/**
 * Signs a cancellation digest through an explicit `eth_sign` callback.
 * @param {string[]} order_uids
 * @param {number} chain_id
 * @param {Function} digest_signer
 * @returns {Promise<any>}
 */
export function signCancellationEthSignDigest(order_uids, chain_id, digest_signer) {
    const ptr0 = passArrayJsValueToWasm0(order_uids, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signCancellationEthSignDigest(ptr0, len0, chain_id, digest_signer);
    return ret;
}

/**
 * Signs cancellation typed data through an EIP-1193 callback.
 * @param {string[]} order_uids
 * @param {number} chain_id
 * @param {string} owner
 * @param {Function} request_callback
 * @returns {Promise<any>}
 */
export function signCancellationWithEip1193(order_uids, chain_id, owner, request_callback) {
    const ptr0 = passArrayJsValueToWasm0(order_uids, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.signCancellationWithEip1193(ptr0, len0, chain_id, ptr1, len1, request_callback);
    return ret;
}

/**
 * Signs cancellation typed data through a typed-data callback.
 * @param {string[]} order_uids
 * @param {number} chain_id
 * @param {Function} typed_data_signer
 * @returns {Promise<any>}
 */
export function signCancellationWithTypedDataSigner(order_uids, chain_id, typed_data_signer) {
    const ptr0 = passArrayJsValueToWasm0(order_uids, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signCancellationWithTypedDataSigner(ptr0, len0, chain_id, typed_data_signer);
    return ret;
}

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @param {string} owner
 * @param {Function} digest_signer
 * @returns {Promise<any>}
 */
export function signOrderEthSignDigest(input, chain_id, owner, digest_signer) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderEthSignDigest(input, chain_id, ptr0, len0, digest_signer);
    return ret;
}

/**
 * Signs an order through a custom EIP-1271 callback.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @param {string} owner
 * @param {Function} custom_callback
 * @returns {Promise<any>}
 */
export function signOrderWithCustomEip1271(input, chain_id, owner, custom_callback) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithCustomEip1271(input, chain_id, ptr0, len0, custom_callback);
    return ret;
}

/**
 * Signs an order through an EIP-1193 request callback.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @param {string} owner
 * @param {Function} request_callback
 * @returns {Promise<any>}
 */
export function signOrderWithEip1193(input, chain_id, owner, request_callback) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithEip1193(input, chain_id, ptr0, len0, request_callback);
    return ret;
}

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @param {string} owner
 * @param {Function} typed_data_signer
 * @returns {Promise<any>}
 */
export function signOrderWithEip1271(input, chain_id, owner, typed_data_signer) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithEip1271(input, chain_id, ptr0, len0, typed_data_signer);
    return ret;
}

/**
 * Signs an order through a typed-data callback.
 * @param {OrderInput} input
 * @param {number} chain_id
 * @param {string} owner
 * @param {Function} typed_data_signer
 * @returns {Promise<any>}
 */
export function signOrderWithTypedDataSigner(input, chain_id, owner, typed_data_signer) {
    const ptr0 = passStringToWasm0(owner, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.signOrderWithTypedDataSigner(input, chain_id, ptr0, len0, typed_data_signer);
    return ret;
}

/**
 * Returns supported EVM chain ids.
 * @returns {Uint32Array}
 */
export function supportedChainIds() {
    const ret = wasm.supportedChainIds();
    var v1 = getArrayU32FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
    return v1;
}

/**
 * Validates an app-data document against the embedded schemas.
 * @param {AppDataDocInput} doc
 * @returns {any}
 */
export function validateAppDataDoc(doc) {
    const ret = wasm.validateAppDataDoc(doc);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Returns the wasm crate version.
 * @returns {string}
 */
export function wasmVersion() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.wasmVersion();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_960c155d3d49e4c2: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_Number_32bf70a599af1d4b: function(arg0) {
            const ret = Number(arg0);
            return ret;
        },
        __wbg_String_8564e559799eccda: function(arg0, arg1) {
            const ret = String(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_bigint_get_as_i64_3d3aba5d616c6a51: function(arg0, arg1) {
            const v = arg1;
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_boolean_get_6ea149f0a8dcc5ff: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_ab4b34d23d6778bd: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_a5d8b22e52b24dd1: function(arg0, arg1) {
            const ret = arg0 in arg1;
            return ret;
        },
        __wbg___wbindgen_is_bigint_ec25c7f91b4d9e93: function(arg0) {
            const ret = typeof(arg0) === 'bigint';
            return ret;
        },
        __wbg___wbindgen_is_function_3baa9db1a987f47d: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_63322ec0cd6ea4ef: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_6df3bf7ef1164ed3: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_29a43b4d42920abd: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_eq_d3465d8a07697228: function(arg0, arg1) {
            const ret = arg0 === arg1;
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_cac3565e89b4134c: function(arg0, arg1) {
            const ret = arg0 == arg1;
            return ret;
        },
        __wbg___wbindgen_number_get_c7f42aed0525c451: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_7ed5322991caaec5: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_6b64449b9b9ed33c: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_b46c9b5a9f08ec37: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_abort_4ce5b484434ef6fd: function(arg0) {
            arg0.abort();
        },
        __wbg_abort_79db88f743c3efd7: function(arg0) {
            arg0.abort();
        },
        __wbg_call_14b169f759b26747: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.call(arg1);
            return ret;
        }, arguments); },
        __wbg_call_a24592a6f349a97e: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_clearTimeout_1a62f3563b1611b3: function(arg0, arg1) {
            arg0.clearTimeout(arg1);
        },
        __wbg_clearTimeout_3629d6209dfcc46e: function(arg0) {
            const ret = clearTimeout(arg0);
            return ret;
        },
        __wbg_clearTimeout_a5b2d1f832c8c5b6: function(arg0) {
            globalThis.clearTimeout(arg0);
        },
        __wbg_done_9158f7cc8751ba32: function(arg0) {
            const ret = arg0.done;
            return ret;
        },
        __wbg_entries_e0b73aa8571ddb56: function(arg0) {
            const ret = Object.entries(arg0);
            return ret;
        },
        __wbg_error_a6fa202b58aa1cd3: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_fetch_9ea633a8592ee39a: function(arg0, arg1) {
            const ret = arg0.fetch(arg1);
            return ret;
        },
        __wbg_from_0dbf29f09e7fb200: function(arg0) {
            const ret = Array.from(arg0);
            return ret;
        },
        __wbg_getRandomValues_3f44b700395062e5: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_get_1affdbdd5573b16a: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_6011fa3a58f61074: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_8360291721e2339f: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_unchecked_17f53dad852b9588: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
            const ret = arg0[arg1];
            return ret;
        },
        __wbg_headers_6022deb4e576fb8e: function(arg0) {
            const ret = arg0.headers;
            return ret;
        },
        __wbg_instanceof_ArrayBuffer_7c8433c6ed14ffe3: function(arg0) {
            let result;
            try {
                result = arg0 instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Map_1b76fd4635be43eb: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Map;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Object_7c99480a1cdfb911: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Object;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Response_9b2d111407865ff2: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Response;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_152ba1f289edcf3f: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_cc64c86c8ef9e02b: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_c3109d14ffc06469: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_isSafeInteger_4fc213d1989d6d2a: function(arg0) {
            const ret = Number.isSafeInteger(arg0);
            return ret;
        },
        __wbg_iterator_013bc09ec998c2a7: function() {
            const ret = Symbol.iterator;
            return ret;
        },
        __wbg_length_3d4ecd04bd8d22f1: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_9f1775224cf1d815: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_0c7403db6e782f19: function(arg0) {
            const ret = new Uint8Array(arg0);
            return ret;
        },
        __wbg_new_15a4889b4b90734d: function() { return handleError(function () {
            const ret = new Headers();
            return ret;
        }, arguments); },
        __wbg_new_227d7c05414eb861: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_new_34d45cc8e36aaead: function() {
            const ret = new Map();
            return ret;
        },
        __wbg_new_682678e2f47e32bc: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_98c22165a42231aa: function() { return handleError(function () {
            const ret = new AbortController();
            return ret;
        }, arguments); },
        __wbg_new_aa8d0fa9762c29bd: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_d3704878df906b51: function() {
            const ret = new globalThis.AbortController();
            return ret;
        },
        __wbg_new_typed_323f37fd55ab048d: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h16b7440c88f0269d(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = 0;
            }
        },
        __wbg_new_with_str_and_init_897be1708e42f39d: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
            return ret;
        }, arguments); },
        __wbg_next_0340c4ae324393c3: function() { return handleError(function (arg0) {
            const ret = arg0.next();
            return ret;
        }, arguments); },
        __wbg_next_7646edaa39458ef7: function(arg0) {
            const ret = arg0.next;
            return ret;
        },
        __wbg_now_a9b7df1cbee90986: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_now_e7c6795a7f81e10f: function(arg0) {
            const ret = arg0.now();
            return ret;
        },
        __wbg_performance_3fcf6e32a7e1ed0a: function(arg0) {
            const ret = arg0.performance;
            return ret;
        },
        __wbg_prototypesetcall_a6b02eb00b0f4ce2: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_queueMicrotask_5d15a957e6aa920e: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_queueMicrotask_f8819e5ffc402f36: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_resolve_e6c466bc1052f16c: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_setTimeout_3b32677b3fda46e8: function(arg0, arg1) {
            const ret = globalThis.setTimeout(arg0, arg1 >>> 0);
            return ret;
        },
        __wbg_setTimeout_56bcdccbad22fd44: function() { return handleError(function (arg0, arg1) {
            const ret = setTimeout(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_setTimeout_d8786dd31f90da0f: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_022bee52d0b05b19: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(arg0, arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_1ffc463d4c541483: function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
            arg0.set(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        }, arguments); },
        __wbg_set_3bf1de9fab0cd644: function(arg0, arg1, arg2) {
            arg0[arg1 >>> 0] = arg2;
        },
        __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
            arg0[arg1] = arg2;
        },
        __wbg_set_body_be11680f34217f75: function(arg0, arg1) {
            arg0.body = arg1;
        },
        __wbg_set_fde2cec06c23692b: function(arg0, arg1, arg2) {
            const ret = arg0.set(arg1, arg2);
            return ret;
        },
        __wbg_set_headers_50fc01786240a440: function(arg0, arg1) {
            arg0.headers = arg1;
        },
        __wbg_set_method_c9f1f985f6b6c427: function(arg0, arg1, arg2) {
            arg0.method = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_signal_1d4e73c2305a0e7c: function(arg0, arg1) {
            arg0.signal = arg1;
        },
        __wbg_signal_b74e34a36211c513: function(arg0) {
            const ret = arg0.signal;
            return ret;
        },
        __wbg_signal_fdc54643b47bf85b: function(arg0) {
            const ret = arg0.signal;
            return ret;
        },
        __wbg_stack_3b0d974bbf31e44f: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_8cfadc87a297ca02: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_602256ae5c8f42cf: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_e445c1c7484aecc3: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_f20e8576ef1e0f17: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_status_43e0d2f15b22d69f: function(arg0) {
            const ret = arg0.status;
            return ret;
        },
        __wbg_text_595ef75535aa25c1: function() { return handleError(function (arg0) {
            const ret = arg0.text();
            return ret;
        }, arguments); },
        __wbg_then_792e0c862b060889: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_then_8e16ee11f05e4827: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_value_ee3a06f4579184fa: function(arg0) {
            const ret = arg0.value;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [Externref], shim_idx: 584, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hd9f032c1a2f8138e);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 384, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__hf3184c2633042a72);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { owned: true, function: Function { arguments: [], shim_idx: 546, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm_bindgen__convert__closures_____invoke__h7af8680472e534d6);
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000005: function(arg0) {
            // Cast intrinsic for `I64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000006: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000007: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./cow_sdk_wasm_bg.js": import0,
    };
}

function wasm_bindgen__convert__closures_____invoke__hf3184c2633042a72(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__hf3184c2633042a72(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__h7af8680472e534d6(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h7af8680472e534d6(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__hd9f032c1a2f8138e(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__hd9f032c1a2f8138e(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__h16b7440c88f0269d(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h16b7440c88f0269d(arg0, arg1, arg2, arg3);
}

const FetchCallbackHandleFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fetchcallbackhandle_free(ptr >>> 0, 1));
const HttpToIpfsAdapterFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_httptoipfsadapter_free(ptr >>> 0, 1));
const IpfsClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ipfsclient_free(ptr >>> 0, 1));
const IpfsClientWithFetchFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_ipfsclientwithfetch_free(ptr >>> 0, 1));
const OrderBookClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_orderbookclient_free(ptr >>> 0, 1));
const OrderBookClientWithFetchFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_orderbookclientwithfetch_free(ptr >>> 0, 1));
const SubgraphClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_subgraphclient_free(ptr >>> 0, 1));
const SubgraphClientWithFetchFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_subgraphclientwithfetch_free(ptr >>> 0, 1));
const TradingClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_tradingclient_free(ptr >>> 0, 1));
const TradingClientWithFetchFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_tradingclientwithfetch_free(ptr >>> 0, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => wasm.__wbindgen_destroy_closure(state.a, state.b));

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

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

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
            wasm.__wbindgen_destroy_closure(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
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

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('cow_sdk_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
