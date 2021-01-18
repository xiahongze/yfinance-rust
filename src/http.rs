use crate::options::Opts;

// https://query1.finance.yahoo.com/v7/finance/download/GXY.AX?period1=1579236638&period2=1610859038&interval=1d&events=history&includeAdjustedClose=true
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use std::fs::File;
use std::io::Write;

// #[allow(dead_code)]
pub async fn download(opts: Opts) -> Result<()> {
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    // let client = hyper::Client::new();
    for symb in opts.symbols.iter() {
        let base = format!("https://query1.finance.yahoo.com/v7/finance/download/{}", symb);
        let start = opts.start.and_hms(0, 0, 0).timestamp().to_string();
        let end = opts.end.unwrap().and_hms(0, 0, 0).timestamp().to_string();
        let url = url::Url::parse_with_params(
            base.as_str(),
            &[
                ("period1", start.as_str()),
                ("period2",end.as_str()),
                ("includeAdjustedClose", opts.adjusted_close.to_string().as_str()),
                ("events", "history"),
                ("interval", "1d"),
            ],
        )
        .unwrap();
        println!("{}", url.as_str());
        let uri = url.into_string().parse()?;
        let resp = client.get(uri).await?;
        println!("headers are {:?}", resp.headers());
        println!("status is {:?}", resp.status());
        let body = hyper::body::to_bytes(resp).await?;
        println!("body len is {}, {:?}...", body.len(), body);
        {
            let mut file = File::create("test.csv")?;
            // Write a slice of bytes to the file
            file.write(&body)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{download, Opts};
    use chrono::NaiveDate;
    use tokio_test::*;
    // #[test]
    // fn test2() {
    //     println!("++++++++++++++++++++++");
    //     assert!(true);
    // }

    #[test]
    fn test1() {
        println!("+++++++++++++++++++++");
        let opts = Opts {
            symbols: vec!["GXY.AX".to_string()],
            start: NaiveDate::from_ymd(2020, 1, 1),
            end: Some(NaiveDate::from_ymd(2020, 1, 2)),
            adjusted_close: false,
            config: "".to_string(),
            verbose: 0,
        };

        assert_ok!(block_on(download(opts)));
    }
}
