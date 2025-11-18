use anyhow::Result;
use tracing_subscriber::{self, EnvFilter};
use rmcp::transport::stdio;
use rmcp::ServiceExt;
use eth_trading_server::ETHTradingMCP;

mod eth_trading_server;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our counter router
    let service = ETHTradingMCP::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::Content;

    #[tokio::test]
    async fn test_say_hello() {
        let agent = ETHTradingMCP::new();
        let result = agent.say_hello().await.unwrap();
        assert_eq!(result.content[0], Content::text("hello, it is my first mcp tool!"));
    }

    #[tokio::test]
    async fn test_get_wallet_eth_or_tokens_balance() {
        let agent = ETHTradingMCP::new();
        let request = eth_trading_server::AddressRequest {  
            wallet_address_str: "0xeD30b09f3D699c2B3bA730C7a10f6EB457e07888".to_string(),
            token_addresses_str: None,
        };
        let params = rmcp::handler::server::wrapper::Parameters(request);
        let result = agent.get_wallet_eth_or_tokens_balance(params).await.unwrap();
        assert!(result.content[0].as_text().unwrap().text.contains("The balance of wallet"));

        let request = eth_trading_server::AddressRequest {  
            wallet_address_str: "0xeD30b09f3D699c2B3bA730C7a10f6EB457e07888".to_string(),
            token_addresses_str: "0x0E573Ce2736Dd9637A0b21058352e1667925C7a8".to_string().into(),
        };
        let params = rmcp::handler::server::wrapper::Parameters(request);
        let result = agent.get_wallet_eth_or_tokens_balance(params).await.unwrap();
        assert!(result.content[0].as_text().unwrap().text.contains("The balance of wallet"));
    }

    #[tokio::test]
    async fn test_get_price_of_token() {
        let agent = ETHTradingMCP::new();
        let request = eth_trading_server::TokenAddressOrSymbol {  
            address: Some("0x6B175474E89094C44Da98b954EedeAC495dFfF".to_string()),
            symbol: None,
        };
        let params = rmcp::handler::server::wrapper::Parameters(request);
        let result = agent.get_price_of_token(params).await.unwrap();
        assert!(result.content[0].as_text().unwrap().text.contains("The price of token"));
    }

    #[tokio::test]
    async fn test_swap_tokens_simulate() {
        let agent = ETHTradingMCP::new();
        let request = eth_trading_server::SwapTokenPair {
            from_token: "ETH".to_string(),
            to_token: "DAI".to_string(),
            amount: 1.0,
        };
        let params = rmcp::handler::server::wrapper::Parameters(request);
        let result = agent.swap_tokens_simulate(params).await.unwrap();
        assert!(result.content[0].as_text().unwrap().text.contains("Simulated swapping"));
    }
}