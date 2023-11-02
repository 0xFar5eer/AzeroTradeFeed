use crate::Telegram;
use bson::doc;
use mongodb::{options::IndexOptions, IndexModel};
use rs_utils::clients::mongodb_client::MongoDbClient;
use std::env;

pub struct MongoDbClientTelegram {
    pub client_telegram: MongoDbClient<Telegram>,
}

impl MongoDbClientTelegram {
    pub async fn new() -> MongoDbClientTelegram {
        let uri = &env::var("MONGODB_URI").unwrap();
        let db = &env::var("MONGODB_DATABASE").unwrap();
        let col = &env::var("MONGODB_COLLECTION_TELEGRAM").unwrap();
        let client_name = "mongodb_telegram";
        let client_telegram = MongoDbClient::new(uri, client_name, db, col).await;

        Self { client_telegram }
    }

    pub async fn create_index(&mut self) {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! {"already_posted_hash": 1u32})
            .options(options)
            .build();
        self.client_telegram.create_index(model, None).await;
    }

    pub async fn import_telegrams(&mut self, telegrams: Vec<Telegram>) {
        for doc in telegrams {
            self.client_telegram.insert_one(doc, None).await;
        }
    }

    pub async fn get_not_existing_telegrams(
        &mut self,
        telegram_hashes: Vec<String>,
    ) -> Vec<String> {
        if telegram_hashes.is_empty() {
            return Vec::new();
        }

        let query = doc! {
            "already_posted_hash": {
                "$in": telegram_hashes.clone()
            }
        };

        let found = self
            .client_telegram
            .find(query, None)
            .await
            .into_iter()
            .map(|m| m.already_posted_hash)
            .collect::<Vec<String>>();

        telegram_hashes
            .into_iter()
            .filter(|m| !found.contains(m))
            .collect()
    }
}
