use async_std::path::Path;
use log::info;
use quanta_database::Database;

const QUANTA_STORAGE_FOLDER_NAME: &str = "storage";

pub async fn load_or_create_new_database<P: AsRef<Path>>(application_path: P) -> Database {
    let application_path_ref = application_path.as_ref();
    let storage_file_path = application_path_ref.join(QUANTA_STORAGE_FOLDER_NAME);
    info!(
        "Open or Create QuantaDatabase in: {:?}",
        storage_file_path
    );
    Database::new(storage_file_path).expect("Failed to open QuantaDatabase")
}
