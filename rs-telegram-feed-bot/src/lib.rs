use serde::{Deserialize, Serialize};

pub mod mongodb_client_telegram;
pub mod telegram_posting;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Telegram {
    pub already_posted_hash: String,
}
