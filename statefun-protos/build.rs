fn main() {
    prost_build::compile_protos(&["src/http-function.proto"], &["src/"]).unwrap();
}
