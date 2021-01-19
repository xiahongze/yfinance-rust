mod http;
mod options;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::builder().filter_level(log::LevelFilter::Info).init();
    let opts = options::parse();
    let _ = http::download(opts).await;
}
