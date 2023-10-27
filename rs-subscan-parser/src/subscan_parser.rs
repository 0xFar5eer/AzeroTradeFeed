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
        event_indexes: Vec<String>,
    ) -> Option<Vec<SubscanEvent>> {
        let url = format!(
            "https://{}.api.subscan.io/api/scan/event/params",
            self.network
        );

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key).unwrap());

        let payload = json!({"event_index": event_indexes});

        let resp = self
            .http_client
            .post_request::<Value, Value>(&url, headers, payload)
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

    pub async fn parse_subscan_extrinsic_details(
        &mut self,
        extrinsic_index: String,
    ) -> Option<Vec<SubscanEvent>> {
        let url = format!("https://{}.api.subscan.io/api/scan/extrinsic", self.network);

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key).unwrap());

        let payload = json!({
            "extrinsic_index": extrinsic_index,
            "only_extrinsic_event" : true
        });

        let resp = self
            .http_client
            .post_request::<Value, Value>(&url, headers, payload)
            .await;

        let data = resp.get("data")?.get("event")?.as_array()?;

        let subscan_events = data
            .iter()
            .filter_map(|d| -> Option<_> {
                let event_index = d.get("event_index")?.as_str()?.to_string();
                let params: Value = serde_json::from_str(d.get("params")?.as_str()?).ok()?;
                let event_params = params
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
        num_items: u32,
    ) -> Option<Vec<SubscanOperation>> {
        let url = format!(
            "https://{}.api.subscan.io/api/scan/extrinsics",
            self.network
        );

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key).unwrap());

        let payload = json!(
            {"row": num_items, "page": 0, "module": module, "call": extrinsics_type}
        );
        let resp = self
            .http_client
            .post_request::<Value, Value>(&url, headers, payload)
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
                let block_number = d.get("block_num")?.as_u64()?;
                let extrinsic_index = d.get("extrinsic_index")?.as_str()?.to_string();

                let operation_type = match extrinsics_type {
                    ExtrinsicsType::Bond | ExtrinsicsType::BondExtra | ExtrinsicsType::Rebond => {
                        OperationType::Stake
                    }
                    ExtrinsicsType::Unbond => OperationType::RequestUnstake,
                    ExtrinsicsType::WithdrawUnbonded => OperationType::WithdrawUnstaked,
                };

                let subscan_operation = SubscanOperation {
                    hash: String::new(),
                    block_number,
                    operation_timestamp,
                    operation_quantity: 0.321,
                    operation_usd: 0.123,
                    operation_type,
                    from_wallet,
                    to_wallet: "".to_string(),
                    extrinsic_index,
                };

                Some(subscan_operation)
            })
            .collect();
        Some(subscan_operations)
    }
}
