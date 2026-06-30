use std::sync::Arc;

use cow_sdk_core::{
    Address, Amount, BlockInfo, ContractCall, CowEnv, HexData, HttpTransport, Provider, Signer,
    SupportedChainId, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TypedDataPayload,
};
use cow_sdk_orderbook::OrderbookApi;
use serde_json::{Map, Value};
// The native trade-parameter types are built directly from a `serde_json::Value`
// in the camelCase wire shape, so the component lane shares the SDK's own serde
// parsing (hex address, decimal amount, token-balance validation, and partner-fee
// variant selection) rather than a separate boundary input type.
use cow_sdk_trading::{QuoteResults, TradeParams, Trading};
// The limit-order parameter type is used by the stateful lanes' `run_limit`.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
use cow_sdk_trading::LimitTradeParams;

// The boundary error surface and its constructors are shared with the reads:
// parse/setup failures classify as `validation`; trading-lane failures carry
// the SDK's class and retry hint through `from_trading`.
use super::orderbook::{ReadError, from_trading, invalid};

/// A host signing call: sign a 32-byte digest, return a 65-byte signature.
/// Each world supplies its own generated `signer::sign-digest` import here,
/// so this module stays world-agnostic.
pub type SignFn = fn(&[u8]) -> Result<Vec<u8>, String>;

/// One partner-fee policy, lane-agnostic: the world-specific generated
/// `partner-fee-policy` record is lowered to this in each lane's mapping so the
/// shared `core` builders never name a per-world type. Exactly one basis-point
/// shape is populated, matching the native `PartnerFeePolicy` variant selection.
pub struct PartnerFeePolicyParams<'a> {
    pub volume_bps: Option<u16>,
    pub surplus_bps: Option<u16>,
    pub price_improvement_bps: Option<u16>,
    pub max_volume_bps: Option<u16>,
    pub recipient: &'a str,
}

/// Optional trade fields shared by `swap-request` and `limit-request`, lowered
/// from each world's generated record into lane-agnostic borrows. Threading them
/// through one struct keeps `run_swap`/`run_limit` and the shared input builder
/// in sync, so a field cannot reach one entry and be dropped on another.
#[derive(Default)]
pub struct CommonTradeParams<'a> {
    pub receiver: Option<&'a str>,
    pub valid_to: Option<u32>,
    pub valid_for: Option<u32>,
    pub partially_fillable: Option<bool>,
    pub sell_token_balance: Option<&'a str>,
    pub buy_token_balance: Option<&'a str>,
    pub settlement_contract_override: Option<&'a [(u64, String)]>,
    pub eth_flow_contract_override: Option<&'a [(u64, String)]>,
    pub partner_fee: Option<Vec<PartnerFeePolicyParams<'a>>>,
}

/// Swap parameters, borrowed from the lane's typed `swap-request` record.
pub struct SwapParams<'a> {
    pub chain_id: u64,
    pub owner: &'a str,
    pub sell_token: &'a str,
    pub buy_token: &'a str,
    pub amount: &'a str,
    pub app_code: &'a str,
    pub kind: Option<&'a str>,
    pub slippage_bps: Option<u32>,
    pub env: Option<&'a str>,
    pub common: CommonTradeParams<'a>,
}

fn parse_env(env: Option<&str>) -> Result<CowEnv, String> {
    match env.unwrap_or("prod") {
        "prod" => Ok(CowEnv::Prod),
        "staging" => Ok(CowEnv::Staging),
        other => Err(format!("unknown environment: {other}")),
    }
}

/// Builds the JSON object for one chain-keyed contract override in the shape the
/// native [`cow_sdk_core::AddressPerChain`] (`BTreeMap<ChainId, Address>`)
/// deserializes from: a map whose keys are the chain ids as strings. The
/// addresses are validated by the native `from_value` when the trade parameters
/// are built.
fn override_json(entries: &[(u64, String)]) -> Value {
    let mut map = Map::new();
    for (chain_id, address) in entries {
        map.insert(chain_id.to_string(), Value::String(address.clone()));
    }
    Value::Object(map)
}

