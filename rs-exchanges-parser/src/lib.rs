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

impl Exchanges {
    pub fn get_beautiful_name(&self) -> String {
        match self {
            Exchanges::Mexc => "🚹 Mexc",
            Exchanges::Kucoin => "🦚 Kucoin",
            Exchanges::Gate => "🚪 Gate",
        }
        .to_string()
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
pub enum ExchangesWallets {
    #[default]
    #[strum(to_string = "5H3JuUqCKm28Gz6Z1JpLhRzN3f4UJK1XhktbUQWhFuRJnFvb")]
    Mexc,

    #[strum(to_string = "5DMUZfUht8VaU7ARP77yTcf1jNKm7g9xUrqko6P1WZCHrAyX")]
    Kucoin,

    #[strum(to_string = "123")]
    Gate,
}

impl ExchangesWallets {
    pub fn get_beautiful_name(&self) -> String {
        match self {
            ExchangesWallets::Mexc => Exchanges::Mexc,
            ExchangesWallets::Kucoin => Exchanges::Kucoin,
            ExchangesWallets::Gate => Exchanges::Gate,
        }
        .get_beautiful_name()
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
            "{}_{}_{}_{}_{}_{}",
            self.trade_timestamp,
            self.trade_quantity,
            self.trade_type,
            self.primary_token,
            self.secondary_token,
            self.exchange,
        ));
    }
}
