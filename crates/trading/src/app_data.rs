use serde_json::{Map, Value, json};

use cow_sdk_app_data::{AppDataParams, PartnerFee, app_data_info, generate_app_data_doc};
use cow_sdk_core::AppCode;
use cow_sdk_orderbook::OrderClass;

use crate::{TradingAppDataInfo, TradingError};

/// `metadata.utm.utmSource` default stamped when the caller does not supply
/// an override `metadata.utm` block.
///
/// The source groups traffic under the wider `CoW SDK` family. The Rust crate
/// and version stay visible through `utmMedium`.
const UTM_SOURCE: &str = "cow-sdk";

/// `metadata.utm.utmCampaign` default stamped when the caller does not supply
/// an override `metadata.utm` block.
const UTM_CAMPAIGN: &str = "developer-cohort";

/// `metadata.utm.utmTerm` default that identifies Rust-SDK traffic in
/// attribution analytics. Intentionally distinct from other SDK identifiers
/// so Rust-SDK adoption is not mislabelled.
const UTM_TERM: &str = "rs";

/// Builds the default `metadata.utm` block stamped on app-data documents
/// when the caller does not supply their own `metadata.utm`.
///
/// The block identifies the `CoW SDK` family, Rust SDK, and compile-time version
/// so protocol-side attribution analytics can group SDK traffic while still
/// distinguishing this crate from other client SDKs. The `utmMedium` value
/// embeds the trading crate's published version through
/// `env!("CARGO_PKG_VERSION")`.
fn default_utm() -> Value {
    json!({
        "utmSource": UTM_SOURCE,
        "utmMedium": format!("cow-rs@{}", env!("CARGO_PKG_VERSION")),
        "utmCampaign": UTM_CAMPAIGN,
        "utmContent": "",
        "utmTerm": UTM_TERM,
    })
}

/// Builds the trading app-data document and its derived hash.
///
/// The generated base document always includes quote slippage metadata and
/// order class metadata; `order_class` is stamped in its lowercase wire form
/// through the [`OrderClass`] serde representation.
/// When the caller does not supply `metadata.utm`, an SDK-family default
/// UTM attribution block is stamped onto the base document so downstream
/// analytics can group `CoW SDK` traffic while preserving Rust-specific
/// attribution. Any caller-supplied `metadata.utm` — partial or full —
/// disables the default stamp and is carried through exactly as provided.
/// `advanced_params` then overrides `appCode`, `environment`, and metadata keys using a deep merge.
///
/// # Errors
///
/// Returns an error when the merged app-data document cannot be normalized into a valid app-data
/// payload or hash.
pub async fn build_app_data(
    app_code: &AppCode,
    slippage_bps: u32,
    order_class: OrderClass,
    partner_fee: Option<&PartnerFee>,
    advanced_params: Option<&AppDataParams>,
) -> Result<TradingAppDataInfo, TradingError> {
    let mut metadata = Map::new();
    metadata.insert("quote".to_owned(), json!({ "slippageBips": slippage_bps }));
    metadata.insert(
        "orderClass".to_owned(),
        json!({ "orderClass": order_class }),
    );
    if let Some(partner_fee) = partner_fee {
        partner_fee.validate()?;
        metadata.insert("partnerFee".to_owned(), partner_fee.to_value());
    }

    let override_has_utm = advanced_params
        .and_then(|params| params.metadata.get("utm"))
        .is_some();
    if !override_has_utm {
        metadata.insert("utm".to_owned(), default_utm());
    }

    let mut params = AppDataParams::new(app_code.clone()).with_metadata(metadata);
    if let Some(advanced_params) = advanced_params {
        params = merge_app_data_params(&params, advanced_params);
    }

    let doc = generate_app_data_doc(params);
    let info = app_data_info(doc.clone())?.info;

    Ok(TradingAppDataInfo {
        doc,
        full_app_data: info.app_data_content,
        app_data_keccak256: cow_sdk_core::AppDataHash::new(info.app_data_hex)?,
    })
}

/// Parses an already-sealed app-data wire document back into typed
/// [`AppDataParams`].
///
/// The existing [`AppDataParams`] deserializer lifts `metadata.signer`,
/// `metadata.flashloan`, and `metadata.hooks` out of the wire shape into
/// their typed fields so the returned value is ready to drive the typed
/// merge pipeline without any additional coercion.
///
/// # Errors
///
/// Returns [`TradingError::AppData`] when the supplied document does not
/// conform to the [`AppDataParams`] wire shape — for example when
/// `metadata.signer` carries a value that is not a valid address, when
/// `metadata.flashloan` carries an object that fails the typed flash-loan
/// hints validation, or when `metadata.hooks` carries malformed hook
/// metadata.
pub fn params_from_doc(base_doc: &Value) -> Result<AppDataParams, TradingError> {
    serde_json::from_value::<AppDataParams>(base_doc.clone())
        .map_err(|error| TradingError::AppData(cow_sdk_app_data::AppDataError::from(error)))
}

