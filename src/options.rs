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
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    /// select a proper interval for the data
    /// 1m goes back to 4-5 days
    /// 5m goes back to ~80 days
    /// others goes back to the initial trading date
    #[clap(long, default_value = "1d", possible_values = &["1m", "5m", "1d", "5d", "1wk", "1mo", "3mo"])]
    pub interval: String,
    // #[clap(subcommand)]
    // subcmd: SubCommand,
    /// Request rate in terms of ms
    #[clap(long, default_value = "100")]
    pub rate: MyDuration,
}

pub fn parse() -> Opts {
    let opts: Opts = Opts::parse();
    if let Some(start) = opts.start {
        if let Some(end) = opts.end {
            if start >= end {
                panic!("start date is greater or equal to end date")
            }
        }
    }
    info!("{:?}", opts);
    opts
}

// #[derive(Clap)]
// enum SubCommand {
//     #[clap(version = "1.3", author = "Someone E. <someone_else@other.com>")]
//     Test(Test),
// }
//
// /// A subcommand for controlling testing
// #[derive(Clap)]
// struct Test {
//     /// Print debug info
//     #[clap(short)]
//     debug: bool
// }

#[cfg(test)]
mod tests {
    // use super::*;
    #[test]
    #[should_panic(expected = "I am scared!")]
    fn test_fun() {
        panic!("I am scared!");
    }
}
