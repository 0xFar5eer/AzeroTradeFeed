use crate::utils::print_utils::{self, Status};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;

pub async fn http_get_json(url: &str) -> Value {
    loop {
        let client = reqwest::Client::builder().build().unwrap();

        let resp = client.get(url).timeout(Duration::from_secs(5)).send().await;
        if let Err(e) = resp {
            print_utils::print_status(
                Status::Err,
                "http_client",
                &format!("Request error: {}; Sleeping 100 ms.", e),
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        let resp = resp.unwrap().text().await;
        if let Err(e) = resp {
            print_utils::print_status(
                Status::Err,
                "http_client",
                &format!("Getting response error: {}; Sleeping 100 ms.", e),
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        let resp = resp.unwrap();
        if resp.contains("Too Many Requests") {
            print_utils::print_status(
                Status::Err,
                "http_client",
                "Too many requests; Sleeping 100 ms.",
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        let json: Result<Value, _> = serde_json::from_str(&resp);
        if let Err(e) = json {
            println!("{:#?}", resp);
            print_utils::print_status(
                Status::Err,
                "http_client",
                &format!("Parsing response (json) error: {}; Sleeping 100 ms.", e),
            );
            sleep(Duration::from_millis(100)).await;
            continue;
        }

        let json = json.unwrap();

        return json;
    }
}
