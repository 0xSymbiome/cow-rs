fn takes_orderbook_client(_: Option<&dyn cow_sdk::trading::OrderbookClient>) {}

fn main() {
    takes_orderbook_client(None);
}
