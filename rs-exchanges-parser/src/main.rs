use futures::{stream::FuturesUnordered, StreamExt};
use log::info;
use rs_exchanges_parser::{
    exchange_parsers::{
        gate_parser::GateParser, kucoin_parser::KucoinParser, mexc_parser::MexcParser,
    },
    mongodb_client_exchanges::MongoDbClientExchanges,
    Exchanges, PrimaryToken, SecondaryToken,
};
use rs_utils::utils::logger::initialize_logger;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main(worker_threads = 10)]
async fn main() {
    initialize_logger().expect("failed to initialize logging.");

    start_worker().await;
}

async fn start_worker() {
    let mut mongodb_client_exchanges = MongoDbClientExchanges::new().await;
    mongodb_client_exchanges.create_index().await;

    loop {
        let mut tasks = FuturesUnordered::new();
        tasks.push(tokio::spawn(async move {
            let mut parser = MexcParser::new().await;

            let primary_token = PrimaryToken::Azero;
            let secondary_token = SecondaryToken::Usdt;
            let exchange = Exchanges::Mexc;
            (
                exchange,
                primary_token.clone(),
                secondary_token.clone(),
                parser.parse(primary_token, secondary_token).await,
            )
        }));

        tasks.push(tokio::spawn(async move {
            let mut parser = MexcParser::new().await;

            let primary_token = PrimaryToken::Azero;
            let secondary_token = SecondaryToken::Usdc;
            let exchange = Exchanges::Mexc;
            (
                exchange,
                primary_token.clone(),
                secondary_token.clone(),
                parser.parse(primary_token, secondary_token).await,
            )
        }));

        tasks.push(tokio::spawn(async move {
            let mut parser = KucoinParser::new().await;

            let primary_token = PrimaryToken::Azero;
            let secondary_token = SecondaryToken::Usdt;
            let exchange = Exchanges::Kucoin;
            (
                exchange,
                primary_token.clone(),
                secondary_token.clone(),
                parser.parse(primary_token, secondary_token).await,
            )
        }));

        tasks.push(tokio::spawn(async move {
            let mut parser = GateParser::new().await;

            let primary_token = PrimaryToken::Azero;
            let secondary_token = SecondaryToken::Usdt;
            let exchange = Exchanges::Gate;
            (
                exchange,
                primary_token.clone(),
                secondary_token.clone(),
                parser.parse(primary_token, secondary_token).await,
            )
        }));

        let mut all_exchanges_trades = Vec::new();
        while let Some(res) = tasks.next().await {
            let Ok((exchange, primary_token, secondary_token, one_exchange_trades)) = res else {
                continue;
            };

            let Some(mut one_exchange_trades) = one_exchange_trades else {
                continue;
            };

            info!(
                target: "exchanges_parser", "Imported {} items from {} ({}_{})",
                one_exchange_trades.len(),
                exchange,
                primary_token.to_string().to_uppercase(),
                secondary_token.to_string().to_uppercase()
            );

            all_exchanges_trades.append(&mut one_exchange_trades);
        }

        let mut mongodb_client_exchanges = MongoDbClientExchanges::new().await;
        mongodb_client_exchanges
            .import_exchange(all_exchanges_trades)
            .await;
        sleep(Duration::from_millis(100)).await;
    }
}
