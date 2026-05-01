#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

use std::path::Path;

fn main() {
    let windows_icon = Path::new("../../assets/windows/app-icon.ico");
    let windows_resource = Path::new("../../assets/windows/boltz.rc");

    println!("cargo:rerun-if-changed={}", windows_icon.display());
    println!("cargo:rerun-if-changed={}", windows_resource.display());

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile(
            windows_resource,
            embed_resource::ParamsIncludeDirs([Path::new("../../assets/windows")]),
        )
        .manifest_optional()
        .unwrap();
    }
}
