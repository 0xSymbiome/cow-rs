use criterion::{Criterion, black_box, criterion_group, criterion_main};

use cow_sdk_contracts::{ContractId, Order, Registry, hash_order};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
    SupportedChainId, TypedDataDomain,
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
    Order::new(
        Address::new("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").unwrap(),
        Address::new("0x6b175474e89094c44da98b954eedeac495271d0f").unwrap(),
        Some(Address::new("0x3333333333333333333333333333333333333333").unwrap()),
        Amount::new("1000000000000000000").unwrap(),
        Amount::new("2000000000000000000000").unwrap(),
        1_709_990_000,
        AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
            .unwrap(),
        Amount::new("5000000000000000").unwrap(),
        OrderKind::Sell,
        false,
        Some(SellTokenSource::Erc20),
        Some(BuyTokenDestination::Erc20),
    )
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