/// Builds the JSON for a lane-agnostic partner-fee list in the shape the native
/// untagged [`cow_sdk_app_data::PartnerFee`] deserializes from, reproducing the
/// old boundary fold: a one-element list collapses to a single policy object
/// (`PartnerFee::Single`), while zero or many policies stay a JSON array
/// (`PartnerFee::Multiple`, an empty array being `Multiple([])`). Each policy
/// object carries only the basis-point fields that are set plus the recipient,
/// so the native untagged `PartnerFeePolicy` selects Volume / Surplus /
/// `PriceImprovement` by field presence; the bounds rule is the native type's.
fn partner_fee_json(policies: &[PartnerFeePolicyParams<'_>]) -> Value {
    fn policy_json(policy: &PartnerFeePolicyParams<'_>) -> Value {
        let mut object = Map::new();
        if let Some(volume_bps) = policy.volume_bps {
            object.insert("volumeBps".to_owned(), Value::from(volume_bps));
        }
        if let Some(surplus_bps) = policy.surplus_bps {
            object.insert("surplusBps".to_owned(), Value::from(surplus_bps));
        }
        if let Some(price_improvement_bps) = policy.price_improvement_bps {
            object.insert(
                "priceImprovementBps".to_owned(),
                Value::from(price_improvement_bps),
            );
        }
        if let Some(max_volume_bps) = policy.max_volume_bps {
            object.insert("maxVolumeBps".to_owned(), Value::from(max_volume_bps));
        }
        object.insert(
            "recipient".to_owned(),
            Value::String(policy.recipient.to_owned()),
        );
        Value::Object(object)
    }

    match policies {
        [single] => policy_json(single),
        many => Value::Array(many.iter().map(policy_json).collect()),
    }
}

/// Threads the shared optional trade fields onto a trade-parameter JSON object.
/// Both the swap and the limit shape carry the identical optional-field set, so
/// one routine populates either object: an absent option leaves the key out so
/// the native `from_value` applies its own default, and every present field is
/// inserted as the raw wire value the native deserializer validates (the buy
/// balance rejecting `external`, the addresses and amounts parsing through
/// serde). Building JSON cannot fail, so this returns nothing — the native
/// `from_value` is where an invalid value surfaces.
fn apply_common(object: &mut Map<String, Value>, common: &CommonTradeParams<'_>) {
    if let Some(receiver) = common.receiver {
        object.insert("receiver".to_owned(), Value::String(receiver.to_owned()));
    }
    if let Some(valid_to) = common.valid_to {
        object.insert("validTo".to_owned(), Value::from(valid_to));
    }
    if let Some(valid_for) = common.valid_for {
        object.insert("validFor".to_owned(), Value::from(valid_for));
    }
    if let Some(partially_fillable) = common.partially_fillable {
        object.insert(
            "partiallyFillable".to_owned(),
            Value::Bool(partially_fillable),
        );
    }
    if let Some(balance) = common.sell_token_balance {
        object.insert(
            "sellTokenBalance".to_owned(),
            Value::String(balance.to_owned()),
        );
    }
    if let Some(balance) = common.buy_token_balance {
        object.insert(
            "buyTokenBalance".to_owned(),
            Value::String(balance.to_owned()),
        );
    }
    if let Some(entries) = common.settlement_contract_override {
        object.insert(
            "settlementContractOverride".to_owned(),
            override_json(entries),
        );
    }
    if let Some(entries) = common.eth_flow_contract_override {
        object.insert("ethFlowContractOverride".to_owned(), override_json(entries));
    }
    if let Some(policies) = common.partner_fee.as_deref() {
        object.insert("partnerFee".to_owned(), partner_fee_json(policies));
    }
}

/// The EIP-712 signing hash of a typed-data payload, via the canonical Alloy
/// typed-data shape — byte-identical to the SDK's own order digest.
fn typed_data_digest(payload: &TypedDataPayload) -> Result<[u8; 32], String> {
    let message: serde_json::Value = serde_json::from_str(payload.message_json())
        .map_err(|error| format!("typed-data message: {error}"))?;
    let typed: alloy_dyn_abi::eip712::TypedData = serde_json::from_value(serde_json::json!({
        "domain": payload.domain,
        "types": payload.types,
        "primaryType": payload.primary_type,
        "message": message,
    }))
    .map_err(|error| format!("typed-data shape: {error}"))?;
    Ok(typed
        .eip712_signing_hash()
        .map_err(|error| error.to_string())?
        .0)
}

/// A keys-out signer: the component computes the EIP-712 digest and the host
/// signs it through the `signer` import. The private key never enters here.
struct HostSigner {
    owner: Address,
    chain: SupportedChainId,
    sign: SignFn,
}

impl HostSigner {
    fn host_sign(&self, digest: &[u8; 32]) -> Result<String, String> {
        let signature = (self.sign)(&digest[..])?;
        if signature.len() != 65 {
            return Err(format!(
                "host signer returned {} bytes, expected 65",
                signature.len()
            ));
        }
        Ok(alloy_primitives::hex::encode_prefixed(signature))
    }
}

impl Signer for HostSigner {
    type Error = String;

    fn chain_id(&self) -> Option<SupportedChainId> {
        Some(self.chain)
    }

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(self.owner)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        let digest = alloy_primitives::eip191_hash_message(message).0;
        self.host_sign(&digest)
    }

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        let digest = typed_data_digest(payload)?;
        self.host_sign(&digest)
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Err("send_transaction is unavailable in the component signer".to_owned())
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Err("estimate_gas is unavailable in the component signer".to_owned())
    }
}

