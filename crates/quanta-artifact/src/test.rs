use crate::{id::ArtifactId, magnet::MagnetLink};

#[test]
fn test_artifact_id() {
    let artifact_id = ArtifactId::new(b"beep boop");
    let artifact_id_to_string = artifact_id.to_string();
    let from_string = ArtifactId::from_bs58_string(artifact_id_to_string.as_str()).unwrap();
    assert_eq!(artifact_id, from_string);
}

#[test]
fn test_magnet_link() {
    let mut magnet = MagnetLink::new("hello".to_string(),  5000);
    magnet.new_update_with_artifact_id(ArtifactId::new(b"beep"));
    magnet.new_update_with_artifact_id(ArtifactId::new(b"boop"));
    let string_magnet = magnet.to_string();
    let from_string_magnet = MagnetLink::try_from(string_magnet).unwrap();

    assert_eq!(magnet, from_string_magnet);
}