/// Merges a typed [`AppDataParams`] override onto a previously-sealed
/// app-data wire document and re-emits the canonical wire form.
///
/// The base document is deserialized through the existing
/// [`AppDataParams`] deserializer so the typed `signer` and `flashloan`
/// fields on the base side participate in the merge on equal footing with
/// the override, and the resulting typed value drives
/// [`generate_app_data_doc`] and [`app_data_info`] to re-derive the
/// wire document and its digest from one authoritative typed shape.
///
/// The returned tuple carries both the [`TradingAppDataInfo`] (the
/// wire document, stringified content, and keccak256 hash) and the
/// typed merged [`AppDataParams`], so submission seams can read the
/// final `signer` field directly from the same merged value that
/// produced the wire document rather than re-reading the override.
///
/// The merge applies the reviewed hooks-replacement rule so override-supplied
/// typed hooks or `metadata.hooks` replace the base-side hooks envelope in
/// full instead of recursively merging pre/post sibling arrays.
///
/// # Errors
///
/// Returns [`TradingError::AppData`] when the base document cannot be
/// parsed into typed [`AppDataParams`], or when the merged document
/// cannot be normalized into a valid app-data payload or hash.
pub fn merge_and_seal_app_data(
    base_doc: &Value,
    override_params: &AppDataParams,
) -> Result<(TradingAppDataInfo, AppDataParams), TradingError> {
    let base_params = params_from_doc(base_doc)?;
    let merged_params = merge_app_data_params(&base_params, override_params);
    let doc = generate_app_data_doc(merged_params.clone());
    let info = app_data_info(doc.clone())?.info;

    Ok((
        TradingAppDataInfo {
            doc,
            full_app_data: info.app_data_content,
            app_data_keccak256: cow_sdk_core::AppDataHash::new(info.app_data_hex)?,
        },
        merged_params,
    ))
}

/// Merges a typed [`AppDataParams`] override onto a typed base
/// [`AppDataParams`] and returns the typed merged value.
///
/// Scalar and optional top-level fields (`app_code`, `environment`,
/// `signer`, `flashloan`, `hooks`) follow override-wins semantics with a
/// base-value fallback. The nested `metadata` map is recursively deep
/// merged, with one carve-out: when the override contains typed hooks or a
/// `hooks` metadata entry the base side's `hooks` envelope is dropped before
/// the merge so override-supplied hooks fully replace the base-side hooks
/// envelope instead of recursively merging into it. This keeps the metadata
/// merge shape aligned with the reviewed upstream SDK, where a caller
/// supplying a new hooks object means "use these hooks and nothing else"
/// rather than "merge these hooks on top of whatever pre/post arrays the base
/// doc happens to have".
///
/// Non-`hooks` metadata entries continue to follow standard recursive
/// deep-merge semantics. Arrays fall through to the override value in
/// full — including the `userConsents` array — so replacement rather
/// than concatenation is the default for any JSON array on the
/// metadata side.
#[must_use]
pub(crate) fn merge_app_data_params(
    base: &AppDataParams,
    override_params: &AppDataParams,
) -> AppDataParams {
    let mut base_metadata = base.metadata.clone();
    let override_has_metadata_hooks = override_params.metadata.contains_key("hooks");
    // The reviewed upstream SDK replaces rather than recursively merges
    // `metadata.hooks` — when the override supplies any hooks envelope,
    // pre/post sibling arrays from the base side are dropped before the
    // deep merge so the override's hooks envelope is the final shape.
    if override_params.hooks.is_some() || override_has_metadata_hooks {
        base_metadata.remove("hooks");
    }

    let metadata = match deep_merge_values(
        Value::Object(base_metadata),
        Value::Object(override_params.metadata.clone()),
    ) {
        Value::Object(map) => map,
        _ => Map::new(),
    };

    let mut params = AppDataParams::default().with_metadata(metadata);
    params.app_code = override_params
        .app_code
        .clone()
        .or_else(|| base.app_code.clone());
    params.environment = override_params
        .environment
        .clone()
        .or_else(|| base.environment.clone());
    params.signer = override_params.signer.or(base.signer);
    params.flashloan = override_params
        .flashloan
        .clone()
        .or_else(|| base.flashloan.clone());
    params.hooks = if override_params.hooks.is_some() {
        override_params.hooks.clone()
    } else if override_has_metadata_hooks {
        None
    } else {
        base.hooks.clone()
    };
    params
}

fn deep_merge_values(base: Value, override_value: Value) -> Value {
    match (base, override_value) {
        (Value::Object(mut base), Value::Object(override_map)) => {
            for (key, value) in override_map {
                let merged = deep_merge_values(base.remove(&key).unwrap_or(Value::Null), value);
                base.insert(key, merged);
            }
            Value::Object(base)
        }
        (_, value) => value,
    }
}
