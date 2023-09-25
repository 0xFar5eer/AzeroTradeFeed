use log::info;
use rs_subscan_parser::mongodb_client_subscan::MongoDbClientSubscan;
use rs_utils::utils::logger::initialize_logger;

#[tokio::main(worker_threads = 10)]
async fn main() {
    initialize_logger().expect("failed to initialize logging.");

    info!(target: "subscan_parser", "Started subscan parser worker.");

    start_worker().await;
}

async fn start_worker() {
    let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
    mongodb_client_subscan.create_index().await;

    // loop {
    //     let mut tasks = FuturesUnordered::new();
    //     tasks.push(tokio::spawn(async move {
    //         let mut parser = MexcParser::new().await;

    //         let primary_token = PrimaryToken::Azero;
    //         let secondary_token = SecondaryToken::Usdt;
    //         let exchange = Subscan::Mexc;
    //         (
    //             exchange,
    //             primary_token.clone(),
    //             secondary_token.clone(),
    //             parser.parse(primary_token, secondary_token).await,
    //         )
    //     }));

    //     tasks.push(tokio::spawn(async move {
    //         let mut parser = MexcParser::new().await;

    //         let primary_token = PrimaryToken::Azero;
    //         let secondary_token = SecondaryToken::Usdc;
    //         let exchange = Subscan::Mexc;
    //         (
    //             exchange,
    //             primary_token.clone(),
    //             secondary_token.clone(),
    //             parser.parse(primary_token, secondary_token).await,
    //         )
    //     }));

    //     tasks.push(tokio::spawn(async move {
    //         let mut parser = KucoinParser::new().await;

    //         let primary_token = PrimaryToken::Azero;
    //         let secondary_token = SecondaryToken::Usdt;
    //         let exchange = Subscan::Kucoin;
    //         (
    //             exchange,
    //             primary_token.clone(),
    //             secondary_token.clone(),
    //             parser.parse(primary_token, secondary_token).await,
    //         )
    //     }));

    //     tasks.push(tokio::spawn(async move {
    //         let mut parser = GateParser::new().await;

    //         let primary_token = PrimaryToken::Azero;
    //         let secondary_token = SecondaryToken::Usdt;
    //         let exchange = Subscan::Gate;
    //         (
    //             exchange,
    //             primary_token.clone(),
    //             secondary_token.clone(),
    //             parser.parse(primary_token, secondary_token).await,
    //         )
    //     }));

    //     let mut all_subscan_trades = Vec::new();
    //     while let Some(res) = tasks.next().await {
    //         let Ok((exchange, primary_token, secondary_token, one_exchange_trades)) = res else {
    //             continue;
    //         };

    //         let Some(mut one_exchange_trades) = one_exchange_trades else {
    //             continue;
    //         };

    //         info!(
    //             target: "exchanges_parser", "Imported {} items from {} ({}_{})",
    //             one_exchange_trades.len(),
    //             exchange,
    //             primary_token.to_string().to_uppercase(),
    //             secondary_token.to_string().to_uppercase()
    //         );

    //         all_subscan_trades.append(&mut one_exchange_trades);
    //     }

    //     let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
    //     mongodb_client_subscan
    //         .import_exchange(all_subscan_trades)
    //         .await;
    //     sleep(Duration::from_millis(100)).await;
    // }
}
