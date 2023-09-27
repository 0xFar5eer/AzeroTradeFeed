use crate::{
    ExtrinsicsType, Module, OperationType, SubscanEvent, SubscanEventParam, SubscanOperation,
};
use bson::DateTime;
use reqwest::header::{HeaderMap, HeaderValue};
use rs_utils::clients::http_client::HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    EnumString,
    Default,
    IntoStaticStr,
    EnumIter,
    Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[strum(serialize_all = "snake_case")]
pub enum Network {
    #[default]
    Alephzero,
}

#[derive(Clone, Debug)]
pub struct SubscanParser {
    http_client: HttpClient,
    api_key: String,
    network: String,
}

impl SubscanParser {
    pub async fn new(network: Network, api_key: &str) -> Self {
        let http_client = HttpClient::new("subscan_parser").await;
        SubscanParser {
            network: network.to_string(),
            http_client,
            api_key: api_key.to_string(),
        }
    }

    pub async fn parse_subscan_events(
        &mut self,
        block_number: u64,
        event_ids: Vec<u32>,
    ) -> Option<Vec<SubscanEvent>> {
        let event_indexes = event_ids
            .iter()
            .map(|e| format!("{block_number}-{e}"))
            .collect::<Vec<String>>();
        let url = format!("https://{}.api.subscan.io/api/scan/event", self.network);

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key).unwrap());

        let data = json!({"event_index": event_indexes});

        let resp = self
            .http_client
            .post_request::<Value, Value>(&url, headers, data)
            .await;

        let data = resp.get("data")?.as_array()?;
        let subscan_events = data
            .iter()
            .filter_map(|d| -> Option<_> {
                let event_index = d.get("event_index")?.as_str()?.to_string();
                let event_params = d
                    .get("params")?
                    .as_array()?
                    .iter()
                    .filter_map(|p| {
                        let type_name = p.get("type_name")?.as_str()?.to_string();
                        let value = p.get("value")?.as_str()?.to_string();
                        let name = p.get("name")?.as_str()?.to_string();

                        Some(SubscanEventParam {
                            type_name,
                            value,
                            name,
                        })
                    })
                    .collect();

                Some(SubscanEvent {
                    event_index,
                    event_params,
                })
            })
            .collect::<Vec<SubscanEvent>>();
        Some(subscan_events)
    }

    pub async fn parse_subscan_operations(
        &mut self,
        module: Module,
        extrinsics_type: ExtrinsicsType,
    ) -> Option<Vec<SubscanOperation>> {
        let url = format!(
            "https://{}.api.subscan.io/api/scan/extrinsics",
            self.network
        );

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key).unwrap());

        let payload = format!(
            r#"{{"row": 100, "page": 0, "module": "{module}", "call": "{extrinsics_type}"}}"#,
        );
        let data = serde_json::from_str(&payload).unwrap();

        let resp = self
            .http_client
            .post_request::<Value, Value>(&url, headers, data)
            .await;

        let data = resp.get("data")?.get("extrinsics")?.as_array()?;
        let subscan_operations = data
            .iter()
            .filter_map(|d| {
                if !d.get("success")?.as_bool()? {
                    return None;
                };

                let operation_timestamp =
                    DateTime::from_millis(d.get("block_timestamp")?.as_i64()? * 1_000);
                let from_wallet = d.get("account_id")?.as_str()?.to_string();

                let operation_type = match extrinsics_type {
                    ExtrinsicsType::Bond | ExtrinsicsType::BondExtra | ExtrinsicsType::Rebond => {
                        OperationType::Stake
                    }
                    ExtrinsicsType::Unbond => OperationType::RequestUnstake,
                    ExtrinsicsType::WithdrawUnbonded => OperationType::WithdrawUnstaked,
                };

                let mut exchange_trade = SubscanOperation {
                    hash: String::new(),
                    operation_timestamp,
                    operation_quantity: 0.321,
                    operation_price: 0.123,
                    operation_type,
                    from_wallet,
                    to_wallet: "".to_string(),
                };
                exchange_trade.set_hash();

                Some(exchange_trade)
            })
            .collect();
        Some(subscan_operations)
    }
}
