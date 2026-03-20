use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// tx-gateway 交易客户端 (POST /tx/*)
pub struct GatewayClient {
    http: reqwest::Client,
    base_url: String,
}

/// 统一响应格式
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayResponse {
    pub success: bool,
    pub message: String,
    pub digest: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Gateway 状态
#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub address: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

impl GatewayClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// 通用 POST 请求
    async fn post<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: serde_json::Value,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to tx-gateway at {}. Is the service running?",
                    self.base_url
                )
            })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("tx-gateway returned HTTP {}: {}", status, text);
        }

        resp.json::<T>()
            .await
            .context("Failed to parse tx-gateway response")
    }

    /// 通用 GET 请求
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| {
                format!(
                    "Cannot connect to tx-gateway at {}. Is the service running?",
                    self.base_url
                )
            })?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("tx-gateway returned HTTP {}: {}", status, text);
        }

        resp.json::<T>()
            .await
            .context("Failed to parse tx-gateway response")
    }

    // ── 交易 ──

    pub async fn place_order(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/order", body).await
    }

    pub async fn cancel_order(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/cancel", body).await
    }

    pub async fn close_position(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/close-position", body).await
    }

    pub async fn set_leverage(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/set-leverage", body).await
    }

    // ── 资金 ──

    pub async fn deposit(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/deposit", body).await
    }

    pub async fn withdraw(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/withdraw", body).await
    }

    pub async fn mint_usdc(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/mint-usdc", body).await
    }

    pub async fn faucet(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/faucet", body).await
    }

    // ── 管理 ──

    pub async fn setup_perpetual(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/setup", body).await
    }

    pub async fn update_oracle_prices(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/update-oracle-prices", body).await
    }

    pub async fn update_funding_rates(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/update-funding-rates", body).await
    }

    pub async fn liquidate(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/liquidate", body).await
    }

    pub async fn setup_vault(&self, body: serde_json::Value) -> Result<GatewayResponse> {
        self.post("/tx/setup-vault", body).await
    }

    pub async fn update_perpetual_params(
        &self,
        body: serde_json::Value,
    ) -> Result<GatewayResponse> {
        self.post("/tx/update-perpetual-params", body).await
    }

    // ── 状态 ──

    pub async fn status(&self) -> Result<GatewayStatus> {
        self.get("/tx/status").await
    }

    pub async fn addresses(&self, count: u32) -> Result<serde_json::Value> {
        self.get(&format!("/tx/addresses?count={}", count)).await
    }

    pub async fn address(&self) -> Result<serde_json::Value> {
        self.get("/tx/address").await
    }
}
