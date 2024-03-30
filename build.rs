// build.rs
fn main() {
    cc::Build::new()
        .file("calibre/palmdoc.c")
        .include("calibre")
        .compile("palmdoc");
    println!("cargo:rerun-if-changed=calibre/palmdoc.c");
    println!("cargo:rerun-if-changed=calibre/palmdoc.h");
}
