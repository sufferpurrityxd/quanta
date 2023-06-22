use crate::id::ArtifactId;

#[test]
fn test_artifact_id() {
    let artifact_id = ArtifactId::new(b"beep boop");
    let artifact_id_to_string = artifact_id.to_string();
    let from_string = ArtifactId::from_bs58_string(artifact_id_to_string.as_str()).unwrap();
    assert_eq!(artifact_id, from_string);
}
