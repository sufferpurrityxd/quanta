#![allow(clippy::unused_io_amount)]
mod keypair_manager;
mod run;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> { run::run().await }
