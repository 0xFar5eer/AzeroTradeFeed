use log::error;
use reqwest::{header::HeaderMap, Client, Url};
use serde::{de::DeserializeOwned, Serialize};
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;

static DELAY_MS: u64 = 100;
static TIMEOUT_MS: u64 = 10_000;

#[derive(Clone, Debug)]
pub struct HttpClient {
    pub client_name: String,
    pub client: Client,
}

impl HttpClient {
    pub async fn new(client_name: &str) -> HttpClient {
        loop {
            let client = Client::builder()
                .timeout(Duration::from_millis(TIMEOUT_MS))
                .build();
            if let Err(e) = client {
                error!(target: &format!("http_client_{client_name}"), "Create client error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let client = client.unwrap();

            return Self {
                client,
                client_name: client_name.to_string(),
            };
        }
    }

    pub async fn get_request<T>(&mut self, url: &str, params: Option<HashMap<String, String>>) -> T
    where
        T: Serialize,
        T: DeserializeOwned,
        T: Unpin,
        T: Send,
        T: Sync,
    {
        let url = match params {
            None => Url::parse(url).unwrap(),
            Some(params) => {
                Url::parse_with_params(url, params.iter()).unwrap_or(Url::parse(url).unwrap())
            }
        };
        loop {
            let resp = self.client.get(url.clone()).send().await;
            if let Err(e) = resp {
                error!(target: &format!("http_client_{}", self.client_name), "get_request send error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let resp = resp.unwrap().text().await;
            if let Err(e) = resp {
                error!(target: &format!("http_client_{}", self.client_name), "get_request response error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let resp = resp.unwrap();

            let resp: Result<T, _> = serde_json::from_str(&resp);
            if let Err(e) = resp {
                error!(target: &format!("http_client_{}", self.client_name), "get_request parse response error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let resp = resp.unwrap();
            return resp;
        }
    }

    pub async fn post_request<T, Y>(&mut self, url: &str, headers: HeaderMap, data: Y) -> T
    where
        Y: Serialize,
        T: DeserializeOwned,
        T: Unpin,
        T: Send,
        T: Sync,
    {
        loop {
            let resp = self
                .client
                .post(url)
                .headers(headers.clone())
                .json(&data)
                .send()
                .await;
            if let Err(e) = resp {
                error!(target: &format!("http_client_{}", self.client_name), "post_request send error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let resp = resp.unwrap().text().await;
            if let Err(e) = resp {
                error!(target: &format!("http_client_{}", self.client_name), "post_request response error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let resp = resp.unwrap();

            let resp: Result<T, _> = serde_json::from_str(&resp);
            if let Err(e) = resp {
                error!(target: &format!("http_client_{}", self.client_name), "post_request parse response error: {e}; Sleeping {DELAY_MS} ms.");

                sleep(Duration::from_millis(DELAY_MS)).await;
                continue;
            }

            let resp = resp.unwrap();
            return resp;
        }
    }
}
