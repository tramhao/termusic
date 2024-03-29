//Partially borrowed from sdl2-sys

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn compile_soundtouch(src: impl AsRef<Path>) -> PathBuf {
    let mut cfg = cmake::Config::new(src);
    cfg.profile("release");
    cfg.define("SOUNDSTRETCH", "OFF");

    cfg.build()
}

// fn init_submodule(submodule_path: impl AsRef<Path>) {
//     let submodule_path = submodule_path.as_ref();
//     if !submodule_path.join("CMakeLists.txt").exists() {
//         Command::new("git")
//             .args(["submodule", "update", "--init"])
//             .current_dir(submodule_path)
//             .status()
//             .expect("Git is needed to retrieve the soundtouch source files");
//     }
// }

fn main() {
    let src = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("soundtouch");
    // init_submodule(&src);
    let compiled_path = compile_soundtouch(&src);

    println!(
        "cargo:rustc-link-search={}",
        compiled_path.join("lib64").display()
    );
    println!(
        "cargo:rustc-link-search={}",
        compiled_path.join("lib").display()
    );

    let includes = src.join("include").to_str().unwrap().to_string();
    println!("cargo:include={}", includes);

    let bindings = bindgen::builder()
        .header("wrapper.hpp")
        .enable_cxx_namespaces()
        .respect_cxx_access_specs(true)
        .allowlist_type("soundtouch::SoundTouch")
        .clang_arg(format!("-I{}", includes))
        .generate()
        .expect("Failed");

    println!("cargo:rustc-link-lib=static=SoundTouch");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    if target_os == "macos" {
        println!("cargo:rustc-link-lib=c++");
    } else if target_os != "windows" {
        println!("cargo:rustc-link-lib=stdc++");
    }

    let out_path =
        PathBuf::from(env::var("OUT_DIR").expect("environment variable `OUT_DIR' exists"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed writing bindings.rs")
}
