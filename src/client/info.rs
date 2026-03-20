use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

pub struct InfoClient {
    http: reqwest::Client,
    base_url: String,
}

// ── 响应类型 ──

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MetaResponse {
    pub universe: Vec<PerpetualInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerpetualInfo {
    pub perpetual_id: i32,
    pub object_id: String,
    #[serde(default)]
    pub liquidity_tier_id: i32,
    pub atomic_resolution: i32,
    pub coin: String,
    pub sz_decimals: i32,
    pub max_leverage: i32,
    #[serde(default)]
    pub quantum_conversion_exponent: i32,
    #[serde(default = "default_one_i32")]
    pub subticks_per_tick: i32,
    #[serde(default = "default_one_i64")]
    pub step_base_quantums: i64,
    #[serde(default)]
    pub created_at_ms: i64,
}

fn default_one_i32() -> i32 { 1 }
fn default_one_i64() -> i64 { 1 }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L2BookResponse {
    #[serde(default)]
    pub coin: String,
    #[serde(default)]
    pub perpetual_id: i32,
    pub bids: Vec<L2Level>,
    pub asks: Vec<L2Level>,
    #[serde(default)]
    pub timestamp_ms: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct L2Level {
    pub price: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandleResponse {
    #[serde(default)]
    pub coin: String,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    #[serde(default)]
    pub num_trades: i64,
    pub timestamp_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatInfo {
    #[serde(default)]
    pub coin: String,
    pub perpetual_id: i32,
    pub last_price: String,
    pub volume_24h: String,
    pub high_24h: String,
    pub low_24h: String,
    pub num_trades_24h: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FillResponse {
    #[serde(default)]
    pub coin: String,
    #[serde(default)]
    pub tx_digest: String,
    pub perpetual_id: i32,
    pub side: i16,
    pub price: String,
    pub quantity: String,
    #[serde(default)]
    pub taker_fee: String,
    #[serde(default)]
    pub maker_fee: String,
    #[serde(default)]
    pub taker_account: String,
    #[serde(default)]
    pub maker_account: String,
    pub timestamp_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    #[serde(default)]
    pub coin: String,
    pub order_id: String,
    pub client_id: i64,
    pub perpetual_id: i32,
    pub account_address: String,
    pub subaccount_number: i32,
    pub side: i16,
    pub price: String,
    pub quantity: String,
    pub remaining_quantity: String,
    pub order_type: i16,
    pub time_in_force: i16,
    pub reduce_only: bool,
    pub status: i16,
    pub timestamp_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatusResponse {
    pub status: String,
    pub order: Option<OrderResponse>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClearinghouseStateResponse {
    pub margin_summary: MarginSummary,
    #[serde(default)]
    pub cross_margin_summary: Option<MarginSummary>,
    pub withdrawable: String,
    pub asset_positions: Vec<AssetPosition>,
    #[serde(default)]
    pub time: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MarginSummary {
    pub account_value: String,
    pub total_ntl_pos: String,
    pub total_raw_usd: String,
    pub total_margin_used: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetPosition {
    pub position: PositionInfo,
    #[serde(rename = "type")]
    pub position_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PositionInfo {
    pub coin: String,
    pub szi: String,
    pub entry_px: String,
    pub position_value: String,
    pub unrealized_pnl: String,
    #[serde(default)]
    pub cum_funding: Option<serde_json::Value>,
    #[serde(default)]
    pub leverage: Option<serde_json::Value>,
    pub liquidation_px: Option<String>,
    pub margin_used: String,
    pub return_on_equity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BalanceResponse {
    #[serde(default)]
    pub tx_digest: String,
    #[serde(default)]
    pub account_address: String,
    #[serde(default)]
    pub subaccount_number: u8,
    pub delta: String,
    pub new_balance: String,
    pub update_type: i16,
    pub timestamp_ms: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransferResponse {
    #[serde(default)]
    pub tx_digest: String,
    pub from_account: String,
    #[serde(default)]
    pub from_subaccount_number: u8,
    pub to_account: String,
    #[serde(default)]
    pub to_subaccount_number: u8,
    pub amount: String,
    pub timestamp_ms: i64,
}

impl InfoClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    async fn post_info<T: serde::de::DeserializeOwned>(
        &self,
        body: serde_json::Value,
    ) -> Result<T> {
        let url = format!("{}/info", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| format!("Cannot connect to dex-api at {}. Is the service running?", self.base_url))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("dex-api returned HTTP {}: {}", status, text);
        }

        resp.json::<T>()
            .await
            .context("Failed to parse dex-api response")
    }

    pub async fn meta(&self) -> Result<MetaResponse> {
        self.post_info(serde_json::json!({"type": "meta"})).await
    }

    pub async fn l2_book(&self, perpetual_id: i32) -> Result<L2BookResponse> {
        self.post_info(serde_json::json!({
            "type": "l2Book",
            "perpetualId": perpetual_id
        }))
        .await
    }

    pub async fn all_mids(&self) -> Result<serde_json::Value> {
        self.post_info(serde_json::json!({"type": "allMids"})).await
    }

    pub async fn recent_fills(&self, perpetual_id: i32) -> Result<Vec<FillResponse>> {
        self.post_info(serde_json::json!({
            "type": "recentFills",
            "perpetualId": perpetual_id
        }))
        .await
    }

    pub async fn candle_snapshot(
        &self,
        perpetual_id: i32,
        interval: &str,
    ) -> Result<Vec<CandleResponse>> {
        self.post_info(serde_json::json!({
            "type": "candleSnapshot",
            "perpetualId": perpetual_id,
            "interval": interval
        }))
        .await
    }

    pub async fn market_stats(&self, perpetual_id: i32) -> Result<MarketStatInfo> {
        self.post_info(serde_json::json!({
            "type": "marketStats",
            "perpetualId": perpetual_id
        }))
        .await
    }

    pub async fn clearinghouse_state(
        &self,
        user: &str,
        subaccount: u32,
    ) -> Result<ClearinghouseStateResponse> {
        self.post_info(serde_json::json!({
            "type": "clearinghouseState",
            "user": user,
            "subaccountNumber": subaccount
        }))
        .await
    }

    pub async fn open_orders(
        &self,
        user: &str,
        perpetual_id: Option<i32>,
    ) -> Result<Vec<OrderResponse>> {
        let mut body = serde_json::json!({"type": "openOrders", "user": user});
        if let Some(pid) = perpetual_id {
            body["perpetualId"] = serde_json::json!(pid);
        }
        self.post_info(body).await
    }

    pub async fn historical_orders(
        &self,
        user: &str,
        perpetual_id: Option<i32>,
        limit: Option<u32>,
    ) -> Result<Vec<OrderResponse>> {
        let mut body = serde_json::json!({"type": "historicalOrders", "user": user});
        if let Some(pid) = perpetual_id {
            body["perpetualId"] = serde_json::json!(pid);
        }
        if let Some(l) = limit {
            body["limit"] = serde_json::json!(l);
        }
        self.post_info(body).await
    }

    pub async fn order_status(&self, user: &str, oid: &str) -> Result<OrderStatusResponse> {
        self.post_info(serde_json::json!({
            "type": "orderStatus",
            "user": user,
            "oid": oid
        }))
        .await
    }

    pub async fn user_fills(
        &self,
        user: &str,
        perpetual_id: Option<i32>,
        limit: Option<u32>,
    ) -> Result<Vec<FillResponse>> {
        let mut body = serde_json::json!({"type": "userFills", "user": user});
        if let Some(pid) = perpetual_id {
            body["perpetualId"] = serde_json::json!(pid);
        }
        if let Some(l) = limit {
            body["limit"] = serde_json::json!(l);
        }
        self.post_info(body).await
    }

    pub async fn user_balances(&self, user: &str) -> Result<Vec<BalanceResponse>> {
        self.post_info(serde_json::json!({"type": "userBalances", "user": user}))
            .await
    }

    pub async fn user_transfers(&self, user: &str) -> Result<Vec<TransferResponse>> {
        self.post_info(serde_json::json!({"type": "userTransfers", "user": user}))
            .await
    }

    pub async fn health_check(&self) -> Result<()> {
        let _: serde_json::Value = self.post_info(serde_json::json!({"type": "meta"})).await?;
        Ok(())
    }
}
