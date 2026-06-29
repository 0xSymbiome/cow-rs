wit_bindgen::generate!({ world: "order-engine" });

use exports::cow::protocol::order::{OrderData, OrderKind};

struct Component;

/// Maps the typed order record to the canonical camelCase order JSON the
/// signing crate deserializes, filling the protocol defaults for omitted
/// optional fields.
fn order_data_json(order: &OrderData) -> String {
    serde_json::json!({
        "sellToken": order.sell_token,
        "buyToken": order.buy_token,
        "receiver": order
            .receiver
            .clone()
            .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_owned()),
        "sellAmount": order.sell_amount,
        "buyAmount": order.buy_amount,
        "feeAmount": order.fee_amount.clone().unwrap_or_else(|| "0".to_owned()),
        "validTo": order.valid_to,
        "appData": order.app_data,
        "kind": match order.kind {
            OrderKind::Buy => "buy",
            OrderKind::Sell => "sell",
        },
        "partiallyFillable": order.partially_fillable.unwrap_or(false),
        "sellTokenBalance": order
            .sell_token_balance
            .clone()
            .unwrap_or_else(|| "erc20".to_owned()),
        "buyTokenBalance": order
            .buy_token_balance
            .clone()
            .unwrap_or_else(|| "erc20".to_owned()),
    })
    .to_string()
}

impl exports::cow::protocol::order::Guest for Component {
    fn uid(chain_id: u64, owner: String, order: OrderData) -> Result<String, String> {
        super::compute_uid(chain_id, &owner, &order_data_json(&order))
    }

    fn digest(chain_id: u64, order: OrderData) -> Result<String, String> {
        super::compute_digest(chain_id, &order_data_json(&order))
    }
}

impl exports::cow::protocol::chains::Guest for Component {
    fn supported_chain_ids() -> Vec<u64> {
        cow_sdk_core::SupportedChainId::ALL
            .iter()
            .map(|chain| u64::from(*chain))
            .collect()
    }

    fn domain_separator(chain_id: u64) -> Result<String, String> {
        let chain = super::parse_chain(chain_id)?;
        Ok(cow_sdk_signing::domain_separator(chain, None))
    }

    fn wrapped_native_token(
        chain_id: u64,
    ) -> Result<exports::cow::protocol::chains::WrappedNative, String> {
        let chain = super::parse_chain(chain_id)?;
        let info = cow_sdk_core::wrapped_native_token(chain);
        Ok(exports::cow::protocol::chains::WrappedNative {
            address: info.address.to_hex_string(),
            symbol: info.symbol,
            decimals: info.decimals,
        })
    }

    fn deployments(
        chain_id: u64,
        env: Option<String>,
    ) -> Result<exports::cow::protocol::chains::DeploymentAddresses, String> {
        let chain = super::parse_chain(chain_id)?;
        let env = match env.as_deref().unwrap_or("prod") {
            "prod" | "production" => cow_sdk_core::CowEnv::Prod,
            "staging" | "barn" => cow_sdk_core::CowEnv::Staging,
            other => return Err(format!("unknown environment: {other}")),
        };
        let registry = cow_sdk_contracts::Registry::default();
        let address = |contract| {
            registry
                .address(contract, chain, env)
                .map(|addr| addr.to_hex_string())
                .ok_or_else(|| "deployment is not configured".to_owned())
        };
        Ok(exports::cow::protocol::chains::DeploymentAddresses {
            settlement: address(cow_sdk_contracts::ContractId::Settlement)?,
            vault_relayer: address(cow_sdk_contracts::ContractId::VaultRelayer)?,
            eth_flow: address(cow_sdk_contracts::ContractId::EthFlow)?,
        })
    }
}

impl exports::cow::protocol::app_data::Guest for Component {
    fn hex_to_cid(app_data_hex: String) -> Result<String, String> {
        cow_sdk_app_data::app_data_hex_to_cid(&app_data_hex).map_err(|error| error.to_string())
    }

    fn cid_to_hex(cid: String) -> Result<String, String> {
        cow_sdk_app_data::cid_to_app_data_hex(&cid).map_err(|error| error.to_string())
    }

    fn info(doc_json: String) -> Result<exports::cow::protocol::app_data::AppDataInfo, String> {
        let validated = cow_sdk_app_data::app_data_info(doc_json.as_str())
            .map_err(|error| error.to_string())?;
        Ok(exports::cow::protocol::app_data::AppDataInfo {
            cid: validated.info.cid,
            app_data_content: validated.info.app_data_content,
            app_data_hex: validated.info.app_data_hex,
        })
    }

