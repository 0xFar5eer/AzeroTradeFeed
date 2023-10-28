use crate::Validator;
use bson::doc;
use mongodb::{options::IndexOptions, IndexModel};
use rs_utils::clients::mongodb_client::MongoDbClient;
use std::env;

pub struct MongoDbClientValidator {
    pub client_validator: MongoDbClient<Validator>,
}

impl MongoDbClientValidator {
    pub async fn new() -> MongoDbClientValidator {
        let uri = &env::var("MONGODB_URI").unwrap();
        let db = &env::var("MONGODB_DATABASE").unwrap();
        let col = &env::var("MONGODB_COLLECTION_VALIDATOR").unwrap();
        let client_name = "mongodb_validator";
        let client_validator = MongoDbClient::new(uri, client_name, db, col).await;

        Self { client_validator }
    }

    pub async fn create_index(&mut self) {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! {"nominator": 1u32})
            .options(options)
            .build();
        self.client_validator.create_index(model, None).await;

        let indexes = vec!["validator"];
        for index in indexes {
            let model = IndexModel::builder()
                .keys(doc! {index: 1u32})
                .options(None)
                .build();
            self.client_validator.create_index(model, None).await;
        }
    }

    pub async fn import_or_update_validators(&mut self, validator: Vec<Validator>) {
        for doc in validator {
            if self
                .client_validator
                .find_one(doc! { "nominator": doc.nominator.clone() }, None)
                .await
                .is_none()
            {
                self.client_validator.insert_one(doc, None).await;
                continue;
            }

            self.client_validator
                .update_one(
                    doc! { "nominator": doc.nominator },
                    doc! { "$set": { "validator": doc.validator }},
                    None,
                )
                .await;
        }
    }

    pub async fn get_validator_by_nominator(&mut self, nominator: &str) -> Option<Validator> {
        let query = doc! {
            "nominator": nominator
        };

        self.client_validator.find_one(query, None).await
    }

    pub async fn get_not_existing_nominators(&mut self, nominators: Vec<String>) -> Vec<String> {
        if nominators.is_empty() {
            return Vec::new();
        }

        let query = doc! {
            "nominator": {
                "$in": nominators.clone()
            }
        };

        let found = self
            .client_validator
            .find(query, None)
            .await
            .into_iter()
            .map(|m| m.nominator)
            .collect::<Vec<String>>();

        nominators
            .into_iter()
            .filter(|m| !found.contains(m))
            .collect()
    }
}
