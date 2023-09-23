use crate::{ExchangeTrade, PrimaryToken};
use bson::doc;
use chrono::Utc;
use mongodb::{
    options::{FindOptions, IndexOptions},
    IndexModel,
};
use rs_utils::clients::mongodb_client::MongoDbClient;
use std::env;

pub struct MongoDbClientExchanges {
    pub client_exchanges: MongoDbClient<ExchangeTrade>,
}

impl MongoDbClientExchanges {
    pub async fn new() -> MongoDbClientExchanges {
        let uri = &env::var("MONGODB_URI").unwrap();
        let db = &env::var("MONGODB_DATABASE").unwrap();
        let col = &env::var("MONGODB_COLLECTION_EXCHANGES").unwrap();
        let client_name = "exchanges";
        let client_exchanges = MongoDbClient::new(uri, client_name, db, col).await;

        Self { client_exchanges }
    }

    pub async fn create_index(&mut self) {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! {"hash": 1u32})
            .options(options)
            .build();
        self.client_exchanges.create_index(model, None).await;

        let indexes = vec![
            "trade_timestamp",
            "trade_type",
            "primary_token",
            "secondary_token",
            "exchange",
        ];
        for index in indexes {
            let model = IndexModel::builder()
                .keys(doc! {index: 1u32})
                .options(None)
                .build();
            self.client_exchanges.create_index(model, None).await;
        }
    }

    pub async fn import_exchange(&mut self, exchanges: Vec<ExchangeTrade>) {
        for doc in exchanges {
            self.client_exchanges.insert_one(doc, None).await;
        }
    }

    pub async fn get_filtered_trades(
        &mut self,
        from_timestamp: i64,
        to_timestamp: Option<i64>,
        primary_token: PrimaryToken,
    ) -> Vec<ExchangeTrade> {
        let options = Some(
            FindOptions::builder()
                .sort(doc! {"trade_timestamp": 1i32})
                .build(),
        );
        let to_timestamp = to_timestamp.unwrap_or(Utc::now().timestamp());
        let query = doc! {
            "primary_token": primary_token.to_string(),
            "trade_timestamp": {
                "$gte": from_timestamp,
                "$lt": to_timestamp,
            }

        };

        self.client_exchanges.find(query, options).await
    }
}