    fn validate(doc_json: String) -> Result<(), String> {
        let document: serde_json::Value =
            serde_json::from_str(&doc_json).map_err(|error| error.to_string())?;
        cow_sdk_app_data::validate_app_data_doc(&document).map_err(|error| error.to_string())
    }
}

impl exports::cow::protocol::tx::Guest for Component {
    fn approve(
        chain_id: u64,
        token: String,
        amount: String,
        spender: Option<String>,
        env: Option<String>,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::approve(
            chain_id,
            &token,
            &amount,
            spender.as_deref(),
            env.as_deref(),
        )
        .map(tx_request)
    }

    fn pre_sign(
        chain_id: u64,
        order_uid: String,
        env: Option<String>,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::pre_sign(chain_id, &order_uid, env.as_deref()).map(tx_request)
    }

    fn cancel(
        chain_id: u64,
        order_uid: String,
        env: Option<String>,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::cancel(chain_id, &order_uid, env.as_deref()).map(tx_request)
    }

    fn wrap(
        chain_id: u64,
        amount: String,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::wrap(chain_id, &amount).map(tx_request)
    }

    fn unwrap(
        chain_id: u64,
        amount: String,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::unwrap(chain_id, &amount).map(tx_request)
    }

    fn sell_native(
        chain_id: u64,
        order: OrderData,
        quote_id: i64,
        env: Option<String>,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::sell_native(chain_id, &order_data_json(&order), quote_id, env.as_deref())
            .map(tx_request)
    }

    fn cancel_native(
        chain_id: u64,
        order: OrderData,
        quote_id: i64,
        env: Option<String>,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::tx::cancel_native(chain_id, &order_data_json(&order), quote_id, env.as_deref())
            .map(tx_request)
    }
}

fn tx_request(
    (to, data, value): (String, String, String),
) -> exports::cow::protocol::tx::TxRequest {
    exports::cow::protocol::tx::TxRequest { to, data, value }
}

impl exports::cow::protocol::composable::Guest for Component {
    fn twap_create_transaction(
        twap: exports::cow::protocol::composable::TwapData,
        salt: String,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        let data = twap_from_wit(&twap)?;
        super::composable::create_transaction(&data, &salt).map(tx_request)
    }

    fn twap_remove_transaction(
        order_id: String,
    ) -> Result<exports::cow::protocol::tx::TxRequest, String> {
        super::composable::remove_transaction(&order_id).map(tx_request)
    }

    fn twap_order_id(
        twap: exports::cow::protocol::composable::TwapData,
        salt: String,
    ) -> Result<String, String> {
        let data = twap_from_wit(&twap)?;
        super::composable::order_id(&data, &salt)
    }

    fn twap_timing_at(
        twap: exports::cow::protocol::composable::TwapData,
        start: u64,
        now: u64,
    ) -> Result<exports::cow::protocol::composable::TwapTiming, String> {
        let data = twap_from_wit(&twap)?;
        map_twap_timing(super::composable::timing_at(&data, start, now)?)
    }
}

/// Lowers the WIT `twap-data` record (with its policy variants) to the validated
/// contract [`cow_sdk_contracts::composable::TwapData`].
fn twap_from_wit(
    twap: &exports::cow::protocol::composable::TwapData,
) -> Result<cow_sdk_contracts::composable::TwapData, String> {
    use cow_sdk_contracts::composable::{TwapDurationOfPart, TwapStartTime};
    use exports::cow::protocol::composable::{TwapDuration, TwapStart};

    let start = match &twap.start {
        TwapStart::AtMiningTime => TwapStartTime::AtMiningTime,
        TwapStart::AtEpoch(epoch) => TwapStartTime::AtEpoch(*epoch),
    };
    let duration = match &twap.duration {
        TwapDuration::Auto => TwapDurationOfPart::Auto,
        TwapDuration::LimitDuration(span) => TwapDurationOfPart::LimitDuration(*span),
    };
    super::composable::build_twap(
        &twap.sell_token,
        &twap.buy_token,
        twap.receiver.as_deref(),
        &twap.sell_amount,
        &twap.buy_amount,
        twap.number_of_parts,
        twap.time_between_parts,
        start,
        duration,
        &twap.app_data,
    )
}

/// Maps the contract `TwapTiming` classifier to its WIT variant. `TwapTiming` is
/// `#[non_exhaustive]`, so a future schedule state fails closed rather than
/// emitting a shape the consumer cannot match.
fn map_twap_timing(
    timing: cow_sdk_contracts::composable::TwapTiming,
) -> Result<exports::cow::protocol::composable::TwapTiming, String> {
    use cow_sdk_contracts::composable::TwapTiming as Source;
    use exports::cow::protocol::composable as wit;
    Ok(match timing {
        Source::NotStarted { start_epoch } => wit::TwapTiming::NotStarted(start_epoch),
        Source::Active {
            part,
            valid_to,
            next_part_start,
            is_last,
        } => wit::TwapTiming::Active(wit::TwapActive {
            part,
            valid_to,
            next_part_start,
            is_last,
        }),
        Source::Expired => wit::TwapTiming::Expired,
        _ => return Err("decoded twap timing is not representable".to_owned()),
    })
}

