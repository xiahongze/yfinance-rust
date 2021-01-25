use serde::Deserialize;

#[derive(Deserialize)]
pub struct TradePeriod {
    pub timezone: String,
    pub start: u64,
    pub end: u64,
    pub gmtoffset: u16,
}
#[derive(Deserialize)]
pub struct CurrentTradePeriod {
    pub pre: TradePeriod,
    pub regular: TradePeriod,
    pub post: TradePeriod,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct V8Meta {
    pub currency: String,
    pub symbol: String,
    pub exchange_name: String,
    pub instrument_type: String,
    pub first_trade_date: u64,
    pub regular_market_time: u64,
    pub gmtoffset: u16,
    pub timezone: String,
    pub exchange_timezone_name: String,
    pub regular_market_price: f32,
    pub chart_previous_close: f32,
    pub price_hint: f32,
    pub current_trading_period: CurrentTradePeriod,
    pub data_granularity: String,
    pub range: String,
    pub valid_ranges: Vec<String>,
}
#[derive(Deserialize)]
pub struct OHLCV {
    pub volume: Vec<f64>,
    pub high: Vec<f64>,
    pub close: Vec<f64>,
    pub low: Vec<f64>,
    pub open: Vec<f64>,
}
#[derive(Deserialize)]
pub struct AdjClose {
    pub adjclose: Vec<f64>,
}
#[derive(Deserialize)]
pub struct Indicators {
    pub quote: Vec<OHLCV>,
    pub adjclose: Vec<AdjClose>,
}
#[derive(Deserialize)]
pub struct V8Result {
    pub meta: V8Meta,
    pub timestamp: Vec<u64>,
    pub indicators: Indicators,
}

#[derive(Deserialize)]
pub struct Chart {
    pub result: Vec<V8Result>,
    pub error: Option<String>,
}
