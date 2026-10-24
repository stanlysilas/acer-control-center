fn main() {
    let lib_dir = std::env::var("ACERCONTROL_LIB_DIR")
        .unwrap_or_else(|_| "/usr/local/lib64".to_owned());
    let runtime_lib_dir = std::env::var("ACERCONTROL_RUNTIME_LIB_DIR")
        .unwrap_or_else(|_| lib_dir.clone());
    println!("cargo:rerun-if-env-changed=ACERCONTROL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ACERCONTROL_RUNTIME_LIB_DIR");
    println!("cargo:rustc-link-search=native={lib_dir}");
    println!("cargo:rustc-link-lib=acercontrol");
    println!("cargo:rustc-link-arg=-Wl,-rpath,{runtime_lib_dir}");
}
