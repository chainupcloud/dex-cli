use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DexConfig {
    pub api_url: Option<String>,
    pub gateway_url: Option<String>,
    pub private_key: Option<String>,
    pub address: Option<String>,
    pub environment: Option<String>,
    pub default_subaccount: Option<u32>,
    pub sender_index: Option<u32>,
    /// Agent private key (secp256k1 hex, for session key trading)
    pub agent_key: Option<String>,
    /// Agent expiry timestamp (ms), 0 = permanent
    pub agent_valid_until: Option<u64>,
    /// EVM RPC URL for bridge deposits (e.g. Sepolia)
    pub eth_rpc_url: Option<String>,
    /// Bridge contract address on EVM chain
    pub bridge_address: Option<String>,
    /// USDC token address on EVM chain
    pub usdc_address: Option<String>,
}

/// Default bridge addresses for devnet (Sepolia)
pub const DEFAULT_ETH_RPC_URL: &str = "https://rpc.sepolia.org";
pub const DEFAULT_BRIDGE_ADDRESS_DEV: &str = "0x1A741c8Ae351eEf38c2887cE2B64587756D44d1B";
pub const DEFAULT_BRIDGE_ADDRESS_TEST: &str = "0x02529b8b514D67F10A0B4f3af2d52b239A77fCc5";
pub const DEFAULT_USDC_ADDRESS: &str = "0x4f1b97893ec3ab8a2aa320927b17e889aa152ff5";

/// 配置文件路径: ~/.config/dex/config.json
pub fn config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Cannot determine config directory")?
        .join("dex");
    Ok(config_dir.join("config.json"))
}

/// 加载配置文件，文件不存在时返回默认值
pub fn load_config() -> Result<DexConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(DexConfig::default());
    }
    let content = std::fs::read_to_string(&path)
        .context("Failed to read config file")?;
    let config: DexConfig = serde_json::from_str(&content)
        .context("Failed to parse config file")?;
    Ok(config)
}

/// 保存配置文件，自动创建目录，设置权限
pub fn save_config(config: &DexConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .context("Failed to create config directory")?;
        // Unix: 目录 0o700
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    let content = serde_json::to_string_pretty(config)
        .context("Failed to serialize config")?;
    std::fs::write(&path, content)
        .context("Failed to write config file")?;
    // Unix: 文件 0o600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// 删除配置文件
pub fn delete_config() -> Result<()> {
    let path = config_path()?;
    if path.exists() {
        std::fs::remove_file(&path).context("Failed to delete config file")?;
    }
    Ok(())
}
