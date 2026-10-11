fn main() {
    println!(
        "cargo:rustc-env=MIRKET_TARGET={}",
        std::env::var("TARGET").unwrap()
    );
    for path in [
        "skills",
        "registry",
        "instructions",
        "docs/third-party-notices.md",
        "docs/licenses/packages",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }
}
