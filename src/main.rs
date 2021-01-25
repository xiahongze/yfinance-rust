mod http;
mod options;
mod v8chart;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts = options::parse();
    let _ = http::download(opts).await;
}
