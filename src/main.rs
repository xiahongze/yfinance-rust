use options::SubCommand;

mod http;
mod options;
mod v8chart;
#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts = options::parse();
    let _ = match opts.subcmd {
        SubCommand::Download(opts) => {
            if let Some(start) = opts.start {
                if let Some(end) = opts.end {
                    if start >= end {
                        panic!("start date is greater or equal to end date")
                    }
                }
            }
            http::download(opts).await
        }
    };
}
