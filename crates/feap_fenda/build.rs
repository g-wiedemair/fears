use cmake::Config;
use std::path::PathBuf;

fn main() {
    let dst = Config::new(".").generator_toolset("fortran=ifx").build();

    println!("cargo:rustc-link-search={}", dst.display());
    println!("cargo:rustc-link-lib=foo");

    let fc_lib_pwd = PathBuf::from("C:/Program Files (x86)/Intel/oneAPI/compiler/latest/lib");
    println!("cargo:rustc-link-search={}", fc_lib_pwd.to_str().unwrap());
}
