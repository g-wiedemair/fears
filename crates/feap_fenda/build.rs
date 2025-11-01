fn main() {
    unsafe { std::env::set_var("MACOSX_DEPLOYMENT_TARGET", "11.0") };
    
    feap_binding::Build::new()
        // .files(["srcf/precision.f90", "srcf/f77_interface.f90", "srcf/fenda.f"])
        .file("srcf/foo.f90")
        // .link_lib_modifier("gfortran")
        // .compiler("/opt/homebrew/Cellar/flang/21.1.4/libexec/flang")
        .compile("fendaF");

    // cc::Build::new().file("srcf/foo.c").compile("fendaC");
}
