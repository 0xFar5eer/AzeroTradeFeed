use bson::DateTime;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString, IntoStaticStr};

pub mod exchange_parsers;
pub mod mongodb_client_exchanges;

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
pub enum Exchanges {
    #[default]
    Mexc,
    Kucoin,
    Gate,
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
pub enum TradeType {
    #[default]
    IsBuy,
    IsSell,
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
pub enum PrimaryToken {
    #[default]
    Azero,
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
pub enum SecondaryToken {
    #[default]
    Usdt,
    Usdc,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct ExchangeTrade {
    pub hash: String,
    pub trade_timestamp: DateTime,
    pub trade_quantity: f64,
    pub trade_price: f64,
    pub trade_type: TradeType,
    pub primary_token: PrimaryToken,
    pub secondary_token: SecondaryToken,
    pub exchange: Exchanges,
}

impl ExchangeTrade {
    pub fn set_hash(&mut self) {
        self.hash = sha256::digest(format!(
            "{}_{}_{}_{}_{}_{}_{}",
            self.trade_timestamp,
            self.trade_quantity,
            self.trade_price,
            self.trade_type,
            self.primary_token,
            self.secondary_token,
            self.exchange,
        ));
    }
}