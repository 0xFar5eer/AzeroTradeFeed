use crate::{ExchangeTrade, Exchanges, PrimaryToken, SecondaryToken, TradeType};
use bson::DateTime;
use rs_utils::clients::http_client::HttpClient;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct GateParser {
    pub http_client: HttpClient,
}

impl GateParser {
    pub async fn new() -> Self {
        let http_client = HttpClient::new(&Exchanges::Gate.to_string()).await;
        GateParser { http_client }
    }

    pub async fn parse(
        &mut self,
        primary_token: PrimaryToken,
        secondary_token: SecondaryToken,
    ) -> Option<Vec<ExchangeTrade>> {
        let params = HashMap::from([(
            "currency_pair".to_string(),
            format!(
                "{}_{}",
                primary_token.to_string().to_uppercase(),
                secondary_token.to_string().to_uppercase()
            ),
        )]);
        let url = "https://api.gateio.ws/api/v4/spot/trades";
        let resp = self
            .http_client
            .get_request::<Value>(url, Some(params))
            .await;
        let data = resp.as_array()?;

        if data.is_empty() {
            return None;
        }

        let exchange_trades = data
            .iter()
            .filter_map(|d| {
                let trade_type = if d.get("side")?.as_str()? == "buy" {
                    TradeType::IsBuy
                } else {
                    TradeType::IsSell
                };

                let time = d
                    .get("create_time_ms")?
                    .as_str()?
                    .parse::<f64>()
                    .ok()?
                    .round() as i64;
                let trade_timestamp = DateTime::from_millis(time);
                let trade_quantity: f64 = d.get("amount")?.as_str()?.parse().ok()?;
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
                    exchange: Exchanges::Gate,
                };
                exchange_trade.set_hash();

                Some(exchange_trade)
            })
            .collect();
        Some(exchange_trades)
    }
}

#[cfg(test)]
mod tests {
    use crate::{exchange_parsers::gate_parser::GateParser, PrimaryToken, SecondaryToken};
    use chrono::Utc;

    #[tokio::test]
    async fn gate_azero_usdt_parser_works() {
        let mut gate_parser = GateParser::new().await;
        let azero_usdt = gate_parser
            .parse(PrimaryToken::Azero, SecondaryToken::Usdt)
            .await;

        assert!(azero_usdt.is_some());

        let azero_usdt = azero_usdt.unwrap();
        assert!(!azero_usdt.is_empty());

        let one_day_in_millis = 1000 * 60 * 60 * 24;
        let now_in_millis = Utc::now().timestamp_millis();
        let yesterday_in_millis = now_in_millis - one_day_in_millis;
        let tomorrow_in_millis = now_in_millis + one_day_in_millis;
        let trade_time_millis = azero_usdt
            .first()
            .unwrap()
            .trade_timestamp
            .timestamp_millis();
        assert!(yesterday_in_millis <= trade_time_millis);
        assert!(trade_time_millis < tomorrow_in_millis);
    }
}