/// A pre-resolved EIP-1271 contract signature, returned verbatim as the order
/// signature (ADR 0073). The Component `authorization.eip1271` variant carries
/// the already-resolved opaque blob — the host signs at the boundary, exactly as
/// the wasm-bindgen lane resolves its callback before crossing into native — so
/// this adapter holds only the bytes and never consults a key.
struct ResolvedEip1271Provider {
    signature: String,
}

#[cfg_attr(target_arch = "wasm32", cow_sdk_signing::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), cow_sdk_signing::async_trait)]
impl cow_sdk_signing::eip1271::Eip1271Signer for ResolvedEip1271Provider {
    async fn sign(
        &self,
        _order_to_sign: &cow_sdk_core::OrderData,
    ) -> Result<String, cow_sdk_signing::eip1271::Eip1271SignatureError> {
        Ok(self.signature.clone())
    }
}

/// EIP-1271 provider that signs the order digest through the host `sign-digest`
/// import and wraps the result as the `CoW` contract-signature payload (ADR
/// 0073). It backs the `authorization.eip1271` arm when the carried blob is
/// empty: a Safe whose owner is the host key produces the verifier payload on
/// demand instead of supplying a pre-resolved blob. Keys-out — the private key
/// stays in the host.
struct HostEip1271Provider {
    chain: SupportedChainId,
    sign: SignFn,
}

#[cfg_attr(target_arch = "wasm32", cow_sdk_signing::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), cow_sdk_signing::async_trait)]
impl cow_sdk_signing::eip1271::Eip1271Signer for HostEip1271Provider {
    async fn sign(
        &self,
        order_to_sign: &cow_sdk_core::OrderData,
    ) -> Result<String, cow_sdk_signing::eip1271::Eip1271SignatureError> {
        let fail = |message: String| {
            cow_sdk_signing::eip1271::Eip1271SignatureError::provider("host-sign-digest", message)
        };
        let generated =
            cow_sdk_signing::generate_order_id(self.chain, order_to_sign, &Address::ZERO, None)
                .map_err(|error| fail(error.to_string()))?;
        let ecdsa = (self.sign)(generated.order_digest.as_slice()).map_err(fail)?;
        if ecdsa.len() != 65 {
            return Err(fail(format!(
                "host signer returned {} bytes, expected 65",
                ecdsa.len()
            )));
        }
        let ecdsa_hex = alloy_primitives::hex::encode_prefixed(&ecdsa);
        cow_sdk_signing::eip1271_signature_payload(order_to_sign, &ecdsa_hex)
            .map_err(|error| fail(error.to_string()))
    }
}

/// The lane-agnostic order authorization mode, lowered from each world's
/// generated `authorization` variant (ADR 0073). It mirrors the data-only WIT
/// shape: `Ecdsa` signs through the host signer, `Eip1271` carries the
/// (possibly empty) pre-resolved contract-signature blob, and `PreSign` consults
/// no signer.
pub enum AuthParams {
    /// EOA / EIP-712 signing through the host signer.
    Ecdsa,
    /// Safe off-chain contract signature; the bytes are the pre-resolved blob,
    /// empty meaning the host signer produces it on demand.
    Eip1271(Vec<u8>),
    /// Safe on-chain pre-sign; no signing.
    PreSign,
}

