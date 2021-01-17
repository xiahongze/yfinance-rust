mod options;
mod http;

fn main() {
    // let start = NaiveDate::parse_from_str("2020-01-17", "%Y-%m-%d");
    // println!("Hello, world!, {:?}", start);
    let opts = options::parse();
    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("+++++++++++++++++++++++++++");
    println!("Value for config: {}", opts.config);
    println!("Using start: {:?}", opts.start.and_hms(0, 0,0).timestamp());
    println!("Using end: {:?}", opts.end);
    println!("Using symbols: {:?}", opts.symbols);
    println!("Using adjusted close: {:?}", opts.adjusted_close);
    // http::download(opts);
}
