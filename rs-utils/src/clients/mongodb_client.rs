use bson::{doc, Bson, Document};
use futures::StreamExt;
use log::error;
use mongodb::{
    options::{
        ClientOptions, CountOptions, CreateIndexOptions, DeleteOptions, FindOneOptions,
        FindOptions, InsertOneOptions, UpdateOptions,
    },
    results::{CreateIndexResult, DeleteResult, UpdateResult},
    Client, Collection, Database, IndexModel,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{borrow::Borrow, time::Duration};
use tokio::time::sleep;

static DELAY_MS: u64 = 100;

pub struct MongoDbClient<T> {
    pub client_name: String,
    pub client: Client,
    pub db: Database,
    pub col: Collection<T>,
}

impl<T> MongoDbClient<T>
where
    T: Serialize,
    T: DeserializeOwned,
    T: Unpin,
    T: Send,
    T: Sync,
{
    pub async fn new(
        uri: &str,
        client_name: &str,
        database: &str,
        collection: &str,
    ) -> MongoDbClient<T> {
        loop {
            let client_options = ClientOptions::parse(uri).await;

            if let Err(e) = client_options {
                error!(target: &format!("mongodb_client_{client_name}"), "Parse MongodbUri error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let mut client_options = client_options.unwrap();
            client_options.app_name = Some(client_name.to_string());
            client_options.connect_timeout = Some(Duration::from_secs(10));
            client_options.server_selection_timeout = Some(Duration::from_secs(10));
            client_options.max_idle_time = Some(Duration::from_secs(90));
            client_options.min_pool_size = Some(1);
            client_options.max_pool_size = Some(1);
            client_options.retry_reads = Some(true);
            client_options.retry_writes = Some(true);
            client_options.direct_connection = Some(true);

            let client = Client::with_options(client_options);

            if let Err(e) = client {
                error!(target: &format!("mongodb_client_{client_name}"), "Connection error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let client = client.unwrap();
            let db = client.database(database);
            let col = db.collection::<T>(collection);

            return Self {
                client,
                db,
                col,
                client_name: client_name.to_string(),
            };
        }
    }

    pub async fn update_one(
        &mut self,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> UpdateResult {
        loop {
            let res = self
                .col
                .update_one(query.clone(), update.clone(), options.clone())
                .await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "update_one error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn update_many(
        &mut self,
        query: Document,
        update: Document,
        options: Option<UpdateOptions>,
    ) -> UpdateResult {
        loop {
            let res = self
                .col
                .update_many(query.clone(), update.clone(), options.clone())
                .await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "update_many error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn count_documents(&mut self, query: Document, options: Option<CountOptions>) -> u64 {
        loop {
            let res = self
                .col
                .count_documents(query.clone(), options.clone())
                .await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "count_documents error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn create_index(
        &mut self,
        index: IndexModel,
        options: Option<CreateIndexOptions>,
    ) -> CreateIndexResult {
        loop {
            let res = self.col.create_index(index.clone(), options.clone()).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "create_index error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn insert_one(
        &mut self,
        doc: impl Borrow<T> + Clone,
        options: Option<InsertOneOptions>,
    ) {
        loop {
            let res = self.col.insert_one(doc.clone(), options.clone()).await;
            if let Err(e) = res {
                if e.to_string()
                    .contains("E11000 duplicate key error collection")
                {
                    return;
                }
                error!(target: &format!("mongodb_client_{}", self.client_name), "insert_one error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return;
        }
    }

    pub async fn delete_one(
        &mut self,
        query: Document,
        options: Option<DeleteOptions>,
    ) -> DeleteResult {
        loop {
            let res = self.col.delete_one(query.clone(), options.clone()).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "delete_one error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn delete_many(
        &mut self,
        query: Document,
        options: Option<DeleteOptions>,
    ) -> DeleteResult {
        loop {
            let res = self.col.delete_many(query.clone(), options.clone()).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "delete_many error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn find_one(
        &mut self,
        query: Document,
        options: Option<FindOneOptions>,
    ) -> Option<T> {
        loop {
            let res = self.col.find_one(query.clone(), options.clone()).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "find_one error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            return res.unwrap();
        }
    }

    pub async fn find(&mut self, query: Document, options: Option<FindOptions>) -> Vec<T> {
        let mut cur;
        loop {
            let res = self.col.find(query.clone(), options.clone()).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "find error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            cur = res.unwrap();
            break;
        }

        let mut output = Vec::new();
        while let Some(res) = cur.next().await {
            if let Err(e) = res {
                if e.to_string().contains("Cannot run getMore") {
                    break;
                }
                error!(target: &format!("mongodb_client_{}", self.client_name), "find cur.next error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            output.push(res.unwrap());
        }

        output
    }

    pub async fn distinct(&mut self, field: &str) -> Vec<Bson> {
        loop {
            let res = self.col.distinct(field, None, None).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "distinct error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let res = res.unwrap();
            return res;
        }
    }

    pub async fn distinct_huge(&mut self, query: Document, field: &str) -> Vec<String> {
        let mut cur;
        let field = format!("${field}");
        let pipeline = [
            doc! {
                "$match": query,
            },
            doc! {
                "$group": {
                    "_id": field
                }
            },
        ];
        loop {
            let res = self.col.aggregate(pipeline.clone(), None).await;
            if let Err(e) = res {
                error!(target: &format!("mongodb_client_{}", self.client_name), "aggregate error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            cur = res.unwrap();
            break;
        }

        let mut output = Vec::new();
        while let Some(res) = cur.next().await {
            if let Err(e) = res {
                if e.to_string().contains("Cannot run getMore") {
                    break;
                }
                error!(target: &format!("mongodb_client_{}", self.client_name), "aggregate, cur.next error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            output.push(res.unwrap());
        }

        output
            .into_iter()
            .map(|a| a.get_str("_id").unwrap().to_string())
            .collect::<Vec<_>>()
    }
}
