use ethers::{
    abi::{Abi, Token}, contract::{self, Contract}, middleware::transformer::ds_proxy::factory, prelude::*, providers::{Http, Middleware, Provider}, types::{Address, Bytes, TransactionRequest, H160, U256}, utils
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use lazy_static::lazy_static;
use std::{str::FromStr, sync::Arc};

#[derive(Error, Debug)]
pub enum TokenServiceError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Contract call error: {0}")]
    ContractCallError(String),
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
}

// 余额信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub balance: String,
    pub decimals: u8,
    pub symbol: String,
    pub formatted: String,
}

// 价格信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceInfo {
    pub symbol: String,
    pub price_usd: Option<f64>,
    pub price_eth: Option<f64>,
    pub timestamp: u64,
}

// 交易模拟结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapSimulationInfo {
    pub input_amount: String,
    pub output_amount: String,
    pub gas_estimate: u64,
    pub gas_cost_eth: String,
    pub gas_cost_usd: Option<String>,
    pub price_impact: f64,
    pub success: bool,
}

pub struct TokenService {
    provider: Arc<Provider<Http>>,
}

const ETH_RPC_URL: &str = "https://mainnet.infura.io/v3/3f2af82e9b964e57bbb9d85f720f3bcb";
const UNISWAP_V2_ROUTER_ADDRESS: &str = "0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D";

lazy_static! {
    static ref GLOBAL_INSTANCE: TokenService = {
        let provider = Provider::<Http>::try_from(ETH_RPC_URL).unwrap();
        println!("Connected to Ethereum node at {}", ETH_RPC_URL);
        println!("Provider details: {:?}", provider);
        TokenService { provider: Arc::new(provider) }
    };
}

// ERC20 ABI
lazy_static::lazy_static! {
    static ref ERC20_ABI: Abi = serde_json::from_str(r#"
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
    ]"#).unwrap();
}

