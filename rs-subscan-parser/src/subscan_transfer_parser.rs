use crate::{
    mongodb_client_identities::MongoDbClientIdentity,
    subscan_parser::{Network, SubscanParser},
    SubscanOperation,
};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use rs_exchanges_parser::{
    mongodb_client_exchanges::MongoDbClientExchanges, PrimaryToken, SecondaryToken,
};
use std::collections::HashSet;

pub async fn parse_transfers() -> Option<Vec<SubscanOperation>> {
    let price_task = tokio::spawn(async move {
        let mut mongodb_client_exchanges = MongoDbClientExchanges::new().await;
        mongodb_client_exchanges
            .get_usd_price(PrimaryToken::Azero, SecondaryToken::Usdt)
            .await
    });

    let mut tasks = FuturesUnordered::new();
    for page in 0..10 {
        tasks.push(tokio::spawn(async move {
            let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
            subscan_parser.parse_subscan_transfers(page, 100).await
        }));
    }

    let mut subscan_operations = Vec::new();
    let mut identities = HashSet::new();
    while let Some(res) = tasks.next().await {
        let Ok(s) = res else {
            continue;
        };

        let Some((mut s, d)) = s else {
            continue;
        };
        subscan_operations.append(&mut s);
        for dd in d {
            identities.insert(dd);
        }
    }

    let identities = identities.into_iter().collect_vec();

    // removing operations with less than 2000 AZERO amount
    let mut subscan_operations = subscan_operations
        .into_iter()
        .filter(|p| p.operation_quantity > 2000.001)
        .collect::<Vec<_>>();

    // updating to current price
    let price = price_task.await.ok()??;
    for s in subscan_operations.iter_mut() {
        s.operation_usd = s.operation_quantity * price;

        s.set_hash();
    }

    // saving newly parsed identities
    let mut mongodb_client_identity = MongoDbClientIdentity::new().await;
    mongodb_client_identity
        .import_or_update_identities(identities)
        .await;

    Some(subscan_operations)
}
