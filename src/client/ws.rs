use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

pub struct WsClient {
    pub url: String,
}

impl WsClient {
    pub fn new(api_url: &str) -> Self {
        let ws_url = api_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        Self {
            url: format!("{}/ws", ws_url.trim_end_matches('/')),
        }
    }

    pub fn validate_url(&self) -> Result<()> {
        url::Url::parse(&self.url).context("Invalid WebSocket URL")?;
        Ok(())
    }

    pub async fn subscribe(
        &self,
        channel: &str,
        format: crate::output::OutputFormat,
    ) -> Result<()> {
        self.validate_url()?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .with_context(|| format!("Cannot connect to WebSocket at {}", self.url))?;

        let (mut write, mut read) = ws_stream.split();

        let sub_msg = serde_json::json!({
            "method": "subscribeChannel",
            "subscription": {
                "channel": channel
            }
        });

        write
            .send(Message::Text(sub_msg.to_string().into()))
            .await
            .context("Failed to send subscription message")?;

        eprintln!("Subscribed to channel: {channel}");
        eprintln!("Press Ctrl-C to stop.\n");

        let ctrl_c = tokio::signal::ctrl_c();
        tokio::pin!(ctrl_c);

        loop {
            tokio::select! {
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            match format {
                                crate::output::OutputFormat::Json => {
                                    println!("{text}");
                                }
                                crate::output::OutputFormat::Table => {
                                    let text_str: String = text.to_string();
                                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text_str) {
                                        println!("{}", serde_json::to_string_pretty(&val).unwrap_or(text_str));
                                    } else {
                                        println!("{text_str}");
                                    }
                                }
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            eprintln!("WebSocket closed by server.");
                            break;
                        }
                        Some(Err(e)) => {
                            anyhow::bail!("WebSocket error: {e}");
                        }
                        None => {
                            eprintln!("WebSocket stream ended.");
                            break;
                        }
                        _ => {}
                    }
                }
                _ = &mut ctrl_c => {
                    eprintln!("\nDisconnected.");
                    break;
                }
            }
        }

        Ok(())
    }
}
