use crate::options::Opts;

// https://query1.finance.yahoo.com/v7/finance/download/GXY.AX?period1=1579236638&period2=1610859038&interval=1d&events=history&includeAdjustedClose=true
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

use tokio::fs::File;
use tokio::io;
// Needed for the stream conversion
use futures::stream::{StreamExt, TryStreamExt};
use hyper::{Body, Response};

// #[allow(dead_code)]
pub async fn download(opts: Opts) -> Result<()> {
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    // let client = hyper::Client::new();
    for symb in opts.symbols.iter() {
        let resp = client.get(make_uri(&opts, symb)).await?;
        info!("content type: {:?}", resp.headers().get("content-type"));
        info!("status: {:?}", resp.status());
        write_to_file(resp).await?;
    }
    Ok(())
}

/**
 * https://stackoverflow.com/questions/60964238/how-to-write-a-hyper-response-body-to-a-file
 * had to use into_body() because after consuming the body resp is not going to be used
 * if using body(), a ref is used but resp is moved so it won't compile
 * */
async fn write_to_file(resp: Response<Body>) -> Result<()> {
    let futures_io_async_read = resp
        .into_body()
        .map(|result| result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string())))
        .into_async_read();
    let mut tokio_async_read = tokio_util::compat::FuturesAsyncReadCompatExt::compat(futures_io_async_read);

    let mut file = File::create("test.csv").await?;
    io::copy(&mut tokio_async_read, &mut file).await?;
    Ok(())
}

fn make_uri(opts: &Opts, symbol: &String) -> hyper::Uri {
    let base = format!("https://query1.finance.yahoo.com/v7/finance/download/{}", symbol);
    let start = opts.start.and_hms(0, 0, 0).timestamp().to_string();
    let end = opts.end.unwrap().and_hms(0, 0, 0).timestamp().to_string();
    let url = url::Url::parse_with_params(
        base.as_str(),
        &[
            ("period1", start.as_str()),
            ("period2", end.as_str()),
            ("includeAdjustedClose", opts.adjusted_close.to_string().as_str()),
            ("events", "history"),
            ("interval", "1d"),
        ],
    )
    .unwrap();
    println!("{}", url.as_str());
    url.into_string().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::{download, Opts};
    use chrono::NaiveDate;
    use tokio_test::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
    // #[test]
    // fn test2() {
    //     println!("++++++++++++++++++++++");
    //     assert!(true);
    // }

    #[test]
    fn test1() {
        init();
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
