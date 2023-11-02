use chrono::Utc;
use log::info;
use num_format::{Locale, ToFormattedString};
use rs_exchanges_parser::{
    mongodb_client_exchanges::MongoDbClientExchanges, ExchangeTrade, Exchanges, PrimaryToken,
    TradeType,
};
use rs_subscan_parser::{
    mongodb_client_identities::MongoDbClientIdentity, mongodb_client_subscan::MongoDbClientSubscan,
    OperationType,
};
use rs_telegram_feed_bot::{
    mongodb_client_telegram::MongoDbClientTelegram, telegram_posting::TelegramPosting, Telegram,
};
use rs_utils::utils::logger::initialize_logger;
use std::{cmp, env, time::Duration};
use tokio::time::sleep;

static FILTER_MIN_USD: f64 = 1_000.0;

#[tokio::main(worker_threads = 100)]
async fn main() {
    initialize_logger().expect("failed to initialize logging.");

    info!(target: "telegram_feed_bot", "Started telegram feed worker.");

    start_worker().await;
}

async fn start_worker() {
    let mut mongodb_client_telegram = MongoDbClientTelegram::new().await;
    mongodb_client_telegram.create_index().await;

    let bot_father_key = &env::var("TELEGRAM_BOT_FATHER_KEY").unwrap();
    let channel_id = &env::var("TELEGRAM_CHANNEL_ID").unwrap();

    loop {
        let mut mongodb_client_exchanges = MongoDbClientExchanges::new().await;
        let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
        let mut mongodb_client_identity = MongoDbClientIdentity::new().await;

        let from_timestamp = Utc::now().timestamp() - 60 * 60;
        let subscan_operations = mongodb_client_subscan
            .get_filtered_operations(from_timestamp, None)
            .await;
        let subscan_operations = subscan_operations
            .into_iter()
            .filter(|p| p.operation_usd > FILTER_MIN_USD)
            .collect::<Vec<_>>();

        let non_grouped_exchanges_operations = mongodb_client_exchanges
            .get_filtered_trades(PrimaryToken::Azero, from_timestamp, None)
            .await;
        let mut exchanges_operations: Vec<ExchangeTrade> = Vec::new();
        for e in non_grouped_exchanges_operations {
            let found = exchanges_operations.iter_mut().find(|p| {
                p.trade_timestamp == e.trade_timestamp
                    && p.trade_type == e.trade_type
                    && p.exchange == e.exchange
            });
            let Some(found) = found else {
                exchanges_operations.push(e.clone());
                continue;
            };

            // getting geometric mean price of grouped trade
            found.trade_price =
                (found.trade_price * found.trade_quantity + e.trade_price + e.trade_quantity)
                    / (found.trade_quantity + e.trade_quantity);
            found.trade_quantity += e.trade_quantity;
        }
        let exchanges_operations = exchanges_operations
            .into_iter()
            .filter(|p| p.trade_price * p.trade_quantity > FILTER_MIN_USD)
            .collect::<Vec<_>>();

        let advertisement = r#"[🅰️ Stake with Azero Is Life Validator to support the feed development](https://azero.live/validator?address=5DEu6VG3WkJ1rdPadU4SffSse4sodA5PUE4apnw74c451Lak)"#;

        let mut subscan_counter = 0;
        let mut messages = Vec::new();
        for subscan_operation in subscan_operations {
            let circle = match subscan_operation.operation_type {
                OperationType::Stake => "🔵",
                OperationType::ReStake => "🟡",
                OperationType::RequestUnstake => "🟣",
                OperationType::WithdrawUnstaked => "⚫",
                OperationType::Transfer => "⚪",
            };

            let circles = get_circles(circle, subscan_operation.operation_usd);

            let from_identity = mongodb_client_identity
                .get_identity_by_address(&subscan_operation.from_wallet)
                .await
                .map(|p| p.identity)
                .unwrap_or(subscan_operation.from_wallet.clone());
            let to_identity = mongodb_client_identity
                .get_identity_by_address(&subscan_operation.to_wallet)
                .await
                .map(|p| p.identity)
                .unwrap_or(subscan_operation.to_wallet.clone());

            let message = match subscan_operation.operation_type {
                OperationType::Stake => format!(
                    r#"📘 Started stake of {} AZERO (${})

{circles}

From address: [{from_identity}](https://alephzero.subscan.io/account/{})
To validator: [{to_identity}](https://alephzero.subscan.io/account/{})

[📶 Tx Hash](https://alephzero.subscan.io/extrinsic/{}) | "#,
                    (subscan_operation.operation_quantity.floor() as u64)
                        .to_formatted_string(&Locale::en),
                    (subscan_operation.operation_usd.floor() as u64)
                        .to_formatted_string(&Locale::en),
                    subscan_operation.from_wallet,
                    subscan_operation.to_wallet,
                    subscan_operation.extrinsic_index,
                ),
                OperationType::ReStake => format!(
                    r#"📒 Re-staked stake of {} AZERO (${})

{circles}

From address: [{from_identity}](https://alephzero.subscan.io/account/{})
To validator: [{to_identity}](https://alephzero.subscan.io/account/{})

[📶 Tx Hash](https://alephzero.subscan.io/extrinsic/{}) | "#,
                    (subscan_operation.operation_quantity.floor() as u64)
                        .to_formatted_string(&Locale::en),
                    (subscan_operation.operation_usd.floor() as u64)
                        .to_formatted_string(&Locale::en),
                    subscan_operation.from_wallet,
                    subscan_operation.to_wallet,
                    subscan_operation.extrinsic_index,
                ),
                OperationType::RequestUnstake => {
                    format!(
                        r#"👿 Requested unstake of {} AZERO (${})

{circles}

From address: [{from_identity}](https://alephzero.subscan.io/account/{})
From validator: [{to_identity}](https://alephzero.subscan.io/account/{})

[📶 Tx Hash](https://alephzero.subscan.io/extrinsic/{}) | "#,
                        (subscan_operation.operation_quantity.floor() as u64)
                            .to_formatted_string(&Locale::en),
                        (subscan_operation.operation_usd.floor() as u64)
                            .to_formatted_string(&Locale::en),
                        subscan_operation.from_wallet,
                        subscan_operation.to_wallet,
                        subscan_operation.extrinsic_index,
                    )
                }
                OperationType::WithdrawUnstaked => {
                    format!(
                        r#"🤬 Withdraw unstaked of {} AZERO (${})

{circles}

From address: [{from_identity}](https://alephzero.subscan.io/account/{})
From validator: [{to_identity}](https://alephzero.subscan.io/account/{})

[📶 Tx Hash](https://alephzero.subscan.io/extrinsic/{}) | "#,
                        (subscan_operation.operation_quantity.floor() as u64)
                            .to_formatted_string(&Locale::en),
                        (subscan_operation.operation_usd.floor() as u64)
                            .to_formatted_string(&Locale::en),
                        subscan_operation.from_wallet,
                        subscan_operation.to_wallet,
                        subscan_operation.extrinsic_index,
                    )
                }
                OperationType::Transfer => {
                    format!(
                        r#"🔀 Transferred {} AZERO ({}$)
                    
{circles}

From address: [{from_identity}](https://alephzero.subscan.io/account/{})
To address: [{to_identity}](https://alephzero.subscan.io/account/{})

[📶 Tx Hash](https://alephzero.subscan.io/extrinsic/{}) | "#,
                        (subscan_operation.operation_quantity.floor() as u64)
                            .to_formatted_string(&Locale::en),
                        (subscan_operation.operation_usd.floor() as u64)
                            .to_formatted_string(&Locale::en),
                        subscan_operation.from_wallet,
                        subscan_operation.to_wallet,
                        subscan_operation.extrinsic_index
                    )
                }
            };

            messages.push(message);

            subscan_counter += 1;
        }

        let mut exchange_counter = 0;
        for exchanges_operation in exchanges_operations {
            let circle = match exchanges_operation.trade_type {
                TradeType::IsBuy => "🟢",
                TradeType::IsSell => "🔴",
            };

            let operation_usd =
                exchanges_operation.trade_price * exchanges_operation.trade_quantity;
            let circles = get_circles(circle, operation_usd);

            let amount_usd = exchanges_operation.trade_price * exchanges_operation.trade_quantity;

            let exchange = match exchanges_operation.exchange {
                Exchanges::Mexc => "🚹 Mexc",
                Exchanges::Kucoin => "🦚 Kucoin",
                Exchanges::Gate => "🚪 Gate",
            };

            let message = match exchanges_operation.trade_type {
                TradeType::IsSell => format!(
                    r#"👹 1 AZERO = {:.4} USDT
Sold {} AZERO for {} {} on {exchange}

{circles}

"#,
                    exchanges_operation.trade_price,
                    (exchanges_operation.trade_quantity.floor() as u64)
                        .to_formatted_string(&Locale::en),
                    (amount_usd.floor() as u64).to_formatted_string(&Locale::en),
                    exchanges_operation
                        .secondary_token
                        .to_string()
                        .to_uppercase(),
                ),
                TradeType::IsBuy => format!(
                    r#"🚀 1 AZERO = {:.4} USDT
Bought {} AZERO for {} {} on {exchange}

{circles}

"#,
                    exchanges_operation.trade_price,
                    (exchanges_operation.trade_quantity.floor() as u64)
                        .to_formatted_string(&Locale::en),
                    (amount_usd.floor() as u64).to_formatted_string(&Locale::en),
                    exchanges_operation
                        .secondary_token
                        .to_string()
                        .to_uppercase(),
                ),
            };

            messages.push(message);

            exchange_counter += 1;
        }

        let mut mongodb_client_telegram = MongoDbClientTelegram::new().await;
        let telegram_hashes = messages.iter().map(sha256::digest).collect();
        let non_existing_hashes = mongodb_client_telegram
            .get_not_existing_telegrams(telegram_hashes)
            .await;
        let messages_len = messages.len();

        let messages = messages
            .into_iter()
            .filter(|p| non_existing_hashes.contains(&sha256::digest(p)))
            .collect::<Vec<String>>();
        let skipped_counter = messages_len - messages.len();

        let mut telegram_posting = TelegramPosting::new(bot_father_key, channel_id).await;
        for message in messages {
            let message_with_advertisement = format!("{message}{advertisement}");
            telegram_posting
                .post_message(&message_with_advertisement)
                .await;

            let already_posted_hash = sha256::digest(&message);
            mongodb_client_telegram
                .import_telegrams(vec![Telegram {
                    already_posted_hash,
                }])
                .await;

            sleep(Duration::from_millis(250)).await;
        }

        info!(target: "telegram_posting", "Skipped {skipped_counter}. Posted {exchange_counter} trades and {subscan_counter} subscan operations. Sleeping 1 sec.");

        sleep(Duration::from_millis(1_000)).await;
    }
}

fn get_circles(circle: &str, operation_usd: f64) -> String {
    let circles_len = (operation_usd / 1_000.0).floor() as u64;
    let circles_len = cmp::max(1, circles_len);
    let circles_len = cmp::min(500, circles_len);
    let mut circles = String::new();
    for _ in 0..circles_len {
        circles = format!("{circles}{circle}");
    }

    circles
}
