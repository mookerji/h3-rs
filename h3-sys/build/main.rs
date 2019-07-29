fn configure() {
    println!("cargo:rustc-link-lib=h3.1");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
}

fn main() {
    configure()
}
