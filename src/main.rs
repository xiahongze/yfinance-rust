use std::fs::read_dir;

use options::SubCommand;
use v8chart::{load_from_json, write_to_csv, DataSet};

mod http;
mod options;
mod v8chart;
#[macro_use]
extern crate log;

fn convert_to_csv(json_dir: &String) -> std::io::Result<()> {
    read_dir(json_dir)?
        .map(|res| res.map(|e| e.path()))
        .filter_map(|res| res.ok())
        .filter(|path| path.extension().map_or(false, |ext| ext == "json"))
        .for_each(|path| match load_from_json(path.to_str().unwrap()) {
            Ok(chart_wrapper) => {
                let ds_vec: Vec<DataSet> = chart_wrapper.chart.into();

                let outputs = if ds_vec.len() == 1 {
                    vec![path.with_extension("csv")]
                } else {
                    let stem = path
                        .file_stem()
                        .map_or("unknown_stem", |s| s.to_str().unwrap())
                        .to_string();
                    (0..ds_vec.len())
                        .map(|i| path.with_file_name(format!("{:?}_{}.csv", stem, i)))
                        .collect()
                };

                ds_vec.iter().zip(outputs.iter()).for_each(|(ds, path)| {
                    if let Err(err) = write_to_csv(ds, path) {
                        error!("failed to write to csv {:?} with {:?}", path, err);
                    } else {
                        info!("successfully converted to {:?}", path);
                    }
                });
            }
            Err(err) => error!("failed to load json from {:?} with {:?}", path, err),
        });
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let opts = options::parse();
    let (json_dir, convert) = match opts.subcmd {
        SubCommand::Download(opts) => {
            if let Some(start) = opts.start {
                if let Some(end) = opts.end {
                    if start >= end {
                        panic!("start date is greater or equal to end date")
                    }
                }
            }
            let _ = http::download(&opts).await;
            (opts.output_dir, opts.convert)
        }
        SubCommand::Convert(opts) => (opts.input_dir, true),
    };
    if convert {
        match convert_to_csv(&json_dir) {
            Err(err) => error!("failed to walk dir {} with {:?}", json_dir, err),
            _ => {}
        }
    }
}
