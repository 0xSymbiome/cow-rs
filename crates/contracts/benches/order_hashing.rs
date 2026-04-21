use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_contracts::{ContractId, Order, Registry, hash_order};
use cow_sdk_core::{
    Address, Amount, AppDataHash, CowEnv, OrderBalance, OrderKind, SupportedChainId,
    TypedDataDomain,
};

fn sample_domain() -> TypedDataDomain {
    TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: 1,
        verifying_contract: Registry::default()
            .address(
                ContractId::Settlement,
                SupportedChainId::Mainnet,
                CowEnv::Prod,
            )
            .expect("canonical settlement address is registered for every supported chain"),
    }
}

fn sample_order() -> Order {
    Order {
        sell_token: Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        buy_token: Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        receiver: Some(Address::new("0x3333333333333333333333333333333333333333").unwrap()),
        sell_amount: Amount::new("1000000000000000000").unwrap(),
        buy_amount: Amount::new("2000000000000000000000").unwrap(),
        valid_to: 1_709_990_000,
        app_data: AppDataHash::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .unwrap(),
        fee_amount: Amount::new("5000000000000000").unwrap(),
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: Some(OrderBalance::Erc20),
        buy_token_balance: Some(OrderBalance::Erc20),
    }
}

fn bench_hash_order(c: &mut Criterion) {
    let domain = sample_domain();
    let order = sample_order();
    c.bench_function("hash_order", |b| {
        b.iter(|| {
            let digest = hash_order(black_box(&domain), black_box(&order))
                .expect("fixed sample order must hash successfully");
            black_box(digest);
        });
    });
}

criterion_group!(benches, bench_hash_order);
criterion_main!(benches);
