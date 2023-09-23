use crate::{ExchangeTrade, Exchanges, PrimaryToken, SecondaryToken, TradeType};
use bson::DateTime;
use rs_utils::clients::http_client::HttpClient;
use serde_json::Value;

#[derive(Clone, Debug)]
pub struct MexcParser {
    pub http_client: HttpClient,
}

impl MexcParser {
    pub async fn new() -> Self {
        let http_client = HttpClient::new(&Exchanges::Mexc.to_string()).await;
        MexcParser { http_client }
    }

    pub async fn parse(
        &mut self,
        primary_token: PrimaryToken,
        secondary_token: SecondaryToken,
    ) -> Option<Vec<ExchangeTrade>> {
        let url = format!(
            "https://www.mexc.com/open/api/v2/market/deals?symbol={}_{}",
            primary_token.to_string().to_lowercase(),
            secondary_token.to_string().to_lowercase()
        );
        let resp = self.http_client.get_request::<Value>(&url).await;

        let code = resp.get("code")?.as_u64()?;
        if code != 200 {
            return None;
        }

        let data = resp.get("data")?.as_array()?;
        let exchange_trades = data
            .iter()
            .filter_map(|d| {
                let trade_type = if d.get("trade_type")?.as_str()? == "BID" {
                    TradeType::IsBuy
                } else {
                    TradeType::IsSell
                };

                let trade_timestamp = DateTime::from_millis(d.get("trade_time")?.as_i64()?);
                let trade_quantity: f64 = d.get("trade_quantity")?.as_str()?.parse().ok()?;
                let trade_price: f64 = d.get("trade_price")?.as_str()?.parse().ok()?;
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
                    exchange: Exchanges::Mexc,
                };
                exchange_trade.set_hash();

                Some(exchange_trade)
            })
            .collect();
        Some(exchange_trades)
    }
}
