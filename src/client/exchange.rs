//! EIP-712 signed exchange client — POST /exchange
//!
//! 仿照 dex-ui 前端签名流程：
//! 1. 构造 EIP-712 typed data (domain + type hash + struct hash)
//! 2. 用 secp256k1 私钥签名
//! 3. 发送到 dex-api POST /exchange

use alloy::primitives::{keccak256, B256};
use alloy::signers::local::PrivateKeySigner;
use alloy::signers::SignerSync;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// EIP-712 domain: Hermes-Dex v1, chainId=1, verifyingContract=0x0
const DOMAIN_NAME: &str = "Hermes-Dex";
const DOMAIN_VERSION: &str = "1";
const CHAIN_ID: u64 = 1;

/// EIP-712 exchange client
pub struct ExchangeClient {
    http: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeResponse {
    pub status: String,
    #[serde(default)]
    pub response: Option<serde_json::Value>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExchangeRequest {
    action: serde_json::Value,
    nonce: u64,
    deadline: u64,
    signature: SignatureValue,
    #[serde(rename = "vaultAddress", skip_serializing_if = "Option::is_none")]
    vault_address: Option<String>,
}

#[derive(Debug, Serialize)]
struct SignatureValue {
    r: String,
    s: String,
    v: u8,
}

impl ExchangeClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// 下单
    pub async fn place_order(
        &self,
        signer: &PrivateKeySigner,
        perpetual_id: u32,
        is_buy: bool,
        quantums: u64,
        subticks: u64,
        time_in_force: u8,
        reduce_only: bool,
        subaccount_number: u32,
        client_id: u32,
        worst_price: u64,
    ) -> Result<ExchangeResponse> {
        let nonce = current_nonce_ms();
        let deadline = nonce + 3_600_000; // +1 hour

        let struct_hash = hash_place_order(
            subaccount_number,
            client_id,
            perpetual_id,
            is_buy,
            quantums,
            subticks,
            time_in_force,
            0,     // goodTilBlockTime
            reduce_only,
            0,     // conditionType
            0,     // triggerSubticks
            worst_price,
            nonce,
            deadline,
        );

        let signing_hash = eip712_signing_hash(struct_hash);
        let sig = sign_hash(signer, signing_hash)?;

        let action = serde_json::json!({
            "type": "order",
            "orders": [{
                "a": perpetual_id,
                "b": is_buy,
                "p": subticks.to_string(),
                "s": quantums.to_string(),
                "r": reduce_only,
                "t": { "limit": { "tif": tif_to_str(time_in_force) } },
                "subaccountNumber": subaccount_number,
                "worstPrice": worst_price.to_string(),
            }],
            "grouping": "na"
        });

        self.post_exchange(action, nonce, deadline, sig).await
    }

    /// 撤单
    pub async fn cancel_order(
        &self,
        signer: &PrivateKeySigner,
        perpetual_id: u32,
        client_id: u32,
        subaccount_number: u32,
    ) -> Result<ExchangeResponse> {
        let nonce = current_nonce_ms();
        let deadline = nonce + 300_000; // +5 minutes

        let struct_hash = hash_cancel_order(
            subaccount_number,
            client_id,
            perpetual_id,
            nonce,
            deadline,
        );

        let signing_hash = eip712_signing_hash(struct_hash);
        let sig = sign_hash(signer, signing_hash)?;

        let action = serde_json::json!({
            "type": "cancel",
            "cancels": [{
                "a": perpetual_id,
                "o": client_id,
                "subaccountNumber": subaccount_number,
            }]
        });

        self.post_exchange(action, nonce, deadline, sig).await
    }

    /// 平仓
    pub async fn close_position(
        &self,
        signer: &PrivateKeySigner,
        perpetual_id: u32,
        protection_price: &str,
        subaccount_number: u32,
    ) -> Result<ExchangeResponse> {
        let nonce = current_nonce_ms();
        let deadline = nonce + 3_600_000;

        // closePosition 复用 PlaceOrder 类型签名（size=0 表示全平）
        let struct_hash = hash_place_order(
            subaccount_number,
            0, // client_id
            perpetual_id,
            true,  // is_buy (方向由链上判断)
            0,     // quantums=0 表示全平
            0,     // subticks
            1,     // IOC
            0,
            true,  // reduce_only
            0,
            0,
            0,
            nonce,
            deadline,
        );

        let signing_hash = eip712_signing_hash(struct_hash);
        let sig = sign_hash(signer, signing_hash)?;

        let action = serde_json::json!({
            "type": "closePosition",
            "a": perpetual_id,
            "p": protection_price,
            "subaccountNumber": subaccount_number,
        });

        self.post_exchange(action, nonce, deadline, sig).await
    }

    /// 调整杠杆
    pub async fn update_leverage(
        &self,
        signer: &PrivateKeySigner,
        perpetual_id: u32,
        is_cross: bool,
        leverage: u32,
        subaccount_number: u32,
    ) -> Result<ExchangeResponse> {
        let nonce = current_nonce_ms();
        let deadline = nonce + 300_000;

        let struct_hash = hash_update_leverage(
            subaccount_number,
            perpetual_id,
            is_cross,
            leverage,
            nonce,
            deadline,
        );

        let signing_hash = eip712_signing_hash(struct_hash);
        let sig = sign_hash(signer, signing_hash)?;

        let action = serde_json::json!({
            "type": "updateLeverage",
            "asset": perpetual_id,
            "isCross": is_cross,
            "leverage": leverage,
        });

        self.post_exchange(action, nonce, deadline, sig).await
    }

    /// 发送签名请求到 POST /exchange
    async fn post_exchange(
        &self,
        action: serde_json::Value,
        nonce: u64,
        deadline: u64,
        sig: SignatureValue,
    ) -> Result<ExchangeResponse> {
        let req = ExchangeRequest {
            action,
            nonce,
            deadline,
            signature: sig,
            vault_address: None,
        };

        let url = format!("{}/exchange", self.base_url);
        let resp = self
            .http
            .post(&url)
            .json(&req)
            .send()
            .await
            .with_context(|| format!("Cannot connect to dex-api at {}", self.base_url))?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Exchange API returned HTTP {}: {}", status, text);
        }

        resp.json::<ExchangeResponse>()
            .await
            .context("Failed to parse exchange response")
    }
}

