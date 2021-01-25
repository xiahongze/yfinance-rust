use crate::options::Opts;

use chrono::Local;
use std::path::{Path, PathBuf};
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

pub async fn download(opts: Opts) -> Vec<(PathBuf, Result<()>)> {
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
    let mut paths = Vec::new();
    // let client = hyper::Client::new();
    for symb in opts.symbols.iter() {
        let client = client_arc.clone();
        let filename = format!(
            "{}_{}_{}.json",
            symb,
            opts.start
                .map_or("init".to_string(), |s| s.format("%Y%m%d").to_string()),
            opts.end
                .unwrap_or(Local::now().naive_local().date())
                .format("%Y%m%d")
                .to_string(),
        );
        let uri = make_uri(&opts, symb);
        let pathbuf = out_dir.join(filename);
        paths.push(pathbuf.clone());
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

    paths.into_iter().zip(results).collect()
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
    let start = opts
        .start
        .map_or("0".to_string(), |s| s.and_hms(0, 0, 0).timestamp().to_string());
    let end = opts.end.map_or("99999999999".to_string(), |s| {
        s.and_hms(0, 0, 0).timestamp().to_string()
    });
    let url = url::Url::parse_with_params(
        base.as_str(),
        &[
            ("period1", start),
            ("period2", end),
            ("interval", opts.interval.to_owned()),
            ("events", "div,split".to_string()),
        ],
    )
    .unwrap();
    debug!("{}", url.as_str());
    url.into_string().parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Opts;
    use chrono::NaiveDate;
    use std::{fs::remove_file, path::PathBuf};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn make_opts() -> Opts {
        Opts {
            symbols: vec!["GXY.AX".to_string(), "A2M.AX".to_string()],
            start: Some(NaiveDate::from_ymd(2020, 1, 3)),
            end: Some(NaiveDate::from_ymd(2020, 1, 7)),
            include_pre_post: true,
            verbose: 0,
            output_dir: "./target/output".to_string(),
            interval: "1d".to_string(),
            rate: "500".parse().unwrap(),
        }
    }

    fn assert_remove(path_results: Vec<(PathBuf, Result<()>)>, count: usize) {
        assert_eq!(path_results.iter().filter_map(|(_, r)| r.as_ref().ok()).count(), count);
        // remove temperorary files
        path_results.iter().for_each(|(p, _)| remove_file(p.as_path()).unwrap());
    }

    /// with `tokio::test`, we don't need the std test macro and we can use async functions
    #[tokio::test]
    async fn test_download_success() {
        init();
        let opts = make_opts();
        let path_results = download(opts).await;
        // assert_remove(path_results, 2);
    }

    #[tokio::test]
    async fn test_download_fail() {
        init();
        let mut opts = make_opts();
        opts.start = Some(NaiveDate::from_ymd(2020, 1, 6));
        let path_results = download(opts).await;
        assert_remove(path_results, 0);
    }

    #[tokio::test]
    async fn test_optional_start_end() {
        init();
        let mut opts = make_opts();
        opts.start = None;
        opts.end = None;
        let path_results = download(opts).await;
        assert_remove(path_results, 2);
    }
}
