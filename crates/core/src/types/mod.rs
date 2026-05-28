//! Strongly typed user-domain values used across the SDK surface.

pub use self::{amount::*, app_code::*, identity::*, order::*, quote::*, validity::*};

mod amount;
mod app_code;
mod identity;
mod order;
mod quote;
mod validity;
#[cfg(test)]
mod tests {
    use super::{
        Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
        UnsignedOrder,
    };

    #[test]
    fn unsigned_order_builder_serializes_identically_to_internal_literal_construction() {
        let sell_token = Address::new("0x1111111111111111111111111111111111111111").unwrap();
        let buy_token = Address::new("0x2222222222222222222222222222222222222222").unwrap();
        let receiver = Address::new("0x3333333333333333333333333333333333333333").unwrap();
        let sell_amount = Amount::new("100").unwrap();
        let buy_amount = Amount::new("200").unwrap();
        let valid_to = 1_700_000_000;
        let app_data =
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap();
        let fee_amount = Amount::new("5").unwrap();

        let from_builder = UnsignedOrder::new(
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            Amount::ZERO,
            OrderKind::Sell,
            false,
            SellTokenSource::Erc20,
            BuyTokenDestination::Erc20,
        )
        .with_receiver(receiver)
        .with_app_data(app_data)
        .with_fee_amount(fee_amount)
        .with_partially_fillable(true)
        .with_sell_token_balance(SellTokenSource::External)
        .with_buy_token_balance(BuyTokenDestination::Internal);

        let from_literal = UnsignedOrder {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind: OrderKind::Sell,
            partially_fillable: true,
            sell_token_balance: SellTokenSource::External,
            buy_token_balance: BuyTokenDestination::Internal,
        };

        assert_eq!(
            serde_json::to_vec(&from_builder).unwrap(),
            serde_json::to_vec(&from_literal).unwrap()
        );
    }
}