/// The lane-agnostic placement result, lowered into each world's generated
/// `order-placement` variant (ADR 0073). `Live` carries the order UID; `Pending`
/// additionally carries the on-chain activation calls as `(to, data, value)`
/// wire parts.
pub enum Placement {
    /// The order is live at post (`Ecdsa` / `Eip1271`).
    Live { order_uid: String },
    /// The order is posted but not yet authorized on-chain (`PreSign`).
    Pending {
        order_uid: String,
        calls: Vec<(String, String, String)>,
    },
}

/// A host contract-read call: `(address, method, abi-json, args-json)` in, the
/// host's ABI-decoded result as JSON out. World-agnostic — each world wraps its
/// generated `contract-read` import as this fn pointer.
pub type ReadFn = fn(&str, &str, &str, &str) -> Result<String, String>;

const READ_ONLY: &str = "only contract reads are available through the contract-read import";

/// A read-only [`Provider`] backed by the host `contract-read` import
/// (node-out): the component encodes the call, the host runs the `eth_call`.
/// Only `read_contract` is wired; the rest of the read surface is unused here.
struct ContractReadProvider {
    read: ReadFn,
}

impl Provider for ContractReadProvider {
    type Error = String;

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        (self.read)(
            &request.address.to_hex_string(),
            &request.method,
            &request.abi_json,
            &request.args_json,
        )
    }

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Err(READ_ONLY.to_owned())
    }
    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Err(READ_ONLY.to_owned())
    }
    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Err(READ_ONLY.to_owned())
    }
    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Err(READ_ONLY.to_owned())
    }
    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Err(READ_ONLY.to_owned())
    }
}

/// Reads the trader's `CoW` Protocol allowance — the ERC-20 allowance `token`
/// has granted the chain's vault relayer — through the host `contract-read`
/// import, returned as a JSON decimal-string value. No signer; keys-out.
pub async fn run_allowance(
    read: ReadFn,
    chain_id: u64,
    owner: &str,
    token: &str,
    env: Option<&str>,
) -> Result<String, ReadError> {
    let chain = SupportedChainId::try_from(chain_id).map_err(|error| invalid(error.to_string()))?;
    let env = parse_env(env).map_err(invalid)?;
    let owner = parse_addr(owner)?;
    let token = parse_addr(token)?;
    let provider = ContractReadProvider { read };
    let allowance =
        cow_sdk_trading::cow_protocol_allowance(&provider, &token, &owner, chain, env, None)
            .await
            .map_err(|error| from_trading(&error))?;
    serde_json::to_string(&allowance).map_err(|error| invalid(error.to_string()))
}

/// Parses a `0x` address argument into the typed `Address`, as a boundary
/// validation error.
fn parse_addr(value: &str) -> Result<Address, ReadError> {
    Address::new(value).map_err(|error| invalid(error.to_string()))
}

/// Builds the native [`TradeParams`] from the lane-agnostic swap borrows.
///
/// The fields are assembled into a `serde_json::Value` in the SDK's camelCase
/// wire shape and parsed through the native deserializer, so the address, the
/// amount, the token-balance validation, and the partner-fee variant selection
/// are all the SDK's own — a parse failure maps to the boundary validation
/// error. The swap defaults the wasm-bindgen lane applies are preserved: an
/// absent `kind` is `sell` and an absent `slippage-bps` is 50. The environment
/// is not set on the value — like the wasm lane it is resolved at orderbook
/// construction — so the built `TradeParams` leaves `env` unset.
fn swap_trade_params(params: &SwapParams<'_>) -> Result<TradeParams, ReadError> {
    let mut object = Map::new();
    object.insert(
        "kind".to_owned(),
        Value::String(params.kind.unwrap_or("sell").to_owned()),
    );
    object.insert("owner".to_owned(), Value::String(params.owner.to_owned()));
    object.insert(
        "sellToken".to_owned(),
        Value::String(params.sell_token.to_owned()),
    );
    object.insert(
        "buyToken".to_owned(),
        Value::String(params.buy_token.to_owned()),
    );
    object.insert("amount".to_owned(), Value::String(params.amount.to_owned()));
    object.insert(
        "slippageBps".to_owned(),
        Value::from(params.slippage_bps.unwrap_or(50)),
    );
    apply_common(&mut object, &params.common);
    serde_json::from_value(Value::Object(object)).map_err(|error| invalid(error.to_string()))
}

