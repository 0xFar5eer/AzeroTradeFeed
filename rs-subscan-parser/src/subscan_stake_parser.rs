use crate::{
    mongodb_client_identities::MongoDbClientIdentity,
    mongodb_client_subscan::MongoDbClientSubscan,
    mongodb_client_validator::MongoDbClientValidator,
    subscan_parser::{Network, SubscanParser, AZERO_DENOMINATOR},
    ExtrinsicsType, Module, SubscanOperation, Validator, MINIMUM_AZERO_TO_SAVE_TO_DB,
};
use futures::{stream::FuturesUnordered, StreamExt};
use itertools::Itertools;
use rs_exchanges_parser::{
    mongodb_client_exchanges::MongoDbClientExchanges, PrimaryToken, SecondaryToken,
};
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use std::collections::HashSet;
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
            let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
            subscan_parser
                .parse_subscan_operations("", Module::Staking, e, 100)
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

    // skipping already existing records
    let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
    let subscan_operations = mongodb_client_subscan
        .get_not_existing_operations(subscan_operations)
        .await;

    // adding from_wallet and operation_quantity
    let mut tasks = FuturesUnordered::new();
    for s in subscan_operations {
        let mut s_clone = s.clone();
        tasks.push(tokio::spawn(async move {
            let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
            let events = subscan_parser
                .parse_subscan_extrinsic_details(s.extrinsic_index)
                .await?;

            let stake_event = events.iter().find(|p| p.module_id == "staking")?;

            // event must have at least 2 parameters
            if stake_event.event_params.len() < 2 {
                return None;
            }

            let stash_param = stake_event.event_params.first()?;
            if stash_param.name != "stash" && stash_param.name != "who" {
                return None;
            }

            let amount_param = stake_event.event_params.last()?;
            if amount_param.name != "amount" {
                return None;
            }

            let stash_wallet = stash_param.value.clone()[2..].to_string();
            let decoded = hex::decode(stash_wallet).ok()?;
            let byte_arr: [u8; 32] = decoded.try_into().ok()?;
            let address = AccountId32::from(byte_arr)
                .to_ss58check_with_version(Ss58AddressFormat::custom(42));
            s_clone.from_wallet = address;
            s_clone.operation_quantity =
                amount_param.value.parse::<f64>().ok()? / AZERO_DENOMINATOR;

            Some(s_clone)
        }));
    }

    let mut subscan_operations = Vec::new();
    while let Some(res) = tasks.next().await {
        let Ok(s) = res else {
            continue;
        };

        let Some(s) = s else {
            continue;
        };
        subscan_operations.push(s);
    }

    // parsing batch all operations
    let batch_all_operations = tokio::spawn(async move {
        let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
        subscan_parser.parse_subscan_batch_all("", 0, 20).await
    })
    .await
    .ok()??;

    // skipping already existing records
    let mut batch_all_operations = mongodb_client_subscan
        .get_not_existing_operations(batch_all_operations)
        .await;

    subscan_operations.append(&mut batch_all_operations);

    // saving validators to db
    let validators = convert_operations_to_validators(subscan_operations.clone());
    let validators_task = tokio::spawn(async move {
        let mut mongodb_client_validator = MongoDbClientValidator::new().await;
        mongodb_client_validator
            .import_or_update_validators(validators)
            .await
    });

    // removing operations with less than MINIMUM_AZERO_TO_SAVE_TO_DB AZERO amount
    let mut subscan_operations = subscan_operations
        .into_iter()
        .filter(|p| p.operation_quantity > MINIMUM_AZERO_TO_SAVE_TO_DB)
        .collect::<Vec<_>>();

    // updating to current price
    let price = price_task.await.ok()??;
    for s in subscan_operations.iter_mut() {
        s.operation_usd = s.operation_quantity * price;
    }

    validators_task.await.ok()?;

    // getting nominators missing in validators DB to update them
    let nominators = subscan_operations
        .iter()
        .map(|m| m.from_wallet.clone())
        .unique()
        .collect::<Vec<String>>();
    let mut mongodb_client_validator = MongoDbClientValidator::new().await;
    let not_existing_nominators = mongodb_client_validator
        .get_not_existing_nominators(nominators)
        .await;

    // parsing validators for given non existing nominators
    let mut tasks = FuturesUnordered::new();
    for nominator in not_existing_nominators.into_iter() {
        let nominator_clone = nominator.clone();
        tasks.push(tokio::spawn(async move {
            let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
            subscan_parser
                .parse_subscan_batch_all(&nominator_clone, 0, 100)
                .await
        }));

        tasks.push(tokio::spawn(async move {
            let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
            subscan_parser
                .parse_subscan_operations(&nominator, Module::Staking, ExtrinsicsType::Nominate, 1)
                .await
        }));
    }

    let mut validators = Vec::new();
    while let Some(res) = tasks.next().await {
        let Ok(s) = res else {
            continue;
        };

        let Some(s) = s else {
            continue;
        };

        let mut v = convert_operations_to_validators(s);
        validators.append(&mut v);
    }

    // updating validators
    mongodb_client_validator
        .import_or_update_validators(validators)
        .await;

    for s in subscan_operations.iter_mut() {
        let to_wallet = mongodb_client_validator
            .get_validator_by_nominator(&s.from_wallet)
            .await;
        let Some(to_wallet) = to_wallet else {
            continue;
        };
        s.to_wallet = to_wallet.validator;
    }

    // for wallets with separate controller wallet, we should find out to which validator they staked from controller wallet
    for s in subscan_operations.iter_mut() {
        if !SubscanParser::is_address_empty(&s.to_wallet) {
            continue;
        }

        if SubscanParser::is_address_empty(&s.controller_wallet) {
            continue;
        }

        let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
        let controller_operations = subscan_parser
            .parse_subscan_operations(
                &s.controller_wallet,
                Module::Staking,
                ExtrinsicsType::Nominate,
                1,
            )
            .await;

        let Some(mut controller_operations) = controller_operations else {
            continue;
        };

        for c in controller_operations.iter_mut() {
            c.from_wallet = s.from_wallet.clone();
        }

        // updating validators
        mongodb_client_validator
            .import_or_update_validators(convert_operations_to_validators(controller_operations))
            .await;
    }

    for s in subscan_operations.iter_mut() {
        s.set_hash();

        let to_wallet = mongodb_client_validator
            .get_validator_by_nominator(&s.from_wallet)
            .await;
        let Some(to_wallet) = to_wallet else {
            continue;
        };
        s.to_wallet = to_wallet.validator;
    }

    // removing operations with less than MINIMUM_AZERO_TO_SAVE_TO_DB AZERO amount
    let subscan_operations = subscan_operations
        .into_iter()
        .filter(|p| p.operation_quantity > MINIMUM_AZERO_TO_SAVE_TO_DB)
        .collect::<Vec<_>>();

    let from_wallets = subscan_operations
        .iter()
        .map(|m| m.from_wallet.to_string())
        .collect::<Vec<_>>();

    let to_wallets = subscan_operations
        .iter()
        .map(|m| m.to_wallet.to_string())
        .collect::<Vec<_>>();
    let new_addresses: HashSet<String> =
        HashSet::from_iter(from_wallets.into_iter().chain(to_wallets.into_iter()));
    let new_addresses = new_addresses.into_iter().collect::<Vec<_>>();

    // skipping already existing records
    let mut mongodb_client_identity = MongoDbClientIdentity::new().await;
    let new_addresses = mongodb_client_identity
        .get_not_existing_addresses(new_addresses)
        .await;

    // parsing non existing identities
    let mut tasks = FuturesUnordered::new();
    for a in new_addresses {
        tasks.push(tokio::spawn(async move {
            let mut subscan_parser = SubscanParser::new(Network::Alephzero).await;
            subscan_parser.parse_subscan_identity(&a, 0, 1).await
        }));
    }

    let mut identities = Vec::new();
    while let Some(res) = tasks.next().await {
        let Ok(s) = res else {
            continue;
        };

        let Some(mut s) = s else {
            continue;
        };

        identities.append(&mut s);
    }

    // saving newly parsed identities
    mongodb_client_identity
        .import_or_update_identities(identities)
        .await;

    Some(subscan_operations)
}

fn convert_operations_to_validators(source: Vec<SubscanOperation>) -> Vec<Validator> {
    source
        .into_iter()
        .filter_map(|p| {
            if SubscanParser::is_address_empty(&p.to_wallet)
                || SubscanParser::is_address_empty(&p.to_wallet)
            {
                return None;
            }

            Some(Validator {
                nominator: p.from_wallet,
                validator: p.to_wallet,
            })
        })
        .collect()
}
