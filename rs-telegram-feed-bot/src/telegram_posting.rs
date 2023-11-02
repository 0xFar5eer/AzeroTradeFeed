use log::error;
use rs_utils::clients::http_client::HttpClient;
use serde_json::Value;
use std::{collections::HashMap, time::Duration};
use tokio::time::sleep;

#[derive(Clone, Debug)]
pub struct TelegramPosting {
    http_client: HttpClient,
    bot_father_key: String,
    channel_id: String,
}

impl TelegramPosting {
    pub async fn new(bot_father_key: &str, channel_id: &str) -> Self {
        let http_client = HttpClient::new("telegram_posting").await;
        TelegramPosting {
            bot_father_key: bot_father_key.to_string(),
            channel_id: channel_id.to_string(),
            http_client,
        }
    }

    pub async fn post_message(&mut self, message: &str) -> Option<()> {
        let mut resp;

        loop {
            let url = format!(
                "https://api.telegram.org/bot{}/sendMessage",
                self.bot_father_key
            );

            //?chat_id=[MY_CHANNEL_NAME]&text=[MY_MESSAGE_TEXT]
            let params = HashMap::from([
                ("chat_id".to_string(), self.channel_id.to_string()),
                ("text".to_string(), message.to_string()),
                ("parse_mode".to_string(), "markdown".to_string()),
                ("disable_web_page_preview".to_string(), "true".to_string()),
            ]);

            resp = self
                .http_client
                .get_request::<Value>(&url, Some(params))
                .await;

            let is_ok = resp.get("ok")?.as_bool()?;
            if !is_ok {
                let code = resp.get("error_code")?.as_u64()?;
                let message = resp.get("description")?.as_str()?;
                error!(target: "telegram_posting", "Parse error[{code}]: {message}. Sleeping 1 seconds.");
                sleep(Duration::from_millis(1_000)).await;
                continue;
            }

            break;
        }

        Some(())
    }
}
