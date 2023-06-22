#![allow(dead_code)]
mod ah;
mod hv;
#[cfg(test)]
mod test;

pub(crate) const HASH_SIZE: usize = 32;
pub(crate) type Hash = [u8; HASH_SIZE];

pub use {
    ah::AdvancedHasher,
    hv::{HashValue, HashValueError},
};
