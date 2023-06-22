#![allow(dead_code)]
mod behaviour;
mod codec;
mod protobuffable;
mod protocol;
mod request;
mod response;
mod searchid;
#[cfg(test)]
mod test;

pub use behaviour::{Behaviour, Event, Storage};

mod swap_pb {
    include!(concat!(env!("OUT_DIR"), "/swap_pb.rs"));
}
