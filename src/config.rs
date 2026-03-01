use std::env;
use chrono::Duration;

pub struct Config {
    //交易所参数
    pub okx:ExchangeConfig,
    pub bybit:ExchangeConfig,
    //套利规则
    pub max_delay_ms: i64,         //最大延迟：10ms
    pub min_depth_aty: f64,        //最小深度（可调整）
    pub total_fee: f64,            //双边总手续费
    pub slippage_padding: f64,     //滑点垫  %
    pub margin_warning_rate: f64,  //保证金预警：20%
    pub testnet: bool,             //测试网开关
}

pub struct ExchangeConfig {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: String,        //OKX专属
    pub ws_public_url: String,     //行情WS地址
    pub ws_private_url: String,    //私有下单ws地址
    pub inst_id: String,           //交易对    
}

impl Config {
    pub fn load() -> Self {
        Self {
            okx:ExchangeConfig {
                api_key: env::var("OKX_API_KEY").unwrap(),
                api_secret: env::var("OKX_API_SECRET").unwrap(),
                passphrase: env::var("OKX_PASSPHRASE").unwrap(),
                ws_public_url: if self::is_testnet() {
                    "wss://wspap.okx.com:8443/ws/v5/public?brokerId=9999"  //虚拟盘
                } else {
                    "wss://ws.okx.com:8443/ws/v5/public"   //实盘
                }.to_string(),
                ws_private_url: if self::is_testnet() {
                    "wss://wspap.okx.com:8443/ws/v5/private?brokerId=9999"
                } else {
                    "wss://wspap.okx.com:8443/ws/v5"
                }.to_string(),
                inst_id: "BTC_USDT_SWAP".to_string(),
            },
            bybit: ExchangeConfig {
                api_key: env::var("BYBIT_API_KEY").unwrap(),
                api_secret: env::var("BYBIT_API_SECRET").unwrap(),
                passphrase: "".to_string(), // Bybit无passphrase
                ws_public_url: if Self::is_testnet() {
                    "wss://stream-testnet.bybit.com/v5/public/linear"
                } else {
                    "wss://stream.bybit.com/v5/public/linear"
                }.to_string(),
                ws_private_url: if Self::is_testnet() {
                    "wss://stream-testnet.bybit.com/v5/private/linear"
                } else {
                    "wss://stream.bybit.com/v5/private/linear"
                }.to_string(),
                inst_id: "BTCUSDT".to_string(),
            },
            max_delay_ms: 10,
            min_depth_qty: 0.001,
            total_fee: 0.0021,
            slippage_padding:0.0005,
            margin_warning_rate:0.2,
            testnet:env::var("TESTNET").unwrap_or("true".to_string()) == "true",
        }
    }

    fn is_testnet() -> bool {
        env::var("TESTNET").unwrap_or("true".to_string()) == "true"
    }
}
123123123