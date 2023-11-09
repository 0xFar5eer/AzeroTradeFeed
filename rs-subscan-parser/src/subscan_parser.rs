use crate::{
    ExtrinsicsType, Identity, Module, OperationType, SubscanEvent, SubscanEventParam,
    SubscanOperation,
};
use bson::DateTime;
use log::error;
use rand::seq::IteratorRandom;
use reqwest::header::{HeaderMap, HeaderValue};
use rs_utils::clients::http_client::HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sp_core::crypto::{AccountId32, Ss58AddressFormat, Ss58Codec};
use std::{env, time::Duration};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};
use tokio::time::sleep;

pub static EMPTY_ADDRESS: &str = "0x0";
pub static AZERO_DENOMINATOR: f64 = 1e12;

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
    network: String,
}

impl SubscanParser {
    pub async fn new(network: Network) -> Self {
        let http_client = HttpClient::new("subscan_parser").await;
        SubscanParser {
            network: network.to_string(),
            http_client,
        }
    }

    pub async fn parse_subscan_events(
        &mut self,
        event_indexes: Vec<String>,
    ) -> Option<Vec<SubscanEvent>> {
        let mut resp;

        loop {
            let url = format!(
                "https://{}.api.subscan.io/api/scan/event/params",
                self.network
            );

            let subscan_api_key = SubscanParser::get_random_api_key();

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&subscan_api_key).unwrap(),
            );

            let payload = json!({"event_index": event_indexes});

            resp = self
                .http_client
                .post_request::<Value, Value>(&url, headers, payload)
                .await;

            let code = resp.get("code")?.as_u64()?;
            if code != 0 {
                let message = resp.get("message")?.as_str()?;
                error!(target: "subscan_parser", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
        }

        let data = resp.get("data")?.as_array()?;
        let subscan_events = data
            .iter()
            .filter_map(|d| -> Option<_> {
                let module_id = d.get("module_id")?.as_str()?.to_string();
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
                    module_id,
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
        let mut resp;

        loop {
            let url = format!("https://{}.api.subscan.io/api/scan/extrinsic", self.network);

            let subscan_api_key = SubscanParser::get_random_api_key();

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&subscan_api_key).unwrap(),
            );

            let payload = json!({
                "extrinsic_index": extrinsic_index,
                "only_extrinsic_event" : true
            });

            resp = self
                .http_client
                .post_request::<Value, Value>(&url, headers, payload)
                .await;

            let code = resp.get("code")?.as_u64()?;
            if code != 0 {
                let message = resp.get("message")?.as_str()?;
                error!(target: "subscan_parser", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
        }

        let data = resp.get("data")?.get("event")?.as_array()?;

        let subscan_events = data
            .iter()
            .filter_map(|d| -> Option<_> {
                let module_id = d.get("module_id")?.as_str()?.to_string();
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
                    module_id,
                    event_index,
                    event_params,
                })
            })
            .collect::<Vec<SubscanEvent>>();
        Some(subscan_events)
    }

    pub async fn parse_subscan_operations(
        &mut self,
        address: &str,
        module: Module,
        extrinsics_type: ExtrinsicsType,
        num_items: u32,
    ) -> Option<Vec<SubscanOperation>> {
        let mut resp;

        loop {
            let url = format!(
                "https://{}.api.subscan.io/api/scan/extrinsics",
                self.network
            );

            let subscan_api_key = SubscanParser::get_random_api_key();

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&subscan_api_key).unwrap(),
            );

            let payload = json!(
                {"address": address, "row": num_items, "page": 0, "module": module, "call": extrinsics_type.to_string(), "success": true}
            );
            resp = self
                .http_client
                .post_request::<Value, Value>(&url, headers, payload)
                .await;

