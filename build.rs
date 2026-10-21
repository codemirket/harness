#[path = "src/source_identity.rs"]
mod source_identity;
fn main() {
    let target = std::env::var("TARGET").unwrap();
    println!("cargo:rustc-env=MIRKET_TARGET={target}");
    for path in source_identity::INPUTS {
        println!("cargo:rerun-if-changed={path}");
    }
    println!(
        "cargo:rustc-env=MIRKET_EXECUTION_SHA256={}",
        source_identity::execution_fingerprint(std::path::Path::new(".")).unwrap()
    );
    println!(
        "cargo:rustc-env=MIRKET_SOURCE_SHA256={}",
        source_identity::fingerprint(std::path::Path::new(".")).unwrap()
    );
}
