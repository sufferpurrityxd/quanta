use std::{path::PathBuf, sync::Arc};

use log::info;
use quanta_http::run_http_server;
use quanta_network::QuantaNetwork;

use crate::{keypair_manager::load_or_generate_new_keypair, storage::load_or_create_new_database};

const QUANTA_APPLICATION_PATH_FOLDER_NAME: &str = ".quanta";
const QUANTA_HTTP_SERVER_ADDRS: (&str, u16) = ("127.0.0.1", 51255);

async fn configure_application_path() -> PathBuf {
    let application_path = home::home_dir()
        .expect("Failed to Get `HOME` Dirrectory")
        .join(QUANTA_APPLICATION_PATH_FOLDER_NAME);
    async_std::fs::create_dir_all(&application_path)
        .await
        .expect("Failed to create application dirrectory");
    application_path
}

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init_timed();
    let application_path = configure_application_path().await;

    let keypair = load_or_generate_new_keypair(&application_path).await;
    let local_peer_id = libp2p::PeerId::from(keypair.public());
    info!("LocalPeerId={}", local_peer_id);

    let storage = Arc::new(load_or_create_new_database(&application_path).await);
    info!("Creating QuantaNetwork Service for p2p communications");
    let (network, network_proxy) =
        QuantaNetwork::new(&keypair, local_peer_id, Arc::clone(&storage));

    tokio::spawn(async move {
        info!("Running QuantaNetwork Service in new thread");
        network
            .run_and_handle()
            .await
            .expect("QuantaNetwork finished with unexpected error")
    });

    info!(
        "Running HTTP-API Server on: {}",
        format!(
            "{}:{}",
            QUANTA_HTTP_SERVER_ADDRS.0, QUANTA_HTTP_SERVER_ADDRS.1
        )
    );
    run_http_server(
        QUANTA_HTTP_SERVER_ADDRS,
        Arc::clone(&storage),
        Arc::new(network_proxy),
    )
    .await?;

    Ok(())
}
