fn main() { prost_build::compile_protos(&["./src/swap_pb.proto"], &["./src"]).unwrap(); }