/// The owner the built parameters carry; the swap value always sets it, so this
/// only fails if the owner address was malformed.
fn input_owner(owner: Option<&Address>) -> Result<Address, ReadError> {
    owner
        .copied()
        .ok_or_else(|| invalid("owner address is required".to_owned()))
}

/// Builds the orderbook + trading clients over the lane's transport. The
/// component target has no default transport, so each call injects its own.
fn build_trading<T>(
    transport: T,
    chain: SupportedChainId,
    env: CowEnv,
    app_code: &str,
) -> Result<Trading, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    let orderbook = OrderbookApi::builder()
        .chain(chain)
        .env(env)
        .transport(Arc::new(transport))
        .build()
        .map_err(|error| invalid(error.to_string()))?;
    Trading::builder()
        .chain_id(chain)
        .app_code(app_code)
        .orderbook_shared(Arc::new(orderbook))
        .build()
        .map_err(|error| invalid(error.to_string()))
}

/// Quotes, signs through the host signer, and posts a swap order. Returns a
/// JSON result string carrying the accepted order UID.
pub async fn run_swap<T>(
    transport: T,
    sign: SignFn,
    params: SwapParams<'_>,
) -> Result<String, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    let chain =
        SupportedChainId::try_from(params.chain_id).map_err(|error| invalid(error.to_string()))?;
    let env = parse_env(params.env).map_err(invalid)?;
    let app_code = params.app_code;
    let trade = swap_trade_params(&params)?;
    let owner = input_owner(trade.owner.as_ref())?;

    let trading = build_trading(transport, chain, env, app_code)?;

    let quote = trading
        .quote_only(trade, None)
        .await
        .map_err(|error| from_trading(&error))?;
    let buy_amount = quote.quote_response.quote.buy_amount.to_string();

    let signer = HostSigner { owner, chain, sign };
    let posted = trading
        .post_swap_order_from_quote(&quote, &signer, None)
        .await
        .map_err(|error| from_trading(&error))?;

    Ok(serde_json::json!({
        "uid": posted.order_id.to_hex_string(),
        "buyAmount": buy_amount,
    })
    .to_string())
}

/// Limit-order parameters, borrowed from a lane's `limit-request` record.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
pub struct LimitParams<'a> {
    pub chain_id: u64,
    pub owner: &'a str,
    pub sell_token: &'a str,
    pub buy_token: &'a str,
    pub sell_amount: &'a str,
    pub buy_amount: &'a str,
    pub app_code: &'a str,
    pub kind: Option<&'a str>,
    pub env: Option<&'a str>,
    pub quote_id: Option<i64>,
    pub slippage_bps: Option<u32>,
    pub common: CommonTradeParams<'a>,
}

/// Builds the native [`LimitTradeParams`] from the lane-agnostic limit borrows.
///
/// As with the swap value the fields are assembled into a `serde_json::Value` in
/// the camelCase wire shape and parsed through the native deserializer, so the
/// addresses, the amounts, and the partner-fee selection are the SDK's own; a
/// parse failure maps to the boundary validation error. The environment is
/// resolved at orderbook construction, so the built `LimitTradeParams` leaves
/// `env` unset; an absent `kind` is `sell`.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
fn limit_trade_params(params: &LimitParams<'_>) -> Result<LimitTradeParams, ReadError> {
    let mut object = Map::new();
    object.insert(
        "kind".to_owned(),
        Value::String(params.kind.unwrap_or("sell").to_owned()),
    );
    object.insert("owner".to_owned(), Value::String(params.owner.to_owned()));
    object.insert(
        "sellToken".to_owned(),
        Value::String(params.sell_token.to_owned()),
    );
    object.insert(
        "buyToken".to_owned(),
        Value::String(params.buy_token.to_owned()),
    );
    object.insert(
        "sellAmount".to_owned(),
        Value::String(params.sell_amount.to_owned()),
    );
    object.insert(
        "buyAmount".to_owned(),
        Value::String(params.buy_amount.to_owned()),
    );
    if let Some(quote_id) = params.quote_id {
        object.insert("quoteId".to_owned(), Value::from(quote_id));
    }
    if let Some(slippage_bps) = params.slippage_bps {
        object.insert("slippageBps".to_owned(), Value::from(slippage_bps));
    }
    apply_common(&mut object, &params.common);
    serde_json::from_value(Value::Object(object)).map_err(|error| invalid(error.to_string()))
}