// ============================================================================
// EIP-712 hashing
// ============================================================================

/// Domain separator (cached at compile time is not possible, compute once)
fn domain_separator() -> B256 {
    let type_hash = keccak256(
        b"EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)",
    );
    let name_hash = keccak256(DOMAIN_NAME.as_bytes());
    let version_hash = keccak256(DOMAIN_VERSION.as_bytes());

    let mut buf = Vec::with_capacity(5 * 32);
    buf.extend_from_slice(type_hash.as_slice());
    buf.extend_from_slice(name_hash.as_slice());
    buf.extend_from_slice(version_hash.as_slice());
    buf.extend_from_slice(&abi_encode_u256(CHAIN_ID as u128));
    buf.extend_from_slice(&abi_encode_address(&[0u8; 20]));

    keccak256(&buf)
}

/// EIP-712 final signing hash: keccak256(0x19 0x01 || domainSeparator || structHash)
fn eip712_signing_hash(struct_hash: B256) -> B256 {
    let domain = domain_separator();
    let mut buf = Vec::with_capacity(2 + 32 + 32);
    buf.push(0x19);
    buf.push(0x01);
    buf.extend_from_slice(domain.as_slice());
    buf.extend_from_slice(struct_hash.as_slice());
    keccak256(&buf)
}

fn hash_place_order(
    subaccount_number: u32,
    client_id: u32,
    perpetual_id: u32,
    is_buy: bool,
    quantums: u64,
    subticks: u64,
    time_in_force: u8,
    good_til_block_time: u64,
    reduce_only: bool,
    condition_type: u8,
    trigger_subticks: u64,
    worst_price: u64,
    nonce: u64,
    deadline: u64,
) -> B256 {
    let type_hash = keccak256(
        b"PlaceOrder(uint32 subaccountNumber,uint32 clientId,uint32 perpetualId,bool isBuy,uint64 quantums,uint64 subticks,uint8 timeInForce,uint64 goodTilBlockTime,bool reduceOnly,uint8 conditionType,uint64 triggerSubticks,uint64 worstPrice,uint64 nonce,uint64 deadline)",
    );

    let mut buf = Vec::with_capacity(15 * 32);
    buf.extend_from_slice(type_hash.as_slice());
    buf.extend_from_slice(&abi_encode_u32(subaccount_number));
    buf.extend_from_slice(&abi_encode_u32(client_id));
    buf.extend_from_slice(&abi_encode_u32(perpetual_id));
    buf.extend_from_slice(&abi_encode_bool(is_buy));
    buf.extend_from_slice(&abi_encode_u64(quantums));
    buf.extend_from_slice(&abi_encode_u64(subticks));
    buf.extend_from_slice(&abi_encode_u8(time_in_force));
    buf.extend_from_slice(&abi_encode_u64(good_til_block_time));
    buf.extend_from_slice(&abi_encode_bool(reduce_only));
    buf.extend_from_slice(&abi_encode_u8(condition_type));
    buf.extend_from_slice(&abi_encode_u64(trigger_subticks));
    buf.extend_from_slice(&abi_encode_u64(worst_price));
    buf.extend_from_slice(&abi_encode_u64(nonce));
    buf.extend_from_slice(&abi_encode_u64(deadline));

    keccak256(&buf)
}

