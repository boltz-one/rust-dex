use std::env;

fn main() {
    if env::var_os("TRACING_FACADE").is_some() {
        println!("cargo::rustc-cfg=tracing_facade");
    }
    if env::var_os("TRACING_FACADE_WITH_MEMORY").is_some() {
        println!("cargo::rustc-cfg=tracing_facade");
        println!("cargo::rustc-cfg=tracing_facade_with_memory");
    }
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=TRACING_FACADE");
    println!("cargo::rerun-if-env-changed=TRACING_FACADE_WITH_MEMORY");
}
