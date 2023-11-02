use crate::SubscanOperation;
use bson::doc;
use chrono::Utc;
use mongodb::{
    options::{FindOptions, IndexOptions},
    IndexModel,
};
use rs_utils::clients::mongodb_client::MongoDbClient;
use std::{env, time::Duration};

static RECORDS_TTL_SECONDS: u64 = 90 * 24 * 60 * 60;

pub struct MongoDbClientSubscan {
    pub client_subscan: MongoDbClient<SubscanOperation>,
}

impl MongoDbClientSubscan {
    pub async fn new() -> MongoDbClientSubscan {
        let uri = &env::var("MONGODB_URI").unwrap();
        let db = &env::var("MONGODB_DATABASE").unwrap();
        let col = &env::var("MONGODB_COLLECTION_SUBSCAN").unwrap();
        let client_name = "mongodb_subscan";
        let client_subscan = MongoDbClient::new(uri, client_name, db, col).await;

        Self { client_subscan }
    }

    pub async fn create_index(&mut self) {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! {"hash": 1u32})
            .options(options)
            .build();
        self.client_subscan.create_index(model, None).await;

        let options = IndexOptions::builder()
            .unique(false)
            .expire_after(Duration::from_secs(RECORDS_TTL_SECONDS))
            .build();
        let model = IndexModel::builder()
            .keys(doc! {"operation_timestamp": 1u32})
            .options(options)
            .build();
        self.client_subscan.create_index(model, None).await;

        let indexes = vec![
            "operation_type",
            "from_wallet",
            "to_wallet",
            "extrinsic_index",
        ];
        for index in indexes {
            let model = IndexModel::builder()
                .keys(doc! {index: 1u32})
                .options(None)
                .build();
            self.client_subscan.create_index(model, None).await;
        }
    }

    pub async fn import_subscan_operations(&mut self, subscan: Vec<SubscanOperation>) {
        for doc in subscan {
            self.client_subscan.insert_one(doc, None).await;
        }
    }

    pub async fn get_filtered_operations(
        &mut self,
        from_timestamp: i64,
        to_timestamp: Option<i64>,
    ) -> Vec<SubscanOperation> {
        let options = Some(
            FindOptions::builder()
                .sort(doc! {"operation_timestamp": 1i32})
                .build(),
        );
        let to_timestamp = to_timestamp.unwrap_or(Utc::now().timestamp());
        let query = doc! {
            "operation_timestamp": {
                "$gte": from_timestamp,
                "$lt": to_timestamp,
            }

        };

        self.client_subscan.find(query, options).await
    }

    pub async fn get_not_existing_operations(
        &mut self,
        subscan_operations: Vec<SubscanOperation>,
    ) -> Vec<SubscanOperation> {
        if subscan_operations.is_empty() {
            return Vec::new();
        }

        let indexes = subscan_operations
            .iter()
            .map(|p| p.extrinsic_index.to_string())
            .collect::<Vec<String>>();
        let query = doc! {
            "extrinsic_index": {
                "$in": indexes
            }
        };

        let found = self
            .client_subscan
            .find(query, None)
            .await
            .into_iter()
            .map(|m| m.extrinsic_index)
            .collect::<Vec<String>>();

        subscan_operations
            .into_iter()
            .filter(|m| !found.contains(&m.extrinsic_index))
            .collect()
    }
}