            let code = resp.get("code")?.as_u64()?;
            if code != 0 {
                let message = resp.get("message")?.as_str()?;
                error!(target: "subscan_parser", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
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
                let block_number = d.get("block_num")?.as_u64()?;
                let extrinsic_index = d.get("extrinsic_index")?.as_str()?.to_string();

                let operation_type = match extrinsics_type {
                    ExtrinsicsType::Bond | ExtrinsicsType::BondExtra | ExtrinsicsType::Rebond => {
                        OperationType::Stake
                    }
                    ExtrinsicsType::Nominate => OperationType::ReStake,
                    ExtrinsicsType::Unbond => OperationType::RequestUnstake,
                    ExtrinsicsType::WithdrawUnbonded => OperationType::WithdrawUnstaked,
                };

                let to_wallet = if extrinsics_type == ExtrinsicsType::Nominate {
                    let params: Value = serde_json::from_str(d.get("params")?.as_str()?).ok()?;

                    let addr = params
                        .as_array()?
                        .first()?
                        .get("value")?
                        .as_array()?
                        .first()?
                        .get("Id")?
                        .as_str()?;

                    let addr = addr[2..].to_string();
                    let decoded = hex::decode(addr).ok()?;
                    let byte_arr: [u8; 32] = decoded.try_into().ok()?;
                    AccountId32::from(byte_arr)
                        .to_ss58check_with_version(Ss58AddressFormat::custom(42))
                } else {
                    EMPTY_ADDRESS.to_string()
                };

                let controller_wallet = if extrinsics_type == ExtrinsicsType::Bond {
                    let params: Value = serde_json::from_str(d.get("params")?.as_str()?).ok()?;

                    let addr = params
                        .as_array()?
                        .iter()
                        .find(|p| p.get("name").unwrap().as_str().unwrap() == "controller")?
                        .get("value")?
                        .get("Id")?
                        .as_str()?;

                    let addr = addr[2..].to_string();
                    let decoded = hex::decode(addr).ok()?;
                    let byte_arr: [u8; 32] = decoded.try_into().ok()?;
                    AccountId32::from(byte_arr)
                        .to_ss58check_with_version(Ss58AddressFormat::custom(42))
                } else {
                    EMPTY_ADDRESS.to_string()
                };

                let subscan_operation = SubscanOperation {
                    hash: String::new(),
                    block_number,
                    operation_timestamp,
                    operation_quantity: 0.321,
                    operation_usd: 0.123,
                    operation_type,
                    from_wallet,
                    to_wallet,
                    controller_wallet,
                    extrinsic_index,
                };

                Some(subscan_operation)
            })
            .rev()
            .collect();
        Some(subscan_operations)
    }

    pub async fn parse_subscan_batch_all(
        &mut self,
        address: &str,
        page: u32,
        num_items: u32,
    ) -> Option<Vec<SubscanOperation>> {
        let mut resp;

        loop {
            let url = format!(
                "https://{}.api.subscan.io/api/scan/extrinsics",
                self.network
            );

            let subscan_api_key = SubscanParser::get_random_api_key();

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&subscan_api_key).unwrap(),
            );

            let payload = json!(
                {"address": address, "row": num_items, "page": page, "module": "utility", "call": "batch_all", "success": true}
            );
            resp = self
                .http_client
                .post_request::<Value, Value>(&url, headers, payload)
                .await;

            let code = resp.get("code")?.as_u64()?;
            if code != 0 {
                let message = resp.get("message")?.as_str()?;
                error!(target: "subscan_parser", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
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
                let block_number = d.get("block_num")?.as_u64()?;
                let extrinsic_index = d.get("extrinsic_index")?.as_str()?.to_string();

                let params: Value = serde_json::from_str(d.get("params")?.as_str()?).ok()?;
                let value = params.as_array()?.first()?.get("value")?.as_array()?;
                let bond_extra = value
                    .iter()
                    .find(|p| p.get("call_name").unwrap() == "bond_extra");
                let bond = value.iter().find(|p| p.get("call_name").unwrap() == "bond");
                let unbond = value
                    .iter()
                    .find(|p| p.get("call_name").unwrap() == "unbond");
                let nominate = value
                    .iter()
                    .find(|p| p.get("call_name").unwrap() == "nominate");

                let bond_amount = if bond.is_some() {
                    str::parse::<f64>(
                        bond.unwrap()
                            .get("params")?
                            .as_array()?
                            .iter()
                            .find(|p| p.get("name").unwrap() == "value")?
                            .get("value")?
                            .as_str()?,
                    )
                    .ok()?
                        / AZERO_DENOMINATOR
                } else {
                    0.0
                };

                let bond_extra_amount = if bond_extra.is_some() {
                    str::parse::<f64>(
                        bond_extra
                            .unwrap()
                            .get("params")?
                            .as_array()?
                            .iter()
                            .find(|p| p.get("name").unwrap() == "max_additional")?
                            .get("value")?
                            .as_str()?,
                    )
                    .ok()?
                        / AZERO_DENOMINATOR
                } else {
                    0.0
                };

                let unbond_amount = if unbond.is_some() {
                    str::parse::<f64>(
                        unbond
                            .unwrap()
                            .get("params")?
                            .as_array()?
                            .iter()
                            .find(|p| p.get("name").unwrap() == "value")?
                            .get("value")?
                            .as_str()?,
                    )
                    .ok()?
                        / AZERO_DENOMINATOR
                } else {
                    0.0
                };

                let operation_quantity = bond_amount + bond_extra_amount + unbond_amount;

                let to_wallet = if nominate.is_some() {
                    let addr = nominate
                        .unwrap()
                        .get("params")?
                        .as_array()?
                        .first()?
                        .get("value")?
                        .as_array()?
                        .first()?
                        .get("Id")?
                        .as_str()?;

                    let addr = addr[2..].to_string();
                    let decoded = hex::decode(addr).ok()?;
                    let byte_arr: [u8; 32] = decoded.try_into().ok()?;
                    AccountId32::from(byte_arr)
                        .to_ss58check_with_version(Ss58AddressFormat::custom(42))
                } else {
                    EMPTY_ADDRESS.to_string()
                };

                let controller_wallet = if bond.is_some() {
                    let params = bond.unwrap().get("params")?;

                    let addr = params
                        .as_array()?
                        .iter()
                        .find(|p| p.get("name").unwrap().as_str().unwrap() == "controller")?
                        .get("value")?
                        .get("Id")?
                        .as_str()?;

                    let addr = addr[2..].to_string();
                    let decoded = hex::decode(addr).ok()?;
                    let byte_arr: [u8; 32] = decoded.try_into().ok()?;
                    AccountId32::from(byte_arr)
                        .to_ss58check_with_version(Ss58AddressFormat::custom(42))
                } else {
                    EMPTY_ADDRESS.to_string()
                };

                let operation_type = if unbond_amount > 1e-12 {
                    OperationType::RequestUnstake
                } else if to_wallet != EMPTY_ADDRESS {
                    OperationType::ReStake
                } else {
                    OperationType::Stake
                };

                let subscan_operation = SubscanOperation {
                    hash: String::new(),
                    block_number,
                    operation_timestamp,
                    operation_quantity,
                    operation_usd: 0.123,
                    operation_type,
                    from_wallet,
                    to_wallet,
                    controller_wallet,
                    extrinsic_index,
                };

                Some(subscan_operation)
            })
            .rev()
            .collect();

        Some(subscan_operations)
    }

    pub async fn parse_subscan_identity(
        &mut self,
        address: &str,
        page: u32,
        num_items: u32,
    ) -> Option<Vec<Identity>> {
        if SubscanParser::is_address_empty(address) {
            return None;
        }

        let mut resp;

        loop {
            let url = format!(
                "https://{}.api.subscan.io/api/scan/extrinsics",
                self.network
            );

            let subscan_api_key = SubscanParser::get_random_api_key();

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&subscan_api_key).unwrap(),
            );

            let payload = json!(
                {"address": address, "row": num_items, "page": page, "module": "identity", "call": "set_identity", "success": true}
            );
            resp = self
                .http_client
                .post_request::<Value, Value>(&url, headers, payload)
                .await;

            let code = resp.get("code")?.as_u64()?;
            if code != 0 {
                let message = resp.get("message")?.as_str()?;
                error!(target: "subscan_parser", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
        }

        let data = resp.get("data")?.get("extrinsics")?.as_array()?;
        let identities = data
            .iter()
            .filter_map(|d| {
                if !d.get("success")?.as_bool()? {
                    return None;
                };

                let address = d
                    .get("account_display")?
                    .get("address")?
                    .as_str()?
                    .to_string();
                let identity = d
                    .get("account_display")?
                    .get("display")?
                    .as_str()?
                    .to_string();
                let status = d.get("account_display")?.get("identity")?.as_bool()?;
                if !status {
                    return None;
                }

                Some(Identity { address, identity })
            })
            .rev()
            .collect::<Vec<_>>();

        Some(identities)
    }

    pub async fn parse_subscan_transfers(
        &mut self,
        page: u32,
        num_items: u32,
    ) -> Option<(Vec<SubscanOperation>, Vec<Identity>)> {
        let mut resp;

        loop {
            let url = format!("https://{}.api.subscan.io/api/scan/transfers", self.network);

            let subscan_api_key = SubscanParser::get_random_api_key();

            let mut headers = HeaderMap::new();
            headers.insert(
                "X-API-Key",
                HeaderValue::from_str(&subscan_api_key).unwrap(),
            );

            let payload = json!(
                {
                    "row": num_items,
                    "page": page,
                    "success": true,
                    "asset_symbol": "AZERO",
                }
            );
            resp = self
                .http_client
                .post_request::<Value, Value>(&url, headers, payload)
                .await;

            let code = resp.get("code")?.as_u64()?;
            if code != 0 {
                let message = resp.get("message")?.as_str()?;
                error!(target: "subscan_parser", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
        }

        let data = resp.get("data")?.get("transfers")?.as_array()?;
        let subscan_operations = data
            .iter()
            .filter_map(|d| {
                if !d.get("success")?.as_bool()? {
                    return None;
                };

                let operation_timestamp =
                    DateTime::from_millis(d.get("block_timestamp")?.as_i64()? * 1_000);
                let from_wallet = d.get("from")?.as_str()?.to_string();
                let to_wallet = d.get("to")?.as_str()?.to_string();
                let block_number = d.get("block_num")?.as_u64()?;
                let extrinsic_index = d.get("extrinsic_index")?.as_str()?.to_string();
                let operation_quantity = str::parse::<f64>(d.get("amount")?.as_str()?).ok()?;

                let operation_type = OperationType::Transfer;

                let controller_wallet = EMPTY_ADDRESS.to_string();

                let subscan_operation = SubscanOperation {
                    hash: String::new(),
                    block_number,
                    operation_timestamp,
                    operation_quantity,
                    operation_usd: 0.123,
                    operation_type,
                    from_wallet,
                    to_wallet,
                    controller_wallet,
                    extrinsic_index,
                };

                Some(subscan_operation)
            })
            .rev()
            .collect();

        let identities = data
            .iter()
            .filter_map(|d| {
                if !d.get("success")?.as_bool()? {
                    return None;
                };

                let from_address = d.get("from")?.as_str()?.to_string();
                let from_identity = d
                    .get("from_account_display")?
                    .get("display")
                    .and_then(|v| v.as_str())
                    .map(|v| Identity {
                        address: from_address,
                        identity: v.to_string(),
                    });

                let to_address = d.get("to")?.as_str()?.to_string();
                let to_identity = d
                    .get("to_account_display")?
                    .get("display")
                    .and_then(|v| v.as_str())
                    .map(|v| Identity {
                        address: to_address,
                        identity: v.to_string(),
                    });

                let identities = vec![from_identity, to_identity]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>();
                if identities.is_empty() {
                    return None;
                }

                Some(identities)
            })
            .rev()
            .flatten()
            .collect::<Vec<_>>();

        Some((subscan_operations, identities))
    }

    fn get_random_api_key() -> String {
        env::var("SUBSCAN_API_KEY")
            .unwrap()
            .split(',')
            .choose(&mut rand::thread_rng())
            .unwrap()
            .to_string()
    }

    pub fn is_address_empty(addr: &str) -> bool {
        addr == EMPTY_ADDRESS || addr.is_empty()
    }
}
