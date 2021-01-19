use chrono::{Datelike, Local, NaiveDate};
use clap::Clap;

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Hongze Xia hongzex@gmail.com>")]
pub struct Opts {
    /// List of symbols to download. Required.
    pub symbols: Vec<String>,
    /// A start date to download from
    #[clap(long, default_value = "2020-01-01")]
    pub start: NaiveDate,
    /// An end date. Default to Now
    #[clap(long)]
    pub end: Option<NaiveDate>,
    /// Enable the Adjusted Close Column
    #[clap(long)]
    pub adjusted_close: bool,
    /// Sets a output directory. The format of the output CSV looks like
    /// `SYMBOL_20200202_20200303.csv`
    #[clap(short, long, default_value = ".")]
    pub output_dir: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    /// Note: Intraday data cannot extend last 60 days
    #[clap(long, default_value = "1d", possible_values = &["1m", "2m", "5m", "15m", "30m", "60m", "90m", "1h", "1d", "5d", "1wk", "1mo", "3mo"])]
    pub interval: String,
    // #[clap(subcommand)]
    // subcmd: SubCommand,
}

pub fn parse() -> Opts {
    let mut opts: Opts = Opts::parse();
    if opts.end.is_none() {
        let now = Local::now();
        opts.end = Some(NaiveDate::from_ymd(now.year(), now.month(), now.day()))
    }
    if let Some(end) = opts.end {
        if opts.start >= end {
            panic!("start date is greater or equal to end date")
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
