pub mod http_function {
    include!(concat!(
        env!("OUT_DIR"),
        "/org.apache.flink.statefun.flink.core.polyglot.rs"
    ));
}
