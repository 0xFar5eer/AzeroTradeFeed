use crate::{ExtrinsicsType, OperationType, SubscanOperation};
use bson::DateTime;
use reqwest::header::{HeaderMap, HeaderValue};
use rs_utils::clients::http_client::HttpClient;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct SubscanParser {
    pub http_client: HttpClient,
    pub api_key: String,
}

impl SubscanParser {
    pub async fn new(api_key: &str) -> Self {
        let http_client = HttpClient::new("subscan_parser").await;
        SubscanParser {
            http_client,
            api_key: api_key.to_string(),
        }
    }

    // TODO: add to_wallet, operation_price, operation_quantity

    pub async fn parse_subscan_operations(
        &mut self,
        extrinsics_type: ExtrinsicsType,
    ) -> Option<Vec<SubscanOperation>> {
        let url = "https://alephzero.api.subscan.io/api/scan/extrinsics";

        let mut headers = HeaderMap::new();
        headers.insert("X-API-Key", HeaderValue::from_str(&self.api_key).unwrap());

        let payload = format!(
            r#"{{"row": 100, "page": 0, "module": "staking", "call": "{}"}}"#,
            extrinsics_type
        );
        let data = serde_json::from_str(&payload).unwrap();

        let resp = self
            .http_client
            .post_request::<Value, Value>(url, headers, data)
            .await;

        let code = resp.get("code")?.as_u64()?;
        if code != 200 {
            return None;
        }

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
