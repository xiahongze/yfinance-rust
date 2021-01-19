use crate::options::Opts;

use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io;
// Needed for the stream conversion
use futures::stream::{StreamExt, TryStreamExt};
use hyper::{
    body::{to_bytes, Bytes},
    Body, Response, StatusCode,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
enum DownloadError {
    Other(StatusCode, String, Bytes),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadError::Other(code, symb, bytes) => {
                write!(f, "status: {:?}, symbol: {}, resp: {:?}", code, symb, bytes)
            }
        }
    }
}

impl std::error::Error for DownloadError {}

// https://query1.finance.yahoo.com/v7/finance/download/GXY.AX?period1=1579236638&period2=1610859038&interval=1d&events=history&includeAdjustedClose=true
// #[allow(dead_code)]
pub async fn download(opts: Opts) -> Result<()> {
    let https = hyper_tls::HttpsConnector::new();
    let client_arc = Arc::new(hyper::Client::builder().build::<_, hyper::Body>(https));
    // let opts_arc = Arc::new(opts);
    // let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let out_dir = Path::new(&opts.output_dir);
    if !out_dir.exists() {
        // try to create a directory
        std::fs::create_dir(out_dir)?;
    }
    let mut tasks = Vec::new();
    // let client = hyper::Client::new();
    for symb in opts.symbols.iter() {
        let client = client_arc.clone();
        let filename = format!(
            "{}_{}_{}.csv",
            symb,
            opts.start.format("%Y%m%d"),
            opts.end.unwrap().format("%Y%m%d")
        );
        let uri = make_uri(&opts, symb);
        let pathbuf = out_dir.join(filename);
        let task = async move {
            let mut resp = client.get(uri).await?;
            debug!(
                "content type: {:?}, status: {:}",
                resp.headers().get("content-type"),
                resp.status()
            );

            match resp.status() {
                StatusCode::OK => write_to_file(resp, pathbuf.as_path()).await,
                // std lib provide to convert to Box
                // handle errors here
                code => Err(DownloadError::Other(code, symb.to_owned(), to_bytes(resp.body_mut()).await?).into()),
            }
        };
        tasks.push(task);
    }
    let total = tasks.len();
    let success = futures::future::join_all(tasks)
        .await
        .iter()
        .map(|r| match r {
            Ok(_) => 1,
            Err(e) => {
                error!("encounter error: {:?}", e);
                0
            }
        })
        .fold(0, |acc, x| acc + x);
    info!("have successfully download {} of {}", success, total);
    Ok(())
}

/**
 * https://stackoverflow.com/questions/60964238/how-to-write-a-hyper-response-body-to-a-file
 * had to use into_body() because after consuming the body resp is not going to be used
 * if using body(), a ref is used but resp is moved so it won't compile
 *
 * Latest update: mut resp and use body_mut()
 *
 * */
async fn write_to_file(mut resp: Response<Body>, path: &Path) -> Result<()> {
    let futures_io_async_read = resp
        .body_mut()
        .map(|result| result.map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string())))
        .into_async_read();
    let mut tokio_async_read = tokio_util::compat::FuturesAsyncReadCompatExt::compat(futures_io_async_read);

    let mut file = File::create(path).await?;
    io::copy(&mut tokio_async_read, &mut file).await?;
    info!("downloaded {:?}", path);
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
            ("interval", opts.interval.as_str()),
        ],
    )
    .unwrap();
    debug!("{}", url.as_str());
    url.into_string().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::{download, Opts};
    use chrono::NaiveDate;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// with `tokio::test`, we don't need the std test macro and we can use async functions
    #[tokio::test]
    async fn test_download() {
        init();
        let opts = Opts {
            symbols: vec!["GXY.AX".to_string()],
            start: NaiveDate::from_ymd(2020, 1, 1),
            end: Some(NaiveDate::from_ymd(2020, 1, 2)),
            adjusted_close: false,
            verbose: 0,
            output_dir: "./target/output".to_string(),
            interval: "1d".to_string(),
        };

        assert!(download(opts).await.is_ok());
    }
}
