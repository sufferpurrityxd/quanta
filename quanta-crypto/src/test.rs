use {
    crate::{ah::AdvancedHasher, hv::HashValue},
    digest::Digest,
};

fn get_hashvalue() -> HashValue {
    AdvancedHasher::new(b"beep boop", sha3::Sha3_256::new())
        .update_digest(sha3::Keccak256::new())
        .update_digest(sha3::Sha3_256::new())
        .finalize()
}

#[test]
fn test_advanced_hasher_to_string() {
    let hash_value = get_hashvalue();
    assert_eq!(
        "c20fa28614f2f2e5d69cbe82679de08169c015e6ae55b83b6ed030e7bfd8c77d",
        hash_value.to_string()
    );
}

#[test]
fn test_hashvalue_from_string() {
    assert_eq!(
        HashValue::try_from("c20fa28614f2f2e5d69cbe82679de08169c015e6ae55b83b6ed030e7bfd8c77d")
            .unwrap(),
        get_hashvalue()
    )
}

#[test]
fn test_advanced_hasher_to_bytes() {
    let hash_value = get_hashvalue();
    assert_eq!(
        [
            194, 15, 162, 134, 20, 242, 242, 229, 214, 156, 190, 130, 103, 157, 224, 129, 105, 192,
            21, 230, 174, 85, 184, 59, 110, 208, 48, 231, 191, 216, 199, 125
        ]
        .to_vec(),
        hash_value.to_bytes(),
    )
}

#[test]
fn test_hashvalue_from_bytes() {
    assert_eq!(
        HashValue::try_from(
            [
                194, 15, 162, 134, 20, 242, 242, 229, 214, 156, 190, 130, 103, 157, 224, 129, 105,
                192, 21, 230, 174, 85, 184, 59, 110, 208, 48, 231, 191, 216, 199, 125
            ]
            .as_slice()
        )
        .unwrap(),
        get_hashvalue()
    )
}

#[test]
fn test_hasher_blake() {
    let hashvalue = AdvancedHasher::new_with_blake3(b"beep").finalize();
    assert_eq!(
        "18661258f79d57e2901b0c5dda935cfd7807d0501af0fbd0d07608573639ff8b",
        hashvalue.to_string()
    );
}
