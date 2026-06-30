use cow_sdk_contracts::{ContractId, UnsignedTransaction};
use cow_sdk_core::{Address, Amount, CowEnv, OrderData, OrderUid, ProtocolOptions};

use super::parse_chain;

/// The `(to, data, value)` wire parts of a gas-free unsigned transaction.
fn parts(tx: &UnsignedTransaction) -> (String, String, String) {
    (
        tx.to.to_hex_string(),
        tx.data.to_hex_string(),
        tx.value.to_string(),
    )
}

fn parse_env(env: Option<&str>) -> Result<CowEnv, String> {
    match env.unwrap_or("prod") {
        "prod" | "production" => Ok(CowEnv::Prod),
        "staging" | "barn" => Ok(CowEnv::Staging),
        other => Err(format!("unknown environment: {other}")),
    }
}

fn options(env: Option<&str>) -> Result<ProtocolOptions, String> {
    Ok(ProtocolOptions::new().with_env(parse_env(env)?))
}

/// Builds an ERC-20 approval for the vault relayer (or an explicit `spender`).
pub fn approve(
    chain_id: u64,
    token: &str,
    amount: &str,
    spender: Option<&str>,
    env: Option<&str>,
) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let token = Address::new(token).map_err(|error| error.to_string())?;
    let amount = Amount::new(amount).map_err(|error| error.to_string())?;
    let spender = match spender {
        Some(spender) => Address::new(spender).map_err(|error| error.to_string())?,
        None => cow_sdk_contracts::resolve_contract_address(
            ContractId::VaultRelayer,
            None,
            chain,
            parse_env(env)?,
        )
        .ok_or_else(|| "vault relayer is not deployed for this chain".to_owned())?,
    };
    Ok(parts(&cow_sdk_contracts::approve_transaction(
        token, spender, amount,
    )))
}

/// Builds a settlement pre-signature transaction.
pub fn pre_sign(
    chain_id: u64,
    order_uid: &str,
    env: Option<&str>,
) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let uid = OrderUid::new(order_uid).map_err(|error| error.to_string())?;
    let tx = cow_sdk_contracts::pre_sign_transaction(&uid, chain, Some(&options(env)?))
        .map_err(|error| error.to_string())?;
    Ok(parts(&tx))
}

/// Builds the on-chain activation bundle for an already-posted pre-sign order:
/// the ordered `[approve, setPreSignature]` calls a smart-contract wallet runs
/// to authorize it. Wraps the native `build_presign_activation`, returning each
/// call as the `(to, data, value)` wire parts.
pub fn presign_activation(
    chain_id: u64,
    order_uid: &str,
    sell_token: &str,
    amount: &str,
    env: Option<&str>,
) -> Result<Vec<(String, String, String)>, String> {
    let chain = parse_chain(chain_id)?;
    let uid = OrderUid::new(order_uid).map_err(|error| error.to_string())?;
    let sell_token = Address::new(sell_token).map_err(|error| error.to_string())?;
    let amount = Amount::new(amount).map_err(|error| error.to_string())?;
    let activation = cow_sdk_trading::build_presign_activation(
        &uid,
        sell_token,
        amount,
        chain,
        Some(&options(env)?),
    )
    .map_err(|error| error.to_string())?;
    Ok(activation.calls.iter().map(parts).collect())
}

/// Builds a settlement on-chain cancellation transaction.
pub fn cancel(
    chain_id: u64,
    order_uid: &str,
    env: Option<&str>,
) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let uid = OrderUid::new(order_uid).map_err(|error| error.to_string())?;
    let tx = cow_sdk_contracts::invalidate_order_transaction(&uid, chain, Some(&options(env)?))
        .map_err(|error| error.to_string())?;
    Ok(parts(&tx))
}

/// Builds a native-asset wrap transaction.
pub fn wrap(chain_id: u64, amount: &str) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let amount = Amount::new(amount).map_err(|error| error.to_string())?;
    Ok(parts(&cow_sdk_contracts::wrap_transaction(chain, amount)))
}

/// Builds a wrapped-native unwrap transaction.
pub fn unwrap(chain_id: u64, amount: &str) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let amount = Amount::new(amount).map_err(|error| error.to_string())?;
    Ok(parts(&cow_sdk_contracts::unwrap_transaction(chain, amount)))
}

/// Builds an eth-flow native-sell create-order transaction from a camelCase
/// order JSON.
pub fn sell_native(
    chain_id: u64,
    order_json: &str,
    quote_id: i64,
    env: Option<&str>,
) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    let tx = cow_sdk_contracts::ethflow_create_order_transaction(
        &order,
        quote_id,
        chain,
        Some(&options(env)?),
    )
    .map_err(|error| error.to_string())?;
    Ok(parts(&tx))
}

/// Builds an eth-flow on-chain cancellation transaction from a camelCase
/// order JSON.
pub fn cancel_native(
    chain_id: u64,
    order_json: &str,
    quote_id: i64,
    env: Option<&str>,
) -> Result<(String, String, String), String> {
    let chain = parse_chain(chain_id)?;
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    let tx = cow_sdk_contracts::ethflow_invalidate_order_transaction(
        &order,
        quote_id,
        chain,
        Some(&options(env)?),
    )
    .map_err(|error| error.to_string())?;
    Ok(parts(&tx))
}
