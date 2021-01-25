use chrono::{DateTime, FixedOffset, NaiveDateTime};
use itertools::izip;
use serde::Deserialize;
use std::{error::Error, fs::File, io::BufReader};

#[derive(Deserialize)]
pub struct TradePeriod {
    pub timezone: String,
    pub start: u64,
    pub end: u64,
    pub gmtoffset: i32,
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
    pub gmtoffset: i32,
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
    pub volume: Vec<u64>,
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
    pub timestamp: Vec<i64>,
    pub indicators: Indicators,
}

#[derive(Deserialize)]
pub struct Chart {
    pub result: Vec<V8Result>,
    pub error: Option<String>,
}

pub fn load_from_json(path: &str) -> Result<Chart, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader)?;
    Ok(u)
}

pub struct Record {
    pub timestamp: DateTime<FixedOffset>,
    pub volume: u64,
    pub high: f64,
    pub low: f64,
    pub open: f64,
    pub close: f64,
    pub adjclose: f64,
}

pub struct DataSet {
    pub records: Vec<Record>,
    pub meta: V8Meta,
}

impl From<Chart> for Vec<DataSet> {
    fn from(chart: Chart) -> Self {
        let mut dataset_vec: Vec<DataSet> = vec![];
        for result in chart.result.into_iter() {
            let vohlca_iter = result
                .indicators
                .quote
                .iter()
                .zip(result.indicators.adjclose.iter())
                .flat_map(|(ohlcv, adj)| {
                    izip!(
                        &ohlcv.volume,
                        &ohlcv.open,
                        &ohlcv.high,
                        &ohlcv.low,
                        &ohlcv.close,
                        &adj.adjclose
                    )
                });
            let gmtoffset = result.meta.gmtoffset;
            let mut ds = DataSet {
                records: Vec::new(),
                meta: result.meta,
            };
            for (t, (v, o, h, l, c, a)) in result.timestamp.iter().zip(vohlca_iter) {
                let naive = NaiveDateTime::from_timestamp(*t, 0);
                let offset = if gmtoffset > 0 {
                    FixedOffset::east(gmtoffset)
                } else {
                    FixedOffset::west(gmtoffset)
                };
                let tm = DateTime::<FixedOffset>::from_utc(naive, offset);
                ds.records.push(Record {
                    timestamp: tm,
                    volume: *v,
                    open: *o,
                    high: *h,
                    low: *l,
                    close: *c,
                    adjclose: *a,
                });
            }
            dataset_vec.push(ds);
        }
        dataset_vec
    }
}
