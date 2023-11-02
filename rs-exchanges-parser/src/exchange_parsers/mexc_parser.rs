use crate::{ExchangeTrade, Exchanges, PrimaryToken, SecondaryToken, TradeType};
use bson::DateTime;
use rs_utils::clients::http_client::HttpClient;
use serde_json::Value;
use std::collections::HashMap;

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
        let params = HashMap::from([(
            "symbol".to_string(),
            format!(
                "{}_{}",
                primary_token.to_string().to_uppercase(),
                secondary_token.to_string().to_uppercase()
            ),
        )]);
        let url = "https://www.mexc.com/open/api/v2/market/deals";
        let resp = self
            .http_client
            .get_request::<Value>(url, Some(params))
            .await;

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

#[cfg(test)]
mod tests {
    use crate::{exchange_parsers::mexc_parser::MexcParser, PrimaryToken, SecondaryToken};
    use chrono::Utc;

    #[tokio::test]
    async fn mexc_azero_usdt_parser_works() {
        let mut mexc_parser = MexcParser::new().await;
        let azero_usdt = mexc_parser
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
    #[tokio::test]
    async fn mexc_azero_usdc_parser_works() {
        let mut mexc_parser = MexcParser::new().await;
        let azero_usdc = mexc_parser
            .parse(PrimaryToken::Azero, SecondaryToken::Usdc)
            .await;

        assert!(azero_usdc.is_some());

        let azero_usdc = azero_usdc.unwrap();
        assert!(!azero_usdc.is_empty());

        let one_day_in_millis = 1000 * 60 * 60 * 24;
        let now_in_millis = Utc::now().timestamp_millis();
        let yesterday_in_millis = now_in_millis - one_day_in_millis;
        let tomorrow_in_millis = now_in_millis + one_day_in_millis;
        let trade_time_millis = azero_usdc
            .first()
            .unwrap()
            .trade_timestamp
            .timestamp_millis();
        assert!(yesterday_in_millis <= trade_time_millis);
        assert!(trade_time_millis < tomorrow_in_millis);
    }
}
