/// Configuration loading from .env file
/// 
/// Loads all environment variables for:
/// - Yellowstone gRPC connection
/// - Jito bundle engine connection
/// - Solana RPC endpoints (devnet/mainnet)
/// - Wallet keypair path
/// - Tip account for dynamic fee calculation

use anyhow::{anyhow, Context, Result};
use std::env;

/// Global application configuration
#[derive(Clone, Debug)]
pub struct Config {
    /// Yellowstone gRPC endpoint (with TLS)
    pub yellowstone_grpc_endpoint: String,

    /// Jito bundle engine endpoint
    pub jito_bundle_endpoint: String,

    /// Solana RPC URL (can be devnet, testnet, or mainnet)
    pub solana_rpc_url: String,

    /// Path to wallet keypair JSON file (or None if using PRIVATE_KEY env var)
    pub wallet_keypair_path: Option<String>,

    /// Private key hex string (alternative to keypair file)
    pub private_key: Option<String>,

    /// Jito tip account (fee recipient for MEV)
    pub jito_tip_account: String,

    /// Environment: "devnet", "testnet", "mainnet"
    pub environment: String,

    /// Maximum acceptable tip multiplier for dynamic fee calculation
    pub max_tip_multiplier: f64,

    /// Minimum tip in lamports (floor)
    pub min_tip_lamports: u64,

    /// Enable detailed logging
    pub debug: bool,
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        // Load .env file if present
        dotenv::dotenv().ok();

        let env = env::var("ENVIRONMENT").unwrap_or_else(|_| "devnet".to_string());

        // Validate environment
        if !["devnet", "testnet", "mainnet"].contains(&env.as_str()) {
            return Err(anyhow!(
                "ENVIRONMENT must be one of: devnet, testnet, mainnet. Got: {}",
                env
            ));
        }

        // Yellowstone gRPC endpoint
        let yellowstone_grpc_endpoint = env::var("YELLOWSTONE_GRPC_ENDPOINT")
            .context("YELLOWSTONE_GRPC_ENDPOINT not set")?;

        // Jito bundle endpoint
        let jito_bundle_endpoint = env::var("JITO_BUNDLE_ENDPOINT")
            .context("JITO_BUNDLE_ENDPOINT not set")?;

        // Solana RPC URL (can be overridden per environment, or use default)
        let solana_rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| {
            match env.as_str() {
                "mainnet" => "https://api.mainnet-beta.solana.com".to_string(),
                "testnet" => "https://api.testnet.solana.com".to_string(),
                _ => "https://api.devnet.solana.com".to_string(),
            }
        });

        // Wallet keypair path (optional if using PRIVATE_KEY)
        let wallet_keypair_path = env::var("WALLET_KEYPAIR_PATH").ok();

        // Private key (optional if using WALLET_KEYPAIR_PATH)
        let private_key = env::var("PRIVATE_KEY").ok();

        // At least one of wallet_keypair_path or private_key must be set
        if wallet_keypair_path.is_none() && private_key.is_none() {
            return Err(anyhow!(
                "Either WALLET_KEYPAIR_PATH or PRIVATE_KEY must be set"
            ));
        }

        // Jito tip account (where MEV rewards go)
        let jito_tip_account = env::var("JITO_TIP_ACCOUNT")
            .context("JITO_TIP_ACCOUNT not set")?;

        // Optional tuning parameters
        let max_tip_multiplier = env::var("MAX_TIP_MULTIPLIER")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(2.5);

        let min_tip_lamports = env::var("MIN_TIP_LAMPORTS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1_000);

        let debug = env::var("DEBUG")
            .ok()
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            yellowstone_grpc_endpoint,
            jito_bundle_endpoint,
            solana_rpc_url,
            wallet_keypair_path,
            private_key,
            jito_tip_account,
            environment: env,
            max_tip_multiplier,
            min_tip_lamports,
            debug,
        })
    }

    /// Validate that critical endpoints are reachable
    /// (Called at startup, not blocking)
    pub async fn validate_endpoints(&self) -> Result<()> {
        tracing::info!("Validating configuration endpoints...");

        // Validate Yellowstone gRPC endpoint format
        if !self.yellowstone_grpc_endpoint.starts_with("http://")
            && !self.yellowstone_grpc_endpoint.starts_with("https://")
        {
            return Err(anyhow!(
                "YELLOWSTONE_GRPC_ENDPOINT must start with http:// or https://"
            ));
        }

        // Validate Jito endpoint format
        if !self.jito_bundle_endpoint.starts_with("http://")
            && !self.jito_bundle_endpoint.starts_with("https://")
        {
            return Err(anyhow!(
                "JITO_BUNDLE_ENDPOINT must start with http:// or https://"
            ));
        }

        // Validate Solana RPC URL format
        if !self.solana_rpc_url.starts_with("http://")
            && !self.solana_rpc_url.starts_with("https://")
        {
            return Err(anyhow!(
                "SOLANA_RPC_URL must start with http:// or https://"
            ));
        }

        tracing::info!(
            environment = %self.environment,
            yellowstone = %self.yellowstone_grpc_endpoint,
            jito = %self.jito_bundle_endpoint,
            solana_rpc = %self.solana_rpc_url,
            "✅ Configuration validated"
        );

        Ok(())
    }

    /// Check if running on mainnet
    pub fn is_mainnet(&self) -> bool {
        self.environment == "mainnet"
    }

    /// Check if running on testnet
    pub fn is_testnet(&self) -> bool {
        self.environment == "testnet"
    }

    /// Check if running on devnet
    pub fn is_devnet(&self) -> bool {
        self.environment == "devnet"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_environment_validation() {
        env::set_var("YELLOWSTONE_GRPC_ENDPOINT", "https://example.com");
        env::set_var("JITO_BUNDLE_ENDPOINT", "https://jito.example.com");
        env::set_var("WALLET_KEYPAIR_PATH", "/tmp/key.json");
        env::set_var("JITO_TIP_ACCOUNT", "9B5X4b3XfBmrKzf7YsXwqYuvz2aLf5cuucsBiB1A6qws");
        env::set_var("ENVIRONMENT", "devnet");

        let config = Config::load().expect("Config load failed");
        assert_eq!(config.environment, "devnet");
        assert!(config.is_devnet());
        assert!(!config.is_mainnet());
    }

    #[test]
    fn test_config_missing_endpoint() {
        env::remove_var("YELLOWSTONE_GRPC_ENDPOINT");
        let result = Config::load();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_max_tip_multiplier_default() {
        env::set_var("YELLOWSTONE_GRPC_ENDPOINT", "https://example.com");
        env::set_var("JITO_BUNDLE_ENDPOINT", "https://jito.example.com");
        env::set_var("WALLET_KEYPAIR_PATH", "/tmp/key.json");
        env::set_var("JITO_TIP_ACCOUNT", "9B5X4b3XfBmrKzf7YsXwqYuvz2aLf5cuucsBiB1A6qws");
        env::remove_var("MAX_TIP_MULTIPLIER");

        let config = Config::load().expect("Config load failed");
        assert_eq!(config.max_tip_multiplier, 2.5);
    }
      }
      
