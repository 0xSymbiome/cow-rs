use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_core::{Address, Amount, AppDataHash, OrderBalance, OrderKind, SupportedChainId};
use cow_sdk_trading::{LimitTradeParameters, OrderToSignParams, get_order_to_sign};

fn sample_limit_parameters() -> LimitTradeParameters {
    LimitTradeParameters {
        kind: OrderKind::Sell,
        owner: None,
        sell_token: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        sell_token_decimals: 18,
        buy_token: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        buy_token_decimals: 18,
        sell_amount: Amount::new("1000000000000000000").unwrap(),
        buy_amount: Amount::new("2000000000000000000000").unwrap(),
        quote_id: None,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: OrderBalance::Erc20,
        buy_token_balance: OrderBalance::Erc20,
        slippage_bps: Some(50),
        receiver: None,
        valid_for: None,
        valid_to: Some(1_900_000_000),
        partner_fee: None,
    }
}

fn sample_params() -> OrderToSignParams {
    OrderToSignParams {
        chain_id: SupportedChainId::Mainnet,
        from: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        is_ethflow: false,
        network_costs_amount: None,
        apply_costs_slippage_and_fees: false,
        protocol_fee_bps: None,
    }
}

fn bench_get_order_to_sign(c: &mut Criterion) {
    let parameters = sample_limit_parameters();
    let params = sample_params();
    let app_data_hash =
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap();
    c.bench_function("get_order_to_sign", |b| {
        b.iter(|| {
            let order = get_order_to_sign(
                black_box(params.clone()),
                black_box(&parameters),
                black_box(&app_data_hash),
            )
            .expect("fixed limit parameters must resolve to a signed-order payload");
            black_box(order);
        });
    });
}

criterion_group!(benches, bench_get_order_to_sign);
criterion_main!(benches);
