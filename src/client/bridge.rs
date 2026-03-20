//! EVM bridge client — approve USDC + depositUSDCForSubaccount
//!
//! Same secp256k1 key used for both EVM transactions and DEX EIP-712 signing.

use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol;
use anyhow::{Context, Result};

// ERC20 approve + balanceOf + allowance
sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function allowance(address owner, address spender) external view returns (uint256);
        function balanceOf(address account) external view returns (uint256);
    }
}

// Bridge deposit
sol! {
    #[sol(rpc)]
    interface ISuiBridge {
        function depositUSDCForSubaccount(
            bytes32 recipientAddress,
            uint32 subaccountNumber,
            uint256 amount
        ) external;
    }
}

pub struct BridgeClient {
    eth_rpc_url: String,
    bridge_address: Address,
    usdc_address: Address,
}

pub struct DepositResult {
    pub tx_hash: String,
    pub amount: u64,
    pub sui_address: String,
    pub subaccount: u32,
}

impl BridgeClient {
    pub fn new(eth_rpc_url: &str, bridge_address: &str, usdc_address: &str) -> Result<Self> {
        Ok(Self {
            eth_rpc_url: eth_rpc_url.to_string(),
            bridge_address: bridge_address
                .parse()
                .context("Invalid bridge contract address")?,
            usdc_address: usdc_address
                .parse()
                .context("Invalid USDC token address")?,
        })
    }

    fn rpc_url(&self) -> Result<reqwest::Url> {
        self.eth_rpc_url
            .parse()
            .context("Invalid ETH RPC URL")
    }

    /// Check USDC balance on EVM chain
    pub async fn usdc_balance(&self, signer: &PrivateKeySigner) -> Result<u64> {
        let provider = ProviderBuilder::new()
            .wallet(alloy::network::EthereumWallet::from(signer.clone()))
            .connect_http(self.rpc_url()?);

        let usdc = IERC20::new(self.usdc_address, provider);
        let bal = usdc.balanceOf(signer.address()).call().await
            .context("Failed to query USDC balance")?;

        Ok(bal.to::<u64>())
    }

    /// Deposit USDC to DEX subaccount via bridge.
    ///
    /// 1. Check USDC balance
    /// 2. Approve bridge contract (if needed)
    /// 3. Call depositUSDCForSubaccount(suiAddress, subaccountNumber, amount)
    pub async fn deposit(
        &self,
        signer: &PrivateKeySigner,
        sui_address: [u8; 32],
        subaccount_number: u32,
        amount: u64,
    ) -> Result<DepositResult> {
        anyhow::ensure!(subaccount_number < 128, "Subaccount must be < 128");
        anyhow::ensure!(amount > 0, "Amount must be > 0");

        let wallet = alloy::network::EthereumWallet::from(signer.clone());
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(self.rpc_url()?);

        let usdc = IERC20::new(self.usdc_address, &provider);
        let bridge = ISuiBridge::new(self.bridge_address, &provider);
        let amount_u256 = U256::from(amount);

        // 1. Check balance
        let balance = usdc.balanceOf(signer.address()).call().await
            .context("Failed to query USDC balance")?;
        anyhow::ensure!(
            balance >= amount_u256,
            "Insufficient USDC balance: have {} raw, need {} raw",
            balance, amount
        );

        // 2. Check & set allowance
        let allowance = usdc
            .allowance(signer.address(), self.bridge_address)
            .call()
            .await
            .context("Failed to query USDC allowance")?;

        if allowance < amount_u256 {
            eprintln!("Approving USDC spending for bridge contract...");
            let approve_tx = usdc
                .approve(self.bridge_address, U256::MAX)
                .send()
                .await
                .context("Failed to send USDC approve transaction")?;

            let receipt = approve_tx
                .get_receipt()
                .await
                .context("Failed to confirm approve transaction")?;

            eprintln!(
                "  Approved. tx: 0x{}",
                hex::encode(receipt.transaction_hash.as_slice())
            );
        }

        // 3. Bridge deposit
        eprintln!("Submitting bridge deposit...");
        let recipient = alloy::primitives::FixedBytes::from(sui_address);

        let deposit_tx = bridge
            .depositUSDCForSubaccount(recipient, subaccount_number, amount_u256)
            .send()
            .await
            .context("Failed to send bridge deposit transaction")?;

        let receipt = deposit_tx
            .get_receipt()
            .await
            .context("Failed to confirm bridge deposit transaction")?;

        let tx_hash = format!("0x{}", hex::encode(receipt.transaction_hash.as_slice()));

        Ok(DepositResult {
            tx_hash,
            amount,
            sui_address: format!("0x{}", hex::encode(sui_address)),
            subaccount: subaccount_number,
        })
    }
}
