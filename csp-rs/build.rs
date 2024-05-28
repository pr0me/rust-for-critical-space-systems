fn main() {
    println!("cargo:rustc-link-search=../libcsp/builddir/");
    println!("cargo:rustc-link-lib=libcsp");

    println!("cargo:rerun-if-changed=wrapper.h");
}