impl exports::cow::protocol::trading_math::Guest for Component {
    fn calculate_amounts_and_costs(
        quote_json: String,
        slippage_bps: u32,
        partner_fee_bps: u32,
        protocol_fee_bps: String,
    ) -> Result<exports::cow::protocol::trading_math::QuoteAmountsAndCosts, String> {
        let amounts = super::trading_math::calculate_amounts_and_costs(
            &quote_json,
            slippage_bps,
            partner_fee_bps,
            &protocol_fee_bps,
        )?;
        Ok(map_quote_amounts_and_costs(&amounts))
    }

    fn suggest_slippage_bps(
        quote_json: String,
        partner_fee_bps: u32,
        is_eth_flow: bool,
    ) -> Result<u32, String> {
        super::trading_math::suggest_slippage(&quote_json, partner_fee_bps, is_eth_flow)
    }

    fn build_app_data(
        app_code: String,
        slippage_bps: u32,
        order_class: Option<String>,
    ) -> Result<exports::cow::protocol::trading_math::AppDataInfo, String> {
        let info =
            super::trading_math::build_app_data(&app_code, slippage_bps, order_class.as_deref())?;
        Ok(exports::cow::protocol::trading_math::AppDataInfo {
            cid: info.cid,
            app_data_content: info.app_data_content,
            app_data_hex: info.app_data_hex,
        })
    }
}

/// Maps the native [`cow_sdk_core::QuoteAmountsAndCosts`] stage breakdown to its
/// WIT record, stringifying each typed amount in atoms.
fn map_quote_amounts_and_costs(
    amounts: &cow_sdk_core::QuoteAmountsAndCosts,
) -> exports::cow::protocol::trading_math::QuoteAmountsAndCosts {
    use exports::cow::protocol::trading_math as wit;

    let stage = |pair: &cow_sdk_core::Amounts<cow_sdk_core::Amount>| wit::StageAmounts {
        sell_amount: pair.sell_amount.to_string(),
        buy_amount: pair.buy_amount.to_string(),
    };
    wit::QuoteAmountsAndCosts {
        is_sell: amounts.is_sell,
        costs: wit::QuoteCosts {
            network_fee: wit::NetworkFee {
                amount_in_sell_currency: amounts
                    .costs
                    .network_fee
                    .amount_in_sell_currency
                    .to_string(),
                amount_in_buy_currency: amounts
                    .costs
                    .network_fee
                    .amount_in_buy_currency
                    .to_string(),
            },
            partner_fee: wit::FeeComponent {
                amount: amounts.costs.partner_fee.amount.to_string(),
                bps: amounts.costs.partner_fee.bps,
            },
            protocol_fee: wit::FeeComponent {
                amount: amounts.costs.protocol_fee.amount.to_string(),
                bps: amounts.costs.protocol_fee.bps,
            },
        },
        before_all_fees: stage(&amounts.before_all_fees),
        before_network_costs: stage(&amounts.before_network_costs),
        after_protocol_fees: stage(&amounts.after_protocol_fees),
        after_network_costs: stage(&amounts.after_network_costs),
        after_partner_fees: stage(&amounts.after_partner_fees),
        after_slippage: stage(&amounts.after_slippage),
        amounts_to_sign: stage(&amounts.amounts_to_sign),
    }
}

impl exports::cow::protocol::order_signing::Guest for Component {
    fn order_typed_data(chain_id: u64, order: OrderData) -> Result<String, String> {
        super::signing::order_typed_data(chain_id, &order_data_json(&order))
    }

    fn generate_order_id(
        chain_id: u64,
        owner: String,
        order: OrderData,
    ) -> Result<exports::cow::protocol::order_signing::OrderId, String> {
        let (uid, digest) =
            super::signing::generate_order_id(chain_id, &owner, &order_data_json(&order))?;
        Ok(exports::cow::protocol::order_signing::OrderId {
            order_uid: uid,
            order_digest: digest,
        })
    }

    fn eip1271_signature_payload(
        order: OrderData,
        ecdsa_signature: String,
    ) -> Result<String, String> {
        super::signing::eip1271_signature_payload(&order_data_json(&order), &ecdsa_signature)
    }

