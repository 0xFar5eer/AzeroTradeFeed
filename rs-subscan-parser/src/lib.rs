use bson::DateTime;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

pub mod mongodb_client_identities;
pub mod mongodb_client_subscan;
pub mod mongodb_client_validator;
pub mod subscan_parser;
pub mod subscan_stake_parser;
pub mod subscan_transfer_parser;

pub static MINIMUM_AZERO_TO_SAVE_TO_DB: f64 = 499.999999;

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
    ReStake,
    RequestUnstake,
    WithdrawUnstaked,
    Transfer,
    DepositToExchange,
    WithdrawFromExchange,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Validator {
    pub nominator: String,
    pub validator: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Identity {
    pub address: String,
    pub identity: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct SubscanOperation {
    pub hash: String,
    pub block_number: u64,
    pub extrinsic_index: String,
    pub operation_timestamp: DateTime,
    pub operation_quantity: f64,
    pub operation_usd: f64,
    pub operation_type: OperationType,
    pub from_wallet: String,
    pub controller_wallet: String,
    pub to_wallet: String,
}

impl SubscanOperation {
    pub fn set_hash(&mut self) {
        self.hash = sha256::digest(format!(
            "{}_{}_{}_{}_{}",
            self.operation_timestamp,
            self.operation_quantity,
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
    #[strum(to_string = "bond_extra")]
    BondExtra,
    Nominate,
    Rebond,
    Unbond,
    #[strum(to_string = "withdraw_unbonded")]
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
    pub module_id: String,
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
