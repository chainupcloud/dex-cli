use alloy::primitives::Address;
use alloy::signers::local::PrivateKeySigner;
use anyhow::{Context, Result};
use blake2::Digest;

use crate::client::gateway::GatewayClient;
use crate::config;

/// Identity mode
#[derive(Debug, Clone)]
pub enum Identity {
    /// Use tx-gateway deterministic key (dev/test)
    SenderIndex(u32),
    /// Use own secp256k1 private key (EIP-712 signing)
    PrivateKey(String),
    /// No identity configured
    None,
}

impl Identity {
    pub fn is_some(&self) -> bool {
        !matches!(self, Identity::None)
    }

    pub fn sender_index(&self) -> Option<u32> {
        match self {
            Identity::SenderIndex(i) => Some(*i),
            _ => None,
        }
    }

    pub fn private_key(&self) -> Option<&str> {
        match self {
            Identity::PrivateKey(k) => Some(k),
            _ => None,
        }
    }
}

/// Resolve identity from CLI flags → env vars → config file
pub fn resolve_identity(
    sender_index: Option<u32>,
    private_key: Option<&str>,
) -> Identity {
    if let Some(idx) = sender_index {
        return Identity::SenderIndex(idx);
    }
    if let Some(key) = private_key {
        return Identity::PrivateKey(key.to_string());
    }
    if let Ok(cfg) = config::load_config() {
        if let Some(idx) = cfg.sender_index {
            return Identity::SenderIndex(idx);
        }
        if let Some(key) = cfg.private_key {
            return Identity::PrivateKey(key);
        }
    }
    Identity::None
}

pub fn require_identity(identity: &Identity) -> Result<()> {
    if !identity.is_some() {
        anyhow::bail!(
            "No wallet configured. Run 'dex wallet create' or 'dex setup' first.\n\
             You can also use '--sender-index <N>' for dev/test."
        );
    }
    Ok(())
}

/// Create a LocalSigner from a hex private key string
pub fn create_signer(private_key: &str) -> Result<PrivateKeySigner> {
    let key = private_key.strip_prefix("0x").unwrap_or(private_key);
    let bytes = hex::decode(key).context("Invalid hex private key")?;
    anyhow::ensure!(bytes.len() == 32, "Private key must be 32 bytes");
    PrivateKeySigner::from_slice(&bytes).context("Invalid secp256k1 private key")
}

/// Derive Ethereum address from a LocalSigner
pub fn signer_address(signer: &PrivateKeySigner) -> Address {
    signer.address()
}

/// Resolve the signer for EIP-712 signing (only for PrivateKey mode)
pub fn resolve_signer(identity: &Identity) -> Result<PrivateKeySigner> {
    match identity {
        Identity::PrivateKey(key) => create_signer(key),
        Identity::SenderIndex(_) => {
            anyhow::bail!("EIP-712 signing not available in sender-index mode. Use --private-key.")
        }
        Identity::None => {
            anyhow::bail!("No wallet configured. Run 'dex wallet create' first.")
        }
    }
}

/// Resolve address for API queries
pub async fn resolve_address(
    identity: &Identity,
    gateway: &GatewayClient,
) -> Result<String> {
    match identity {
        Identity::SenderIndex(idx) => {
            let resp = gateway.addresses(idx + 1).await?;
            let addrs = resp
                .as_array()
                .ok_or_else(|| anyhow::anyhow!("Expected array from /tx/addresses"))?;
            addrs
                .last()
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("No address returned for sender_index {idx}"))
        }
        Identity::PrivateKey(key) => {
            // API 需要 Sui 地址（Blake2b256 派生），不是 ETH 地址
            let signer = create_signer(key)?;
            Ok(derive_sui_address_hex(&signer))
        }
        Identity::None => {
            anyhow::bail!(
                "No wallet configured. Run 'dex wallet create' or use '--sender-index <N>'."
            );
        }
    }
}

/// Derive 32-byte Sui address from secp256k1 signer.
/// Formula: Blake2b256(0x01 || compressed_pubkey)
pub fn derive_sui_address(signer: &PrivateKeySigner) -> [u8; 32] {
    use alloy::signers::Signer;
    let verifying_key = signer.credential().verifying_key();
    let compressed = verifying_key.to_sec1_bytes(); // 33 bytes compressed

    // Blake2b with 32-byte output = Blake2b256
    type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;
    let result = Blake2b256::new()
        .chain_update([0x01]) // Secp256k1 signature scheme flag
        .chain_update(&compressed)
        .finalize();

    let mut addr = [0u8; 32];
    addr.copy_from_slice(&result);
    addr
}

/// Derive Sui address as 0x-prefixed hex string
pub fn derive_sui_address_hex(signer: &PrivateKeySigner) -> String {
    let addr = derive_sui_address(signer);
    format!("0x{}", hex::encode(addr))
}