    fn cancellations_typed_data(chain_id: u64, order_uids: Vec<String>) -> Result<String, String> {
        super::signing::cancellations_typed_data(chain_id, &order_uids)
    }
}

impl exports::cow::protocol::events::Guest for Component {
    fn decode_settlement_log(
        log: exports::cow::protocol::events::EventLog,
    ) -> Result<exports::cow::protocol::events::SettlementEvent, String> {
        map_settlement_event(super::events::settlement(&log.topics, &log.data)?)
    }

    fn decode_eth_flow_log(
        log: exports::cow::protocol::events::EventLog,
    ) -> Result<exports::cow::protocol::events::EthFlowEvent, String> {
        map_eth_flow_event(super::events::eth_flow(&log.topics, &log.data)?)
    }
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the freshly decoded event is consumed by the mapping and discarded; a reference would only force the caller to bind a temporary"
)]
fn map_settlement_event(
    event: cow_sdk_contracts::SettlementEvent,
) -> Result<exports::cow::protocol::events::SettlementEvent, String> {
    use cow_sdk_contracts::SettlementEvent as Source;
    use exports::cow::protocol::events as wit;
    Ok(match event {
        Source::Trade {
            owner,
            sell_token,
            buy_token,
            sell_amount,
            buy_amount,
            fee_amount,
            order_uid,
        } => wit::SettlementEvent::Trade(wit::SettlementTrade {
            owner: owner.to_hex_string(),
            sell_token: sell_token.to_hex_string(),
            buy_token: buy_token.to_hex_string(),
            sell_amount: sell_amount.to_string(),
            buy_amount: buy_amount.to_string(),
            fee_amount: fee_amount.to_string(),
            order_uid: order_uid.to_hex_string(),
        }),
        Source::Interaction {
            target,
            value,
            selector,
        } => wit::SettlementEvent::Interaction(wit::SettlementInteraction {
            target: target.to_hex_string(),
            value: value.to_string(),
            selector: alloy_primitives::hex::encode_prefixed(selector),
        }),
        Source::Settlement { solver } => wit::SettlementEvent::Settlement(solver.to_hex_string()),
        Source::OrderInvalidated { owner, order_uid } => {
            wit::SettlementEvent::OrderInvalidated(wit::OrderOwnerUid {
                owner: owner.to_hex_string(),
                order_uid: order_uid.to_hex_string(),
            })
        }
        Source::PreSignature {
            owner,
            order_uid,
            signed,
        } => wit::SettlementEvent::PreSignature(wit::PreSignatureEvent {
            owner: owner.to_hex_string(),
            order_uid: order_uid.to_hex_string(),
            signed,
        }),
        _ => return Err("unsupported settlement event".to_owned()),
    })
}

fn map_eth_flow_event(
    event: cow_sdk_contracts::EthFlowEvent,
) -> Result<exports::cow::protocol::events::EthFlowEvent, String> {
    use cow_sdk_contracts::EthFlowEvent as Source;
    use exports::cow::protocol::events as wit;
    Ok(match event {
        Source::OrderPlacement(placement) => {
            let order_json =
                serde_json::to_string(&placement.order).map_err(|error| error.to_string())?;
            // An explicit, fail-closed map mirroring the wasm-bindgen lane
            // (`EthFlowEvent::from_event`): a future on-chain signing scheme
            // errors rather than emitting a `Debug`-derived string the consumer
            // cannot match. `OnchainSigningScheme` is `#[non_exhaustive]`, so the
            // wildcard arm is required and is the fail-closed guard.
            let signing_scheme = match placement.signing_scheme {
                cow_sdk_contracts::OnchainSigningScheme::Eip1271 => "eip1271",
                cow_sdk_contracts::OnchainSigningScheme::PreSign => "presign",
                _ => return Err("decoded on-chain signing scheme is not representable".to_owned()),
            };
            wit::EthFlowEvent::OrderPlacement(wit::EthFlowPlacement {
                sender: placement.sender.to_hex_string(),
                order_json,
                signing_scheme: signing_scheme.to_owned(),
                signature: alloy_primitives::hex::encode_prefixed(&placement.signature_data),
                data: alloy_primitives::hex::encode_prefixed(&placement.data),
            })
        }
        Source::OrderInvalidation(invalidation) => {
            wit::EthFlowEvent::OrderInvalidation(invalidation.order_uid.to_hex_string())
        }
        Source::OrderRefund(refund) => wit::EthFlowEvent::OrderRefund(wit::EthFlowRefund {
            order_uid: refund.order_uid.to_hex_string(),
            refunder: refund.refunder.to_hex_string(),
        }),
        _ => return Err("unsupported eth-flow event".to_owned()),
    })
}

export!(Component);
