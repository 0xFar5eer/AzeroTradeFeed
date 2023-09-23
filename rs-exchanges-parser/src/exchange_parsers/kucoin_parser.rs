use crate::{ExchangeTrade, Exchanges, PrimaryToken, SecondaryToken, TradeType};
use bson::DateTime;
use rs_utils::clients::http_client::HttpClient;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct KucoinParser {
    pub http_client: HttpClient,
}

impl KucoinParser {
    pub async fn new() -> Self {
        let http_client = HttpClient::new(&Exchanges::Kucoin.to_string()).await;
        KucoinParser { http_client }
    }

    pub async fn parse(
        &mut self,
        primary_token: PrimaryToken,
        secondary_token: SecondaryToken,
    ) -> Option<Vec<ExchangeTrade>> {
        let url = format!(
            "https://api.kucoin.com/api/v1/market/histories?symbol={}-{}",
            primary_token.to_string().to_uppercase(),
            secondary_token.to_string().to_uppercase()
        );
        let resp = self.http_client.get_request::<Value>(&url).await;

        let code = resp.get("code")?.as_str()?;
        if code != "200000" {
            return None;
        }

        let data = resp.get("data")?.as_array()?;
        let exchange_trades = data
            .iter()
            .filter_map(|d| {
                let trade_type = if d.get("side")?.as_str()? == "buy" {
                    TradeType::IsBuy
                } else {
                    TradeType::IsSell
                };

                let time = (d.get("time")?.as_u64()? / 1000) as i64;
                let trade_timestamp = DateTime::from_millis(time);
                let trade_quantity: f64 = d.get("size")?.as_str()?.parse().ok()?;
                let trade_price: f64 = d.get("price")?.as_str()?.parse().ok()?;
                let primary_token = primary_token.clone();
                let secondary_token = secondary_token.clone();

                let mut exchange_trade = ExchangeTrade {
                    hash: String::new(),
                    trade_timestamp,
                    trade_quantity,
                    trade_price,
                    trade_type,
                    primary_token,
                    secondary_token,
                    exchange: Exchanges::Kucoin,
                };
                exchange_trade.set_hash();

                Some(exchange_trade)
            })
            .collect();
        Some(exchange_trades)
    }
}
