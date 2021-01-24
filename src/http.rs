use crate::options::Opts;

use std::path::Path;
use std::sync::Arc;
use tokio::fs::File;
use tokio::{io, time::sleep};
// Needed for the stream conversion
use futures::stream::{StreamExt, TryStreamExt};
use hyper::{
    body::{to_bytes, Bytes},
    Body, Response, StatusCode,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
struct DownloadError {
    status: StatusCode,
    symbol: String,
    body: Bytes,
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DownloadError {}

// https://query1.finance.yahoo.com/v7/finance/download/GXY.AX?period1=1579236638&period2=1610859038&interval=1d&events=history&includeAdjustedClose=true
// #[allow(dead_code)]
pub async fn download(opts: Opts) -> Vec<Result<()>> {
    let out_dir = Path::new(&opts.output_dir);
    if !out_dir.exists() {
        // try to create a directory
        match std::fs::create_dir(out_dir) {
            Err(err) => {
                error!("failed to create directory at {:?} with error {:?}", out_dir, err);
                return vec![];
            }
            _ => {}
        }
    }

    let https = hyper_tls::HttpsConnector::new();
    let client_arc = Arc::new(hyper::Client::builder().build::<_, hyper::Body>(https));

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
        sleep(opts.rate.duration).await;
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
                status => Err(DownloadError {
                    status,
                    symbol: symb.to_owned(),
                    body: to_bytes(resp.body_mut()).await?,
                }
                .into()),
            }
        };
        tasks.push(task);
    }
    let total = tasks.len();
    let results = futures::future::join_all(tasks).await;
    let success = results
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

    return results;
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
    let base = format!("https://query1.finance.yahoo.com/v8/finance/chart/{}", symbol);
    let start = opts.start.and_hms(0, 0, 0).timestamp().to_string();
    let end = opts.end.unwrap().and_hms(0, 0, 0).timestamp().to_string();
    let url = url::Url::parse_with_params(
        base.as_str(),
        &[
            ("period1", start.as_str()),
            ("period2", end.as_str()),
            ("interval", opts.interval.as_str()),
            ("events", "div,split"),
        ],
    )
    .unwrap();
    debug!("{}", url.as_str());
    url.into_string().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::download;
    use crate::options::Opts;
    use chrono::NaiveDate;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn make_opts() -> Opts {
        Opts {
            symbols: vec!["GXY.AX".to_string(), "A2M.AX".to_string()],
            start: NaiveDate::from_ymd(2020, 1, 3),
            end: Some(NaiveDate::from_ymd(2020, 1, 5)),
            include_pre_post: true,
            verbose: 0,
            output_dir: "./target/output".to_string(),
            interval: "1d".to_string(),
            rate: "500".parse().unwrap(),
        }
    }

    /// with `tokio::test`, we don't need the std test macro and we can use async functions
    #[tokio::test]
    async fn test_download_success() {
        init();
        let opts = make_opts();
        assert_eq!(download(opts).await.iter().filter_map(|r| r.as_ref().ok()).count(), 2);
    }

    #[tokio::test]
    async fn test_download_fail() {
        init();
        let mut opts = make_opts();
        opts.start = "2020-01-06".into();
        assert_eq!(download(opts).await.iter().filter_map(|r| r.as_ref().ok()).count(), 0);
    }
}
