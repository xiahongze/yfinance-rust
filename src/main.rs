mod http;
mod options;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts = options::parse();
    let _ = http::download(opts).await;
}
