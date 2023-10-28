use log::{error, info};
use rs_subscan_parser::{
    mongodb_client_subscan::MongoDbClientSubscan, mongodb_client_validator::MongoDbClientValidator,
    subscan_stake_parser::parse_staking,
};
use rs_utils::utils::logger::initialize_logger;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main(worker_threads = 100)]
async fn main() {
    initialize_logger().expect("failed to initialize logging.");

    info!(target: "subscan_parser", "Started subscan parser worker.");

    start_worker().await;
}

async fn start_worker() {
    let mut mongodb_client_subscan = MongoDbClientSubscan::new().await;
    mongodb_client_subscan.create_index().await;

    let mut mongodb_client_validator = MongoDbClientValidator::new().await;
    mongodb_client_validator.create_index().await;

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
