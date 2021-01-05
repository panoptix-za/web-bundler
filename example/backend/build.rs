use std::path::PathBuf;
use std::process;
use web_bundler::WebBundlerOpt;

fn main() {
    let out_dir =
        PathBuf::from(std::env::var("OUT_DIR").expect("expected OUT_DIR to be set by Cargo"));

    let opt = WebBundlerOpt {
        src_dir: PathBuf::from("../frontend"),
        dist_dir: out_dir.join("ui"),
        tmp_dir: out_dir.join("tmp"),
        base_url: Some("/".into()),
        wasm_version: std::env::var("CARGO_PKG_VERSION")
            .expect("expected CARGO_PKG_VERSION to be set by Cargo"),
        release: std::env::var("PROFILE").expect("expected PROFILE to be set by Cargo") != "debug",
        workspace_root: PathBuf::from(".."),
        additional_watch_dirs: Vec::new(),
    };
    match web_bundler::run(opt) {
        Ok(()) => {}
        Err(e) => {
            println!("Failed to build frontend. Error: {}", e);
            process::exit(1);
        }
    }
}
