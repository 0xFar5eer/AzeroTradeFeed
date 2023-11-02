use log::{error, info};
use rs_subscan_parser::{
    mongodb_client_identities::MongoDbClientIdentity, mongodb_client_subscan::MongoDbClientSubscan,
    mongodb_client_validator::MongoDbClientValidator, subscan_stake_parser::parse_staking,
};
use rs_utils::utils::logger::initialize_logger;
// use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main(worker_threads = 100)]
async fn main() {
    // let stash_wallet =
    //     "0x7c0109f738ba3beab4f0cadb85cbec36d66eb1f12b0dcda90f0c482467b7c867"[2..].to_string();
    // let decoded = hex::decode(stash_wallet).ok().unwrap();
    // let byte_arr: [u8; 32] = decoded.try_into().ok().unwrap();
    // let address =
    //     AccountId32::from(byte_arr).to_ss58check_with_version(Ss58AddressFormat::custom(42));

    initialize_logger().expect("failed to initialize logging.");

    info!(target: "subscan_parser", "Started subscan parser worker.");

    start_worker().await;
}

async fn start_worker() {
    let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
    mongodb_client_subscan.create_index().await;

    let mut mongodb_client_validator = MongoDbClientValidator::new().await;
    mongodb_client_validator.create_index().await;

    let mut mongodb_client_identity = MongoDbClientIdentity::new().await;
    mongodb_client_identity.create_index().await;

    loop {
        let subscan_operations = parse_staking().await;
        let Some(subscan_operations) = subscan_operations else {
            error!(
                target: "subscan_parser", "Nothing found",
            );
            sleep(Duration::from_millis(1_000)).await;
            continue;
        };

        let subscan_operations_len = subscan_operations.len();
        let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
        mongodb_client_subscan
            .import_subscan_operations(subscan_operations)
            .await;

        info!(
            target: "subscan_parser", "Imported {} items",
            subscan_operations_len,
        );
        sleep(Duration::from_millis(1_000)).await;
    }
}
