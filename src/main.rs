use anyhow::Ok;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::{
    env,
    time::{Duration, Instant},
};

mod error;
pub use error::Error;
mod model;
mod ports;
mod subdomains;
use model::Subdomain;
mod common_ports;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        return Err(Error::CliUsage.into());
    }

    let target = args[1].as_str();

    let http_timeout = Duration::from_secs(5);
    let http_client = Client::builder().timeout(http_timeout).build()?;

    let ports_concurrency = 200;
    let subdomains_concurrency = 100;
    let scan_start = Instant::now();
    let subdomains = subdomains::enumerate(&http_client, target).await?;

    // Concurrent stream method 1: Using buffer_unordered + collect
    let scan_result: Vec<Subdomain> = stream::iter(subdomains.into_iter())
        .map(|subdomain| ports::scan_ports(ports_concurrency, subdomain))
        .buffer_unordered(subdomains_concurrency)
        .collect()
        .await;
    // let scan_start = Instant::now();

    // // using a custom thread pool
    // let pool = rayon::ThreadPoolBuilder::new()
    //     .num_threads(256)
    //     .build()
    //     .unwrap();

    // // pool.install is required to use the custom threadpool
    // pool.install(|| {
    //     let scan_result: Vec<Subdomain> = subdomains::enumerate(&http_client, target)
    //         .unwrap()
    //         .into_par_iter()
    //         .map(ports::scan_ports)
    //         .collect();

    //     for subdomain in scan_result {
    //         println!("{}:", &subdomain.domain);
    //         for port in &subdomain.open_ports {
    //             println!("      {}", port.port);
    //         }
    //         println!();
    //     }
    // });
    let scan_duration = scan_start.elapsed();
    println!("Scan completed in {:?}", scan_duration);

    for subdomain in scan_result {
        println!("{}:", &subdomain.domain);
        for port in &subdomain.open_ports {
            println!("    {}: open", port.port);
        }

        println!("");
    }
    Ok(())
}
