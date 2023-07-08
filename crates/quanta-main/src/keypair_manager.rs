use std::path::{Path, PathBuf};

use async_std::{fs::OpenOptions, io::ReadExt};
use libp2p::{futures::AsyncWriteExt, identity::Keypair};
use log::{error, info};

/// Path-str of folder where [Keypair] are stored
const KEYPAIR_STORE_JOIN_PATH: &str = "keys";
/// Name of [Keypair] file
const KEYPAIR_PATH: &str = "keypair.ser";

/// Load [Keypair] from given application_path or generate new and save it
pub async fn load_or_generate_new_keypair<P: AsRef<Path>>(application_path: P) -> Keypair {
    let application_path_ref = application_path.as_ref();

    let keypair_store_path = application_path_ref.join(KEYPAIR_STORE_JOIN_PATH);
    let keypair_file_path = keypair_store_path.join(KEYPAIR_PATH);

    match async_std::fs::File::open(keypair_file_path.clone()).await {
        Ok(file) => load_from_file(file, keypair_file_path).await,
        Err(why) => {
            error!("Got error when trying to open KeyPair file: {}", why);
            generate_and_save_new_keypair(keypair_file_path).await
        },
    }
}

async fn load_from_file(mut file: async_std::fs::File, keypair_file_path: PathBuf) -> Keypair {
    let mut buf = [0; 32];

    match ReadExt::read_exact(&mut file, &mut buf).await {
        Ok(_) => match Keypair::ed25519_from_bytes(&mut buf) {
            Ok(keypair) => {
                info!("Loaded keypair from: {:?}", keypair_file_path);
                keypair
            },
            Err(why) => {
                error!(
                    "DecodingError when trying to get KeyPair from file: {}",
                    why
                );
                generate_and_save_new_keypair(keypair_file_path).await
            },
        },
        Err(why) => {
            error!("Got error when trying to read KeyPair file: {}", why);
            generate_and_save_new_keypair(keypair_file_path).await
        },
    }
}

async fn generate_and_save_new_keypair(keypair_file_path: PathBuf) -> Keypair {
    info!("Creating new keypair in: {:?}", keypair_file_path);
    let keypair = Keypair::generate_ed25519();

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(keypair_file_path)
        .await
        .expect("Failed to create new KeyPair file");

    file.write(
        keypair
            .to_protobuf_encoding()
            .expect("Failed to decode KeyPair into Protobuf encoding")
            .as_slice(),
    )
    .await
    .expect("Failed to Write KeyPair into file");
    keypair
}