/// Signs through the host signer and posts a limit order at the supplied
/// price (no quote). Returns a JSON result string carrying the order UID.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
pub async fn run_limit<T>(
    transport: T,
    sign: SignFn,
    params: LimitParams<'_>,
) -> Result<String, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    let chain =
        SupportedChainId::try_from(params.chain_id).map_err(|error| invalid(error.to_string()))?;
    let env = parse_env(params.env).map_err(invalid)?;
    let app_code = params.app_code;
    let limit = limit_trade_params(&params)?;
    let owner = input_owner(limit.owner.as_ref())?;

    let trading = build_trading(transport, chain, env, app_code)?;
    let signer = HostSigner { owner, chain, sign };
    let posted = trading
        .post_limit_order(limit, &signer, None)
        .await
        .map_err(|error| from_trading(&error))?;
    Ok(serde_json::json!({ "uid": posted.order_id.to_hex_string() }).to_string())
}

/// Builds the `Arc<dyn Eip1271Signer>` the `Eip1271` authorization arm threads
/// into the native placement: the pre-resolved blob returned verbatim, or — when
/// the blob is empty — the host signer producing the contract-signature payload
/// on demand (ADR 0073).
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
fn eip1271_provider(
    blob: &[u8],
    chain: SupportedChainId,
    sign: SignFn,
) -> Arc<dyn cow_sdk_signing::eip1271::Eip1271Signer> {
    if blob.is_empty() {
        Arc::new(HostEip1271Provider { chain, sign })
    } else {
        Arc::new(ResolvedEip1271Provider {
            signature: alloy_primitives::hex::encode_prefixed(blob),
        })
    }
}

/// Lowers the native [`OrderPlacement`] sum into the lane-agnostic [`Placement`]
/// (ADR 0073), stringifying the activation calls of a pending pre-sign order to
/// their `(to, data, value)` wire parts.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
fn to_placement(placement: cow_sdk_trading::OrderPlacement) -> Placement {
    use cow_sdk_trading::OrderPlacement;
    match placement {
        OrderPlacement::Live { order_uid } => Placement::Live {
            order_uid: order_uid.to_hex_string(),
        },
        OrderPlacement::PendingActivation {
            order_uid,
            activation,
        } => Placement::Pending {
            order_uid: order_uid.to_hex_string(),
            calls: activation
                .calls
                .iter()
                .map(|tx| {
                    (
                        tx.to.to_hex_string(),
                        tx.data.to_hex_string(),
                        tx.value.to_string(),
                    )
                })
                .collect(),
        },
        // `OrderPlacement` is `#[non_exhaustive]`; a future arm fails closed
        // rather than emitting a placement the consumer cannot match.
        _ => Placement::Live {
            order_uid: String::new(),
        },
    }
}

/// Quotes, authorizes a swap through `auth`, and posts it for `owner`,
/// returning the typed placement sum (ADR 0073). `Ecdsa` signs through the host
/// signer and `Eip1271` through the resolved-or-host provider, both resolving to
/// `Live`; `PreSign` posts with no signer and resolves to `Pending` carrying the
/// on-chain activation calls.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
pub async fn run_place_swap<T>(
    transport: T,
    sign: SignFn,
    params: SwapParams<'_>,
    owner: &str,
    auth: AuthParams,
) -> Result<Placement, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    use cow_sdk_trading::{Authorization, NoSigner};

    let chain =
        SupportedChainId::try_from(params.chain_id).map_err(|error| invalid(error.to_string()))?;
    let env = parse_env(params.env).map_err(invalid)?;
    let app_code = params.app_code;
    let owner = parse_addr(owner)?;
    let trade = swap_trade_params(&params)?;

    let trading = build_trading(transport, chain, env, app_code)?;
    let quote = trading
        .quote_only(trade, None)
        .await
        .map_err(|error| from_trading(&error))?;

    let placement = match auth {
        AuthParams::Ecdsa => {
            let signer = HostSigner { owner, chain, sign };
            trading
                .place_swap(&quote, owner, Authorization::ecdsa(&signer), None)
                .await
        }
        AuthParams::Eip1271(blob) => {
            let provider = eip1271_provider(&blob, chain, sign);
            trading
                .place_swap(
                    &quote,
                    owner,
                    Authorization::<NoSigner>::eip1271(provider),
                    None,
                )
                .await
        }
        AuthParams::PreSign => {
            trading
                .place_swap(&quote, owner, Authorization::<NoSigner>::pre_sign(), None)
                .await
        }
    }
    .map_err(|error| from_trading(&error))?;
    Ok(to_placement(placement))
}

