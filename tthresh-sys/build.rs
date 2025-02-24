#![allow(missing_docs)] // FIXME
#![allow(clippy::expect_used)] // FIXME
#![allow(clippy::unwrap_used)] // FIXME
#![allow(clippy::panic)] // FIXME

use std::{
    env,
    path::{Path, PathBuf},
};

fn main() {
    // use cmake to configure (but not compile) the tthresh build
    let mut config = cmake::Config::new("tthresh");
    config.define("BUILD_SHARED_LIBS", "OFF");
    config.define("BUILD_TESTING", "OFF");
    config.build_arg("--version");

    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=tthresh");

    let cargo_callbacks = bindgen::CargoCallbacks::new();
    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++11")
        .clang_arg(format!("-I{}", Path::new("tthresh").join("src").display()))
        .clang_arg(format!(
            "-I{}",
            Path::new("tthresh").join("external").display()
        ))
        .header("wrapper.hpp")
        .parse_callbacks(Box::new(cargo_callbacks))
        .allowlist_function("my_main")
        // MSRV 1.82
        .rust_target(match bindgen::RustTarget::stable(82, 0) {
            Ok(target) => target,
            Err(err) => panic!("{err}"),
        })
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let mut build = cc::Build::new();

    build
        .cpp(true)
        .std("c++11")
        .include(Path::new("tthresh").join("src"))
        .include(Path::new("tthresh").join("external"))
        .file("lib.cpp")
        .warnings(false);

    if cfg!(feature = "openmp") {
        env::var("DEP_OPENMP_FLAG") // set by openmp-sys
            .unwrap()
            .split(' ')
            .for_each(|f| {
                build.flag(f);
            });
    }

    build.compile("tthresh");

    if cfg!(feature = "openmp") {
        if let Some(link) = env::var_os("DEP_OPENMP_CARGO_LINK_INSTRUCTIONS") {
            for i in env::split_paths(&link) {
                if i.as_os_str().is_empty() {
                    continue;
                }
                println!("cargo:{}", i.display());
            }
        }
    }
}
