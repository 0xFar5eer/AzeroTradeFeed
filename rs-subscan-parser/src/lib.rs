use bson::DateTime;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

pub mod mongodb_client_subscan;
pub mod subscan_parser;
pub mod subscan_stake_parser;

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
pub enum OperationType {
    #[default]
    Stake,
    RequestUnstake,
    WithdrawUnstaked,
    Transfer,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct SubscanOperation {
    pub hash: String,
    pub block_number: u64,
    pub operation_timestamp: DateTime,
    pub operation_quantity: f64,
    pub operation_usd: f64,
    pub operation_type: OperationType,
    pub from_wallet: String,
    pub to_wallet: String,
}

impl SubscanOperation {
    pub fn set_hash(&mut self) {
        self.hash = sha256::digest(format!(
            "{}_{}_{}_{}_{}_{}",
            self.operation_timestamp,
            self.operation_quantity,
            self.operation_usd,
            self.operation_type,
            self.from_wallet,
            self.to_wallet,
        ));
    }
}

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
pub enum ExtrinsicsType {
    #[default]
    Bond,
    BondExtra,
    Rebond,
    Unbond,
    WithdrawUnbonded,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct SubscanEventParam {
    pub type_name: String,
    pub value: String,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct SubscanEvent {
    pub event_index: String,
    pub event_params: Vec<SubscanEventParam>,
}

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
pub enum Module {
    #[default]
    Staking,
}
