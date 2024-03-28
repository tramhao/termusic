use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=static=libsonic");
    println!("cargo:rerun-if-changed=sonic/sonic.h");
    println!("cargo:rerun-if-changed=sonic/sonic.c");

    cc::Build::new()
        .file("sonic/sonic.c")
        .include("sonic/sonic.h")
        .compile("libsonic");

    let bindings = bindgen::Builder::default()
        .header("sonic/sonic.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
