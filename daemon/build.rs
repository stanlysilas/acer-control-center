use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let library_dir = env::var_os("ACERCONTROL_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&manifest_dir).join("../lib/build"));
    let runtime_library_dir = env::var_os("ACERCONTROL_RUNTIME_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| library_dir.clone());

    println!("cargo:rerun-if-env-changed=ACERCONTROL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ACERCONTROL_RUNTIME_LIB_DIR");
    println!("cargo:rustc-link-search=native={}", library_dir.display());
    println!("cargo:rustc-link-lib=dylib=acercontrol");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{}", runtime_library_dir.display());
    println!("cargo:rerun-if-changed=../lib/build/libacercontrol.so");
}