fn hash_cancel_order(
    subaccount_number: u32,
    client_id: u32,
    perpetual_id: u32,
    nonce: u64,
    deadline: u64,
) -> B256 {
    let type_hash = keccak256(
        b"CancelOrder(uint32 subaccountNumber,uint32 clientId,uint32 perpetualId,uint64 nonce,uint64 deadline)",
    );

    let mut buf = Vec::with_capacity(6 * 32);
    buf.extend_from_slice(type_hash.as_slice());
    buf.extend_from_slice(&abi_encode_u32(subaccount_number));
    buf.extend_from_slice(&abi_encode_u32(client_id));
    buf.extend_from_slice(&abi_encode_u32(perpetual_id));
    buf.extend_from_slice(&abi_encode_u64(nonce));
    buf.extend_from_slice(&abi_encode_u64(deadline));

    keccak256(&buf)
}

fn hash_update_leverage(
    subaccount_number: u32,
    perpetual_id: u32,
    is_cross: bool,
    leverage: u32,
    nonce: u64,
    deadline: u64,
) -> B256 {
    let type_hash = keccak256(
        b"UpdateLeverage(uint32 subaccountNumber,uint32 perpetualId,bool isCross,uint32 leverage,uint64 nonce,uint64 deadline)",
    );

    let mut buf = Vec::with_capacity(7 * 32);
    buf.extend_from_slice(type_hash.as_slice());
    buf.extend_from_slice(&abi_encode_u32(subaccount_number));
    buf.extend_from_slice(&abi_encode_u32(perpetual_id));
    buf.extend_from_slice(&abi_encode_bool(is_cross));
    buf.extend_from_slice(&abi_encode_u32(leverage));
    buf.extend_from_slice(&abi_encode_u64(nonce));
    buf.extend_from_slice(&abi_encode_u64(deadline));

    keccak256(&buf)
}

// ============================================================================
// ABI encoding helpers (Solidity ABI standard: right-aligned in 32-byte slots)
// ============================================================================

fn abi_encode_u8(v: u8) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[31] = v;
    buf
}

fn abi_encode_u32(v: u32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[28..32].copy_from_slice(&v.to_be_bytes());
    buf
}

fn abi_encode_u64(v: u64) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[24..32].copy_from_slice(&v.to_be_bytes());
    buf
}

fn abi_encode_u256(v: u128) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[16..32].copy_from_slice(&v.to_be_bytes());
    buf
}

fn abi_encode_bool(v: bool) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[31] = if v { 1 } else { 0 };
    buf
}

fn abi_encode_address(addr: &[u8; 20]) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[12..32].copy_from_slice(addr);
    buf
}

// ============================================================================
// Signing
// ============================================================================

fn sign_hash(signer: &PrivateKeySigner, hash: B256) -> Result<SignatureValue> {
    let sig = signer
        .sign_hash_sync(&hash)
        .context("Failed to sign EIP-712 hash")?;

    let r_bytes: [u8; 32] = sig.r().to_be_bytes();
    let s_bytes: [u8; 32] = sig.s().to_be_bytes();
    let v = if sig.v() { 28u8 } else { 27u8 };

    Ok(SignatureValue {
        r: format!("0x{}", hex::encode(r_bytes)),
        s: format!("0x{}", hex::encode(s_bytes)),
        v,
    })
}

fn current_nonce_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn tif_to_str(tif: u8) -> &'static str {
    match tif {
        0 => "Gtc",
        1 => "Ioc",
        2 => "Alo",
        3 => "Fok",
        _ => "Gtc",
    }
}