// Uniswap V2 Router ABI 片段
lazy_static::lazy_static! {
    static ref UNISWAP_V2_ROUTER_ABI: Abi = serde_json::from_str(r#"[{"inputs":[{"internalType":"address","name":"_factory","type":"address"},{"internalType":"address","name":"_WETH","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"WETH","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"},{"internalType":"uint256","name":"amountADesired","type":"uint256"},{"internalType":"uint256","name":"amountBDesired","type":"uint256"},{"internalType":"uint256","name":"amountAMin","type":"uint256"},{"internalType":"uint256","name":"amountBMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"addLiquidity","outputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"amountB","type":"uint256"},{"internalType":"uint256","name":"liquidity","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"amountTokenDesired","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"addLiquidityETH","outputs":[{"internalType":"uint256","name":"amountToken","type":"uint256"},{"internalType":"uint256","name":"amountETH","type":"uint256"},{"internalType":"uint256","name":"liquidity","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"factory","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"reserveIn","type":"uint256"},{"internalType":"uint256","name":"reserveOut","type":"uint256"}],"name":"getAmountIn","outputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"reserveIn","type":"uint256"},{"internalType":"uint256","name":"reserveOut","type":"uint256"}],"name":"getAmountOut","outputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"}],"name":"getAmountsIn","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"}],"name":"getAmountsOut","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"reserveA","type":"uint256"},{"internalType":"uint256","name":"reserveB","type":"uint256"}],"name":"quote","outputs":[{"internalType":"uint256","name":"amountB","type":"uint256"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountAMin","type":"uint256"},{"internalType":"uint256","name":"amountBMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"removeLiquidity","outputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"amountB","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"removeLiquidityETH","outputs":[{"internalType":"uint256","name":"amountToken","type":"uint256"},{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"removeLiquidityETHSupportingFeeOnTransferTokens","outputs":[{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bool","name":"approveMax","type":"bool"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"removeLiquidityETHWithPermit","outputs":[{"internalType":"uint256","name":"amountToken","type":"uint256"},{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bool","name":"approveMax","type":"bool"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"removeLiquidityETHWithPermitSupportingFeeOnTransferTokens","outputs":[{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountAMin","type":"uint256"},{"internalType":"uint256","name":"amountBMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bool","name":"approveMax","type":"bool"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"removeLiquidityWithPermit","outputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"amountB","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapETHForExactTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactETHForTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactETHForTokensSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForETH","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForETHSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForTokensSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapTokensForExactETH","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapTokensForExactTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"stateMutability":"payable","type":"receive"}]"#).unwrap();
}

impl TokenService {
    async fn get_eth_balance(&self, wallet_address: Address) -> Result<BalanceInfo> {
        // Placeholder implementation
        let balance = self.provider
            .get_balance(wallet_address, None)
            .await
            .map_err(|e| TokenServiceError::RpcError(e.to_string()))?;
        
        let balance_eth = utils::format_units(balance, "ether")
            .map_err(|e| TokenServiceError::InvalidAmount(e.to_string()))?;
        Ok(BalanceInfo {
            balance: balance.to_string(),
            decimals: 18,
            symbol: "ETH".into(),
            formatted: format!("{} ETH", balance_eth),
        })
    }

    async fn get_erc20_balance(&self, wallet_addr: Address, contract_address: Address) -> Result<BalanceInfo> {
        // Placeholder implementation
        let contract = Contract::new(contract_address, ERC20_ABI.clone(), self.provider.clone());
        // 获取余额
        let balance: U256 = contract
            .method::<_, U256>("balanceOf", wallet_addr).unwrap()
            .call()
            .await
            .map_err(|e| TokenServiceError::ContractCallError(e.to_string()))?;

        // 获取小数位
        let decimals: u8 = contract
            .method::<_, u8>("decimals", ())?
            .call()
            .await
            .map_err(|e| TokenServiceError::ContractCallError(e.to_string()))?;

        // 获取符号
        let symbol: String = contract
            .method::<_, String>("symbol", ())?
            .call()
            .await
            .unwrap_or_else(|_| "UNKNOWN".to_string());

        let formatted = utils::format_units(balance, decimals as i32)?;
        
        Ok(BalanceInfo {
            balance: balance.to_string(),
            decimals,
            symbol,
            formatted,
        })
    }

    async fn get_token_price_by_symbol(&self, symbol: &str) -> Result<PriceInfo> {
        // Placeholder implementation
        let contract_address = Address::from_str(UNISWAP_V2_ROUTER_ADDRESS).unwrap();
        let contract = Contract::new(contract_address, UNISWAP_V2_ROUTER_ABI.clone(), self.provider.clone());
        // 调用合约方法获取价格信息
        let amount_in = U256::from(1 as u64); // 1.0 ETH，考虑小数位
        let factory_address = Address::from_str("0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f")?;
        let weth_address = Address::from_str("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2")?;
        let usdt_address = Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7")?;
        let bnb_address = Address::from_str("0xB8c77482e45F1F44dE1745F52C74426C631bDD52")?;
        let paxg_address = Address::from_str("0x45804880De22913dAFE09f4980848ECE6EcbAf78")?;
        let path = vec![
            paxg_address,
            usdt_address,
        ];
        let amounts = contract.method::<_, Vec<U256>>("getAmountsOut", (amount_in, path))?
            .call()
            .await
            .map_err(|e| TokenServiceError::ContractCallError(e.to_string()))?;
        println!("Amounts out for 1 DAI to USDC: {:?}", amounts);
        println!("Amounts last is : {:?}", amounts.last());
        Ok(PriceInfo {
            symbol: symbol.into(),
            price_usd: Some(100.0),
            price_eth: Some(1.0),
            timestamp: 0,
        })
    }

    async fn get_token_price_by_address(&self, token_address: Address) -> Result<PriceInfo> {
        // Placeholder implementation
        // let usdc_address = Address::from_str("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
        // let uni_address = Address::from_str("0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984")?;
        // let router_address = Address::from_str("0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D")?;

        // abigen!(
        //     IUniswapV2Router,
        //     "function getAmountsOut(uint amountIn, address[] memory path) public view returns (uint[] memory amounts)"
        // );

        // let router = IUniswapV2Router::new(router_address, client.clone());

        // // 设置输入金额 (100 USDC, 考虑 6 位小数)
        // let amount_in = U256::from(100_000_000u64); // 100 * 10^6
        // let path = vec![usdc_address, uni_address]; // 兑换路径: USDC -> UNI

        // // 调用合约获取报价
        // let amounts: Vec<U256> = router.get_amounts_out(amount_in, path).call().await?;
        
        // // amounts[0] 是输入金额 (USDC), amounts[1] 是输出金额 (UNI)
        // // UNI 有 18 位小数，需要转换
        // let amount_out_wei = amounts[1];
        // let amount_out_ether = amount_out_wei.as_u128() as f64 / 1e18;
        // println!("Estimated UNI for 100 USDC: {}", amount_out_ether);
        Ok(PriceInfo {
            symbol: "TOKEN".into(),
            price_usd: Some(200.0),
            price_eth: Some(2.0),
            timestamp: 0,
        })
    }

    async fn swap_tokens(&self, from_token: Address, to_token: Address, amount: f64, slippage: f64) -> Result<SwapSimulationInfo> {
        // Placeholder implementation
        Ok(SwapSimulationInfo {
            input_amount: amount.to_string(),
            output_amount: (amount * (1.0 - slippage)).to_string(),
            gas_estimate: 21000,
            gas_cost_eth: "0.01".into(),
            gas_cost_usd: Some("20.0".into()),
            price_impact: 0.5,
            success: true,
        })
    }

}