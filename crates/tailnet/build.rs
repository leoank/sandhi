// Links libtailscale's static archive, which the flake builds and names in
// LIBTAILSCALE_LIB_DIR for the packages, the checks and the dev shell (ADR-0002). Nothing
// here runs `go`.
fn main() {
    let var = "LIBTAILSCALE_LIB_DIR";
    println!("cargo:rerun-if-env-changed={var}");
    let Ok(dir) = std::env::var(var) else {
        panic!("{var} must name the directory holding libtailscale; build inside the flake's dev shell, which sets it");
    };
    println!("cargo:rustc-link-search=native={dir}");
    println!("cargo:rustc-link-lib=static=tailscale");
    if std::env::var("TARGET").unwrap_or_default().contains("apple") {
        // Go's runtime and net package on darwin need these.
        println!("cargo:rustc-link-lib=framework=CoreFoundation");
        println!("cargo:rustc-link-lib=framework=Security");
    }
}
