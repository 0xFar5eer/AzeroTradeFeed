use crate::{
    subscan_parser::{Network, SubscanParser},
    ExtrinsicsType, Module, SubscanOperation,
};
use futures::{stream::FuturesUnordered, StreamExt};
use rs_exchanges_parser::{
    mongodb_client_exchanges::MongoDbClientExchanges, PrimaryToken, SecondaryToken,
};
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use std::env;
use strum::IntoEnumIterator;

pub async fn parse_staking() -> Option<Vec<SubscanOperation>> {
    let price_task = tokio::spawn(async move {
        let mut mongodb_client_exchanges = MongoDbClientExchanges::new().await;
        mongodb_client_exchanges
            .get_usd_price(PrimaryToken::Azero, SecondaryToken::Usdt)
            .await
    });

    let mut tasks = FuturesUnordered::new();
    for e in ExtrinsicsType::iter() {
        tasks.push(tokio::spawn(async move {
            let subscan_api_key = &env::var("SUBSCAN_API_KEY").unwrap();
            let mut subscan_parser = SubscanParser::new(Network::Alephzero, subscan_api_key).await;
            subscan_parser
                .parse_subscan_operations(Module::Staking, e, 100)
                .await
        }));
    }

    let mut subscan_operations = Vec::new();
    while let Some(res) = tasks.next().await {
        let Ok(s) = res else {
            continue;
        };

        let Some(mut s) = s else {
            continue;
        };
        subscan_operations.append(&mut s);
    }

    // adding to_wallet and operation_quantity
    let event_indexes = subscan_operations
        .iter()
        .map(|s| format!("{}-2", s.block_number))
        .collect();
    let subscan_api_key = &env::var("SUBSCAN_API_KEY").unwrap();
    let mut subscan_parser = SubscanParser::new(Network::Alephzero, subscan_api_key).await;
    let events = subscan_parser.parse_subscan_events(event_indexes).await?;

    let mut i = 0;
    let mut subscan_operations = subscan_operations
        .iter()
        .filter_map(|s| {
            let event = events.get(i)?;
            i += 1;

            // skipping events where for some reason stash event is not on 2nd index
            if event.event_params.len() != 2 {
                return None;
            }

            let stash_param = event.event_params.first()?;
            let amount_param = event.event_params.last()?;

            let mut s = s.clone();
            let to_wallet = stash_param.value.clone()[2..].to_string();
            let decoded = hex::decode(to_wallet).ok()?;
            let byte_arr: [u8; 32] = decoded.try_into().ok()?;
            let address = AccountId32::from(byte_arr)
                .to_ss58check_with_version(Ss58AddressFormat::custom(42));
            s.to_wallet = address;
            s.operation_quantity = amount_param.value.parse::<f64>().ok()? / 1e12;

            Some(s)
        })
        .collect::<Vec<SubscanOperation>>();

    let price = price_task.await.ok()??;
    for s in subscan_operations.iter_mut() {
        s.operation_usd = s.operation_quantity * price;
        s.set_hash();
    }

    Some(subscan_operations)
}

#[cfg(test)]
mod tests {
    use crate::subscan_stake_parser::parse_staking;
    use rs_utils::utils::logger::initialize_logger;

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn subscan_stake_parser_works() {
        initialize_logger().expect("failed to initialize logging.");

        let res = parse_staking().await;
        assert!(res.is_some());
    }
}