/// Authorizes a limit order through `auth` and posts it for `owner`, returning
/// the typed placement sum (ADR 0073). The authorization arms map exactly as for
/// `run_place_swap`.
#[cfg(any(feature = "world-client-sync", feature = "world-client-async"))]
pub async fn run_place_limit<T>(
    transport: T,
    sign: SignFn,
    params: LimitParams<'_>,
    owner: &str,
    auth: AuthParams,
) -> Result<Placement, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    use cow_sdk_trading::{Authorization, NoSigner};

    let chain =
        SupportedChainId::try_from(params.chain_id).map_err(|error| invalid(error.to_string()))?;
    let env = parse_env(params.env).map_err(invalid)?;
    let app_code = params.app_code;
    let owner = parse_addr(owner)?;
    let limit = limit_trade_params(&params)?;

    let trading = build_trading(transport, chain, env, app_code)?;

    let placement = match auth {
        AuthParams::Ecdsa => {
            let signer = HostSigner { owner, chain, sign };
            trading
                .place_limit(limit, owner, Authorization::ecdsa(&signer), None)
                .await
        }
        AuthParams::Eip1271(blob) => {
            let provider = eip1271_provider(&blob, chain, sign);
            trading
                .place_limit(
                    limit,
                    owner,
                    Authorization::<NoSigner>::eip1271(provider),
                    None,
                )
                .await
        }
        AuthParams::PreSign => {
            trading
                .place_limit(limit, owner, Authorization::<NoSigner>::pre_sign(), None)
                .await
        }
    }
    .map_err(|error| from_trading(&error))?;
    Ok(to_placement(placement))
}

/// Fetches a rich trading quote — amounts and costs, suggested slippage, and
/// the order to sign — for a market swap, returned as the SDK's canonical
/// camelCase JSON. No signer, no posting: a consumer inspects the quote, or
/// feeds it back to `run_post_swap_from_quote` to commit to the quoted price.
pub async fn run_quote<T>(transport: T, params: SwapParams<'_>) -> Result<String, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    let chain =
        SupportedChainId::try_from(params.chain_id).map_err(|error| invalid(error.to_string()))?;
    let env = parse_env(params.env).map_err(invalid)?;
    let app_code = params.app_code;
    let trade = swap_trade_params(&params)?;

    let trading = build_trading(transport, chain, env, app_code)?;

    let quote = trading
        .quote_only(trade, None)
        .await
        .map_err(|error| from_trading(&error))?;
    serde_json::to_string(&quote).map_err(|error| invalid(error.to_string()))
}

/// The app-code used only to build the trading client when posting from a
/// quote. The posted order commits to the quote's own sealed app-data
/// (`post_swap_order_from_quote` reads it from the quote, not the client), so
/// this value never reaches the order.
const POST_FROM_QUOTE_APP_CODE: &str = "cow-sdk-component";

/// Signs (via the host signer) and posts a swap from a quote previously
/// returned by `run_quote`. The chain, environment, and owner are taken from
/// the quote, so the posted order commits to exactly the quoted price; the
/// quote's runtime binding (chain + environment) is re-validated before
/// signing.
pub async fn run_post_swap_from_quote<T>(
    transport: T,
    sign: SignFn,
    quote_json: &str,
) -> Result<String, ReadError>
where
    T: HttpTransport + Send + Sync + 'static,
{
    let quote: QuoteResults =
        serde_json::from_str(quote_json).map_err(|error| invalid(error.to_string()))?;
    let binding = quote.orderbook_binding.as_ref().ok_or_else(|| {
        invalid("quote is missing its orderbook binding; re-fetch the quote".to_owned())
    })?;
    let chain = binding.chain_id;
    let env = binding.env;
    let owner = quote
        .trade_parameters
        .owner
        .ok_or_else(|| invalid("quote is missing its owner".to_owned()))?;

    let trading = build_trading(transport, chain, env, POST_FROM_QUOTE_APP_CODE)?;
    let signer = HostSigner { owner, chain, sign };
    let posted = trading
        .post_swap_order_from_quote(&quote, &signer, None)
        .await
        .map_err(|error| from_trading(&error))?;
    Ok(serde_json::json!({ "uid": posted.order_id.to_hex_string() }).to_string())
}
