use crate::Identity;
use bson::doc;
use mongodb::{options::IndexOptions, IndexModel};
use rs_utils::clients::mongodb_client::MongoDbClient;
use std::env;

pub struct MongoDbClientIdentity {
    pub client_identity: MongoDbClient<Identity>,
}

impl MongoDbClientIdentity {
    pub async fn new() -> MongoDbClientIdentity {
        let uri = &env::var("MONGODB_URI").unwrap();
        let db = &env::var("MONGODB_DATABASE").unwrap();
        let col = &env::var("MONGODB_COLLECTION_IDENTITY").unwrap();
        let client_name = "mongodb_identity";
        let client_identity = MongoDbClient::new(uri, client_name, db, col).await;

        Self { client_identity }
    }

    pub async fn create_index(&mut self) {
        let options = IndexOptions::builder().unique(true).build();
        let model = IndexModel::builder()
            .keys(doc! {"address": 1u32})
            .options(options)
            .build();
        self.client_identity.create_index(model, None).await;

        let indexes = vec!["identity"];
        for index in indexes {
            let model = IndexModel::builder()
                .keys(doc! {index: 1u32})
                .options(None)
                .build();
            self.client_identity.create_index(model, None).await;
        }
    }

    pub async fn import_or_update_identities(&mut self, identities: Vec<Identity>) {
        for doc in identities {
            if self
                .client_identity
                .find_one(doc! { "address": doc.address.clone() }, None)
                .await
                .is_none()
            {
                self.client_identity.insert_one(doc, None).await;
                continue;
            }

            self.client_identity
                .update_one(
                    doc! { "address": doc.address },
                    doc! { "$set": { "identity": doc.identity }},
                    None,
                )
                .await;
        }
    }

    pub async fn get_identity_by_address(&mut self, address: &str) -> Option<Identity> {
        let query = doc! {
            "address": address
        };

        self.client_identity.find_one(query, None).await
    }

    pub async fn get_not_existing_addresses(&mut self, addresses: Vec<String>) -> Vec<String> {
        if addresses.is_empty() {
            return Vec::new();
        }

        let query = doc! {
            "address": {
                "$in": addresses.clone()
            }
        };

        let found = self
            .client_identity
            .find(query, None)
            .await
            .into_iter()
            .map(|m| m.address)
            .collect::<Vec<String>>();

        addresses
            .into_iter()
            .filter(|m| !found.contains(m))
            .collect()
    }
}
