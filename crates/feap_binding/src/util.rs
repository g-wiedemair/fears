use crate::{
    error::{Error, ErrorKind},
    target::TargetInfo,
};
use std::{
    ffi::OsStr,
    fmt::{self, Write},
    path::Path,
    process::Command,
    sync::{atomic::AtomicU8, atomic::Ordering::Relaxed},
};

pub(super) struct JoinOsStrs<'a, T> {
    pub(super) slice: &'a [T],
    pub(super) delimiter: char,
}

impl<T> fmt::Display for JoinOsStrs<'_, T>
where
    T: AsRef<OsStr>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.slice.len();
        for (index, os_str) in self.slice.iter().enumerate() {
            // TODO: Use OsStr::display once it is stablised,
            // Path and OsStr has the same `Display` impl
            write!(f, "{}", Path::new(os_str).display())?;
            if index + 1 < len {
                f.write_char(self.delimiter)?;
            }
        }
        Ok(())
    }
}

pub(crate) struct OptionOsStrDisplay<T>(pub(crate) Option<T>);

impl<T> fmt::Display for OptionOsStrDisplay<T>
where
    T: AsRef<OsStr>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Use OsStr::display once it is stabilized
        if let Some(os_str) = self.0.as_ref() {
            write!(f, "Some({})", Path::new(os_str).display())
        } else {
            f.write_str("None")
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum AsmFileExt {
    /// `.asm` files. On MSVC targets, we assume these should be passed to MASM (`ml.exe`)
    DotAsm,
    /// `.s`files, which do not have the special handling on MSVC targets
    DotS,
}

impl AsmFileExt {
    pub(crate) fn from_path(file: &Path) -> Option<AsmFileExt> {
        if let Some(ext) = file.extension() {
            if let Some(ext) = ext.to_str() {
                let ext = ext.to_lowercase();
                match &*ext {
                    "asm" => return Some(AsmFileExt::DotAsm),
                    "s" => return Some(AsmFileExt::DotS),
                    _ => return None,
                }
            }
        }
        None
    }
}

pub(crate) struct CmdAddOutputFileArgs {
    pub(crate) is_assembler_msvc: bool,
    pub(crate) msvc: bool,
    pub(crate) flang: bool,
    pub(crate) gfortran: bool,
    pub(crate) ifx: bool,
    pub(crate) lfortran: bool,
    pub(crate) is_asm: bool,
    pub(crate) is_arm: bool,
}

pub(crate) fn command_add_output_file(cmd: &mut Command, dst: &Path, args: CmdAddOutputFileArgs) {
    if args.is_assembler_msvc
        || !(!args.msvc || args.flang || args.gfortran || (args.is_asm && args.is_arm))
    {
        todo!()
    } else {
        cmd.arg("-o").arg(dst);
    }
}

/// Automates the `if is_disabled() { return error }`
pub(crate) fn check_disabled() -> Result<(), Error> {
    if is_disabled() {
        return Err(Error::new(
            ErrorKind::Disabled,
            "The functionality has been disabled by the `FF_FORCE_DISABLE` environment variable.",
        ));
    }
    Ok(())
}

/// Returns true if `feap_binding` has been disabled by `FC_FORCE_DISABLE`
fn is_disabled() -> bool {
    static CACHE: AtomicU8 = AtomicU8::new(0);

    let val = CACHE.load(Relaxed);

    fn compute_is_disabled() -> bool {
        match std::env::var_os("FC_FORCE_DISABLE") {
            None => false,
            Some(v) => &*v != "0" && &*v != "false" && &*v != "no",
        }
    }
    match val {
        2 => true,
        1 => false,
        0 => {
            let truth = compute_is_disabled();
            let encoded_truth = if truth { 2u8 } else { 1u8 };
            let _ = CACHE.compare_exchange(0, encoded_truth, Relaxed, Relaxed);
            truth
        }
        _ => unreachable!(),
    }
}

// Rust and clang/cc don't agree on how to name the target.
pub(crate) fn map_darwin_target_from_rust_to_compiler_architecture<'a>(
    target: &TargetInfo<'a>,
) -> &'a str {
    match target.full_arch {
        "aarch64" => "arm64",
        "arm64_32" => "arm64_32",
        "arm64e" => "arm64e",
        "armv7k" => "armv7k",
        "armv7s" => "armv7s",
        "i386" => "i386",
        "i686" => "i386",
        "powerpc" => "ppc",
        "powerpc64" => "ppc64",
        "x86_64" => "x86_64",
        "x86_64h" => "x86_64h",
        arch => arch,
    }
}
