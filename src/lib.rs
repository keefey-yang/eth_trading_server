use anyhow::{Result};

include!("eth_trading_server.rs");

pub async fn get_balance(wallet_address: Address, contract_address: Option<Address>) -> Result<BalanceInfo,  anyhow::Error> {
    match contract_address {
        Some(token_address) => {
            // Fetch ERC20 token balance
            GLOBAL_INSTANCE.get_erc20_balance(wallet_address, token_address).await.map_err(|e| e.into())
        }
        None => {
            // Fetch ETH balance
            GLOBAL_INSTANCE.get_eth_balance(wallet_address).await.map_err(|e| e.into())
        }
    }
}

pub async fn get_token_price<T: std::any::Any>(token_or_symbol: &T) -> Result<PriceInfo, Box<dyn std::error::Error>> {
    return match token_or_symbol.type_id() {
        id if id == std::any::TypeId::of::<String>() => {
            // Handle String input (token symbol)
            let Some(symbol) = (token_or_symbol as &dyn std::any::Any).downcast_ref::<String>() else {
                return Err("Failed to downcast to String".into());
            };
            GLOBAL_INSTANCE.get_token_price_by_symbol(symbol).await.map_err(|e| e.into())
        }
        id if id == std::any::TypeId::of::<Address>() => {
            // Handle Address input (token contract address)
            let Some(token_address) = (token_or_symbol as &dyn std::any::Any).downcast_ref::<Address>() else {
                return Err("Failed to downcast to Address".into());
            };
            GLOBAL_INSTANCE.get_token_price_by_address(*token_address).await.map_err(|e| e.into())
        }
        _ => Err("Unsupported type".into())
    };
}

pub async fn swap_tokens(from_token: Address, to_token: Address, amount: f64, slippage: f64) -> Result<SwapSimulationInfo,  anyhow::Error> {
    GLOBAL_INSTANCE.swap_tokens(from_token, to_token, amount, slippage).await.map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use ethers::contract;

    use super::*;
    use std::str::FromStr;

    #[tokio::test]
    async fn test_get_balance() {
        let wallet_address: Address = Address::from_str("0xeD30b09f3D699c2B3bA730C7a10f6EB457e07888").unwrap();
        let balance = get_balance(wallet_address, None).await.unwrap();
        assert_eq!(balance.balance, "0");

        let contract_address: Address = Address::from_str("0x0E573Ce2736Dd9637A0b21058352e1667925C7a8").unwrap();
        let balance = get_balance(wallet_address, Some(contract_address)).await.unwrap();
        assert_eq!(balance.balance, "0");
    }

    #[tokio::test]
    async fn test_get_token_price() {
        let symbol  = "ETH".to_string();
        let price = get_token_price(&symbol).await.unwrap();
        assert_eq!(price.price_usd, Some(100.0));
        assert_eq!(price.price_eth, Some(1.0));

        let contract_address: Address = "0x0000000000000000000000000000000000000000".parse().unwrap();
        let price = get_token_price(&contract_address).await.unwrap();
        assert_eq!(price.price_usd, Some(200.0));
        assert_eq!(price.price_eth, Some(2.0));
    }

    #[tokio::test]
    async fn test_swap_tokens() {
        let from_token: Address = "0x0000000000000000000000000000000000000000".parse().unwrap();
        let to_token: Address = "0x0000000000000000000000000000000000000000".parse().unwrap();
        let amount = 1.0;
        let slippage = 0.01;
        let swap_info = swap_tokens(from_token, to_token, amount, slippage).await.unwrap();
        assert_eq!(swap_info.input_amount, "1");
        assert_eq!(swap_info.output_amount, "0.99");
    }
}