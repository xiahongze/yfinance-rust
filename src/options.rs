use chrono::{NaiveDate, Local, Datelike};
use clap::Clap;
// https://query1.finance.yahoo.com/v7/finance/download/GXY.AX?period1=1579236638&period2=1610859038&interval=1d&events=history&includeAdjustedClose=true
#[derive(Clap)]
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
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    pub config: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: i32,
    // #[clap(subcommand)]
    // subcmd: SubCommand,
}

pub fn parse() -> Opts {
    let mut opts: Opts = Opts::parse();
    if opts.end.is_none() {
        let now = Local::now();
        opts.end = Some(NaiveDate::from_ymd(now.year(),now.month(), now.day()))
    }
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

