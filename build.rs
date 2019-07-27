// build.rs

extern crate fs_extra;
extern crate foreman;
extern crate bindgen;

use std::process::Command;
use fs_extra::dir::{copy, CopyOptions};
use foreman::{LibKind, SearchKind};


fn main() {
    let out_dir = foreman::out_dir().unwrap();
    let valgrind_dir = out_dir.join("valgrind");
    let vex_dir = valgrind_dir.join("VEX");

    {
        // Copy and build valgrind in OUT_DIR
        let options = CopyOptions { overwrite: true, skip_exist: true, buffer_size: 1<<16 };
        copy("valgrind", &out_dir, &options).unwrap();

        if !valgrind_dir.join("configure").exists() {
            Command::new("./autogen.sh")
                .current_dir(&valgrind_dir)
                .status().unwrap();
        }
        if !vex_dir.join("Makefile").exists() {
            Command::new("./configure")
                .current_dir(&valgrind_dir)
                .status().unwrap();
        }
        Command::new("make").args(&["-j", &format!("{}", foreman::num_jobs().unwrap())])
            .current_dir(&vex_dir)
            .status().unwrap();
    }

    {
        // Tell rustc to link to libvex
        let host = foreman::host().unwrap();
        let mut host_parts = host.as_str().split("-");
        let arch = host_parts.next().unwrap();
        let arch = if arch == "x86_64" { "amd64" } else { arch };
        let _ = host_parts.next();
        let platform = host_parts.next().unwrap();

        foreman::link_search(SearchKind::Native, &vex_dir);
        foreman::link_lib(LibKind::Static, &format!("vex-{}-{}", arch, platform));
    }

    {
        // Generate bindings
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .blacklist_type("_IRStmt__bindgen_ty_1__bindgen_ty_1")
            .clang_arg(&format!("-I{}", &valgrind_dir.to_str().unwrap()))
            .generate()
            .expect("Unable to generate bindings");
        bindings.write_to_file(out_dir.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
