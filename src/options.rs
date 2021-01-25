use chrono::NaiveDate;
use clap::Clap;
use std::{num::ParseIntError, str::FromStr, time::Duration};
#[derive(Debug)]
pub struct MyDuration {
    pub duration: Duration,
}

impl FromStr for MyDuration {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u64>() {
            Ok(ms) => Ok(MyDuration {
                duration: Duration::from_millis(ms),
            }),
            Err(err) => Err(err),
        }
    }
}

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Hongze Xia hongzex@gmail.com>")]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

pub fn parse() -> Opts {
    let opts: Opts = Opts::parse();
    info!("{:?}", opts);
    opts
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    // #[clap(version = "1.3", author = "Someone E. <someone_else@other.com>")]
    Download(DownloadOpts),
    Convert(ConvertOpts),
}

/// Download historical data from yahoo finance
#[derive(Clap, Debug)]
pub struct DownloadOpts {
    /// List of symbols to download. Required.
    pub symbols: Vec<String>,
    /// A start date to download from
    #[clap(long)]
    pub start: Option<NaiveDate>,
    /// An end date. Default to Now
    #[clap(long)]
    pub end: Option<NaiveDate>,
    /// Include pre & post market data
    #[clap(long)]
    pub include_pre_post: bool,
    /// Sets a output directory. The format of the output CSV looks like
    /// `SYMBOL_20200202_20200303.csv`
    #[clap(short, long, default_value = ".")]
    pub output_dir: String,
    /// select a proper interval for the data
    /// 1m goes back to 4-5 days
    /// 5m goes back to ~80 days
    /// others goes back to the initial trading date
    #[clap(long, default_value = "1d", possible_values = &["1m", "5m", "1d", "5d", "1wk", "1mo", "3mo"])]
    pub interval: String,
    /// Request rate in terms of ms
    #[clap(long, default_value = "100")]
    pub rate: MyDuration,
    /// Convert JSON to CSV
    #[clap(long)]
    pub convert: bool,
}
/// Convert yahoo finance v8 json into csv
#[derive(Clap, Debug)]
pub struct ConvertOpts {
    pub input_dir: String,
}

#[cfg(test)]
mod tests {
    // use super::*;
    #[test]
    #[should_panic(expected = "I am scared!")]
    fn test_fun() {
        panic!("I am scared!");
    }
}
