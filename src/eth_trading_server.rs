use anyhow::{Result, Error};
use rmcp::{
    ErrorData as McpError,
    ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{CallToolResult, Content, ErrorCode},
    schemars, tool, tool_handler, tool_router,
};

use ethers::{providers::{Http, Middleware, Provider}, types::Address};
use std::{str::FromStr};

const ETH_RPC_URL: &str = "https://mainnet.infura.io/v3/3f2af82e9b964e57bbb9d85f720f3bcb";
const ERC200_ABI_JSON: &str = r#"
[
    {
        "constant": true,
        "inputs": [{"name": "_owner", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "balance", "type": "uint256"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "decimals",
        "outputs": [{"name": "", "type": "uint8"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "symbol",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "name",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    }
]"#;

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct AddressRequest {
    pub wallet_address_str: String,
    pub token_addresses_str: Option<String>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct TokenAddressOrSymbol {
    pub address: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, schemars::JsonSchema, serde::Deserialize)]
pub struct  SwapTokenPair {
    pub from_token: String,
    pub to_token: String,
    pub amount: f64,
}

#[derive(Clone)]
pub struct ETHTradingMCP {
    tool_router: ToolRouter<ETHTradingMCP>
}

#[tool_router]
impl ETHTradingMCP {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    async fn get_eth_balance_of_wallet_address(&self, wallet_address: Address) -> Result<String, Error> {
        let provider = Provider::<Http>::try_from(ETH_RPC_URL)?;
        let balance = provider.get_balance(wallet_address, None).await?;
        let eth_balance = ethers::utils::format_ether(balance);
        Ok(format!("The balance of wallet {} is {} ETH", wallet_address, eth_balance))
    }

    async fn get_erc200_token_balance_of_wallet_address(&self, wallet_address: Address, token_address: Address) -> Result<String, Error> {
        // Mock implementation, replace with actual ERC20 token balance fetching logic
        let erc200_abi = ethers::abi::Abi::load(ERC200_ABI_JSON.as_bytes())?;
        let provider = std::sync::Arc::new(Provider::<Http>::try_from(ETH_RPC_URL)?);
        // Here you would create a contract instance and call the balanceOf function
        let contract = ethers::contract::Contract::new(token_address, erc200_abi, provider);
        let decimals = contract.method::<_, u8>("decimals", ())?.call().await?;
        let symbol = contract.method::<_, String>("symbol", ())?.call().await?;
        let balance: ethers::types::U256 = contract.method::<_, ethers::types::U256>("balanceOf", wallet_address)?.call().await?;
        let formatted_balance = ethers::utils::format_units(balance, decimals as usize)?;
        Ok(format!("The balance of wallet {} for token {} (symbol: {}) is {}", wallet_address, token_address, symbol, formatted_balance))
    }

    #[tool(description = "Say hello to the client")]
    pub async fn say_hello(&self) -> Result<CallToolResult, McpError> {
        Ok(CallToolResult::success(vec![Content::text("hello, it is my first mcp tool!")]))
    }

    #[tool(description = "Get the ETH or token balances of a wallet address for given token addresses")]
    pub async fn get_wallet_eth_or_tokens_balance(&self, Parameters(AddressRequest {wallet_address_str, token_addresses_str}): Parameters<AddressRequest>) -> Result<CallToolResult, McpError> {
        let wallet_address = Address::from_str(&wallet_address_str)
                                        .map_err(|e| McpError::new(ErrorCode::PARSE_ERROR, format!("Invalid wallet address: {}", e), None))?;
        match token_addresses_str {
            Some(token_address_str) => {
                let token_address = Address::from_str(&token_address_str)
                                        .map_err(|e| McpError::new(ErrorCode::PARSE_ERROR, format!("Invalid token address: {}", e), None))?;
                let result = self.get_erc200_token_balance_of_wallet_address(wallet_address, token_address).await
                    .map_err(|_| McpError::new(ErrorCode::INTERNAL_ERROR, "get token balance failed!", None))?;
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            None => {
                let result = self.get_eth_balance_of_wallet_address(wallet_address).await
                    .map_err(|_| McpError::new(ErrorCode::INTERNAL_ERROR, "get ETH balance failed!", None))?;
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
        }
    }

    #[tool(description = "Get the price of a token by its address or symbol")]
    pub async fn get_price_of_token(&self, Parameters(TokenAddressOrSymbol {address, symbol}): Parameters<TokenAddressOrSymbol>) -> Result<CallToolResult, McpError> {
        let token_info = if let Some(addr) = address {
            format!("address: {}", addr)
        } else if let Some(sym) = symbol {
            format!("symbol: {}", sym)
        } else {
            return Err(McpError::new(ErrorCode::PARSE_ERROR, "Either address or symbol must be provided".to_string(), None));
        };

        // Mock implementation, replace with actual API call
        Ok(CallToolResult::success(vec![Content::text(format!(
            "The price of token with {} is $100",
            token_info
        ))]))
    }

    #[tool(description = "Simulate swapping tokens on a decentralized exchange")]
    pub async fn swap_tokens_simulate(&self, Parameters(SwapTokenPair {from_token, to_token, amount}): Parameters<SwapTokenPair>) -> Result<CallToolResult, McpError> {
        // Mock implementation, replace with actual API call
        Ok(CallToolResult::success(vec![Content::text(format!(
            "Simulated swapping {} of {} to {}. You will receive approximately {} {}.",
            amount, from_token, to_token, amount * 0.98, to_token
        ))]))
    }
}

#[tool_handler]
impl ServerHandler for ETHTradingMCP {
}