use crate::error::{Error, ErrorKind};
use std::sync::OnceLock;
use std::{env, mem};

/// Information specific to a `rustc` target
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct TargetInfo<'a> {
    /// The full architecture
    pub full_arch: &'a str,
    /// The overall target architecture
    pub arch: &'a str,
    /// The target vendor
    pub vendor: &'a str,
    /// The operating system, or `none` on bare-metal targets
    pub os: &'a str,
    /// The environment on top of the operating system
    pub env: &'a str,
    /// The ABI on top of the operating system
    pub abi: &'a str,
}

impl<'a> TargetInfo<'a> {
    pub(crate) fn from_rustc_target(target: &'a str) -> Result<TargetInfo, Error> {
        if target == "x86_64-unknown-linux-none" {
            todo!()
        }
        if target == "armv7a-vex-v5" {
            todo!()
        }

        let mut components = target.split('-');

        // Insist that the target name contains at least a valid architecture
        let full_arch = components.next().ok_or(Error::new(
            ErrorKind::InvalidTarget,
            "target was empty".to_string(),
        ))?;
        let arch = parse_arch(full_arch).ok_or_else(|| {
            Error::new(
                ErrorKind::UnknownTarget,
                format!("target `{target}` had an unknown architecture"),
            )
        })?;

        // Newer target names have begun omitting the vendor, so the only component we know is the OS name
        let components: Vec<_> = components.collect();
        let (vendor, os, mut env, mut abi) = match &*components {
            [] => {
                return Err(Error::new(
                    ErrorKind::InvalidTarget,
                    format!("target `{target}` must have at least two components"),
                ));
            }
            // Two components; format is `arch-os`
            [os] => ("unknown", *os, "", ""),
            // Three components: `arch-vendor-os` or `arch-os-env+abi`
            [vendor_or_os, os_or_envabi] => {
                if let Some((env, abi)) = parse_envabi(os_or_envabi) {
                    ("unknown", *vendor_or_os, env, abi)
                } else {
                    (*vendor_or_os, *os_or_envabi, "", "")
                }
            }
            // Four components; format is `arch-vendor-os-env+abi`
            [vendor, os, envabi] => {
                let (env, abi) = parse_envabi(envabi).ok_or_else(|| {
                    Error::new(
                        ErrorKind::UnknownTarget,
                        format!("unknown environment/ABI `{envabi}` in target `{target}`"),
                    )
                })?;
                (*vendor, *os, env, abi)
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidTarget,
                    format!("too many components in target `{target}`"),
                ));
            }
        };

        // Part of the architecture name is carried over into the ABI
        match full_arch {
            arch if arch.starts_with("riscv32e") => {
                abi = "ilp32e";
            }
            _ => {}
        }

        // Various environment/ABIs are determined based on OS name.
        match os {
            "3ds" | "rtems" | "espidf" => env = "newlib",
            "vxworks" => env = "gnu",
            "redox" => env = "relibc",
            "aix" => abi = "vec-extabi",
            _ => {}
        }

        // Extra overrides for badly named targets.
        match target {
            // Actually simulator targets.
            "i386-apple-ios" | "x86_64-apple-ios" | "x86_64-apple-tvos" => {
                env = "sim";
            }
            // Name should've contained `muslabi64`.
            "mips64-openwrt-linux-musl" => {
                abi = "abi64";
            }
            // Specifies abi even though not in name.
            "armv6-unknown-freebsd" | "armv6k-nintendo-3ds" | "armv7-unknown-freebsd" => {
                abi = "eabihf";
            }
            // Specifies abi even though not in name.
            "armv7-unknown-linux-ohos" | "armv7-unknown-trusty" => {
                abi = "eabi";
            }
            _ => {}
        }

        let os = match os {
            // Horizon is the common/internal OS name for 3DS and the Switch.
            "3ds" | "switch" => "horizon",
            // macOS targets are badly named.
            "darwin" => "macos",

            // WASI targets contain the preview version in them too. Should've
            // been `wasi-p1`/`wasi-p2`, but that's probably too late now.
            os if os.starts_with("wasi") => {
                env = os.strip_prefix("wasi").unwrap();
                "wasi"
            }
            // Badly named targets `*-linux-androideabi`,
            // should be `*-android-eabi`.
            "androideabi" => {
                abi = "eabi";
                "android"
            }

            os => os,
        };

        // Intentionally also marked as an ABI:
        // https://github.com/rust-lang/rust/pull/86922
        if vendor == "fortanix" {
            abi = "fortanix";
        }
        if vendor == "uwp" {
            abi = "uwp";
        }
        if ["powerpc64-unknown-linux-gnu", "powerpc64-wrs-vxworks"].contains(&target) {
            abi = "elfv1";
        }
        if [
            "powerpc64-unknown-freebsd",
            "powerpc64-unknown-linux-musl",
            "powerpc64-unknown-openbsd",
            "powerpc64le-unknown-freebsd",
            "powerpc64le-unknown-linux-gnu",
            "powerpc64le-unknown-linux-musl",
        ]
        .contains(&target)
        {
            abi = "elfv2";
        }

        Ok(Self {
            full_arch,
            arch,
            vendor,
            os,
            env,
            abi,
        })
    }

    pub(crate) fn apple_sdk_name(&self) -> &'static str {
        match (self.os, self.env) {
            ("macos", "") => "macosx",
            ("ios", "") => "iphoneos",
            ("ios", "sim") => "iphonesimulator",
            ("ios", "macabi") => "macosx",
            ("tvos", "") => "appletvos",
            ("tvos", "sim") => "appletvsimulator",
            ("watchos", "") => "watchos",
            ("watchos", "sim") => "watchsimulator",
            ("visionos", "") => "xros",
            ("visionos", "sim") => "xrsimulator",
            (os, _) => panic!("invalid Apple target OS {}", os),
        }
    }
    
    pub(crate) fn is_arm(&self) -> bool {
        matches!(self.arch, "aarch64" | "arm64ec" | "arm")
    }

    pub(crate) fn apple_version_flag(&self, min_version: &str) -> String {
        // There are many aliases for these, and `-mtargetos=` is preferred on Clang nowadays, but
        // for compatibility with older Clang, we use the earliest supported name here.
        //
        // NOTE: GNU does not support `-miphoneos-version-min=` etc. (because it does not support
        // iOS in general), but we specify them anyhow in case we actually have a Clang-like
        // compiler disguised as a GNU-like compiler, or in case GNU adds support for these in the
        // future.
        //
        // See also:
        // https://clang.llvm.org/docs/ClangCommandLineReference.html#cmdoption-clang-mmacos-version-min
        // https://clang.llvm.org/docs/AttributeReference.html#availability
        // https://gcc.gnu.org/onlinedocs/gcc/Darwin-Options.html#index-mmacosx-version-min
        match (self.os, self.env) {
            ("macos", "") => format!("-mmacosx-version-min={min_version}"),
            ("ios", "") => format!("-miphoneos-version-min={min_version}"),
            ("ios", "sim") => format!("-mios-simulator-version-min={min_version}"),
            ("ios", "macabi") => format!("-mtargetos=ios{min_version}-macabi"),
            ("tvos", "") => format!("-mappletvos-version-min={min_version}"),
            ("tvos", "sim") => format!("-mappletvsimulator-version-min={min_version}"),
            ("watchos", "") => format!("-mwatchos-version-min={min_version}"),
            ("watchos", "sim") => format!("-mwatchsimulator-version-min={min_version}"),
            // `-mxros-version-min` does not exist
            // https://github.com/llvm/llvm-project/issues/88271
            ("visionos", "") => format!("-mtargetos=xros{min_version}"),
            ("visionos", "sim") => format!("-mtargetos=xros{min_version}-simulator"),
            (os, _) => panic!("invalid Apple target OS {}", os),
        }
    }
}

/// Parser for [`TargetInfo`], contains cached information
#[derive(Default, Debug)]
pub(crate) struct TargetInfoParser(OnceLock<Result<TargetInfoParserInner, Error>>);

#[derive(Debug)]
struct TargetInfoParserInner {
    full_arch: Box<str>,
    arch: Box<str>,
    vendor: Box<str>,
    os: Box<str>,
    env: Box<str>,
    abi: Box<str>,
}

impl TargetInfoParser {
    pub fn parse_from_cargo_environment_variables(&self) -> Result<TargetInfo<'_>, Error> {
        match self
            .0
            .get_or_init(TargetInfoParserInner::from_cargo_environment_variables)
        {
            Ok(TargetInfoParserInner {
                full_arch,
                arch,
                vendor,
                os,
                env,
                abi,
            }) => Ok(TargetInfo {
                full_arch,
                arch,
                vendor,
                os,
                env,
                abi,
            }),
            Err(e) => Err(e.clone()),
        }
    }
}

impl TargetInfoParserInner {
    fn from_cargo_environment_variables() -> Result<TargetInfoParserInner, Error> {
        // `TARGET` must be present
        let target_name = env::var("TARGET").map_err(|err| {
            Error::new(
                ErrorKind::EnvVarNotFound,
                format!("failed reading TARGET: {err}"),
            )
        })?;

        // Parse the full architecture name from the target name
        let (full_arch, _rest) = target_name.split_once('-').ok_or(Error::new(
            ErrorKind::InvalidTarget,
            format!("target `{target_name}` only had a single component"),
        ))?;

        let cargo_env = |name, fallback: Option<&str>| -> Result<Box<str>, Error> {
            // No need to emit `rerun-if-env-changed` for these
            match env::var(name) {
                Ok(var) => Ok(var.into_boxed_str()),
                Err(err) => match fallback {
                    Some(fallback) => Ok(fallback.into()),
                    None => Err(Error::new(
                        ErrorKind::EnvVarNotFound,
                        format!(
                            "did not find fallback information for target `{target_name}`, and failed reading {name}: {err}"
                        ),
                    )),
                },
            }
        };

        // Prefer to use `CARGO_ENV_*` if set, since these contain the most correct information
        // relative to the current `rustc`, and makes it possible to support custom target specs
        let fallback_target = TargetInfo::from_rustc_target(&target_name).ok();
        let ft = fallback_target.as_ref();
        let arch = cargo_env("CARGO_CFG_TARGET_ARCH", ft.map(|t| t.arch))?;
        let vendor = cargo_env("CARGO_CFG_TARGET_VENDOR", ft.map(|t| t.vendor))?;
        let os = cargo_env("CARGO_CFG_TARGET_OS", ft.map(|t| t.os))?;
        let mut env = cargo_env("CARGO_CFG_TARGET_ENV", ft.map(|t| t.env))?;
        let mut abi = cargo_env("CARGO_CFG_TARGET_ABI", ft.map(|t| t.abi))
            .unwrap_or_else(|_| String::default().into_boxed_str());

        if matches!(&*abi, "macabi" | "sim") {
            debug_assert!(
                matches!(&*env, "" | "macabi" | "sim"),
                "env/abi mismatch: {:?}, {:?}",
                env,
                abi,
            );
            env = mem::replace(&mut abi, String::default().into_boxed_str());
        }

        Ok(Self {
            full_arch: full_arch.to_string().into_boxed_str(),
            arch,
            vendor,
            os,
            env,
            abi,
        })
    }
}

/// Oarse environment and ABI from the last component of the target name
fn parse_envabi(last_component: &str) -> Option<(&str, &str)> {
    let (env, abi) = match last_component {
        // gnullvm | gnueabi | gnueabihf | gnuabiv2 | gnuabi64 | gnuspe | gnux32 | gnu_ilp32
        env_and_abi if env_and_abi.starts_with("gnu") => {
            let abi = env_and_abi.strip_prefix("gnu").unwrap();
            let abi = abi.strip_prefix("_").unwrap_or(abi);
            ("gnu", abi)
        }
        // musl | musleabi | musleabihf | muslabi64 | muslspe
        env_and_abi if env_and_abi.starts_with("musl") => {
            ("musl", env_and_abi.strip_prefix("musl").unwrap())
        }
        // uclibc | uclibceabi | uclibceabihf
        env_and_abi if env_and_abi.starts_with("uclibc") => {
            ("uclibc", env_and_abi.strip_prefix("uclibc").unwrap())
        }
        // newlib | newlibeabihf
        env_and_abi if env_and_abi.starts_with("newlib") => {
            ("newlib", env_and_abi.strip_prefix("newlib").unwrap())
        }

        // Environments
        "msvc" => ("msvc", ""),
        "ohos" => ("ohos", ""),
        "qnx700" => ("nto70", ""),
        "qnx710_iosock" => ("nto71_iosock", ""),
        "qnx710" => ("nto71", ""),
        "qnx800" => ("nto80", ""),
        "sgx" => ("sgx", ""),
        "threads" => ("threads", ""),
        "mlibc" => ("mlibc", ""),

        // ABIs
        "abi64" => ("", "abi64"),
        "abiv2" => ("", "spe"),
        "eabi" => ("", "eabi"),
        "eabihf" => ("", "eabihf"),
        "macabi" => ("macabi", ""),
        "sim" => ("sim", ""),
        "softfloat" => ("", "softfloat"),
        "spe" => ("", "spe"),
        "x32" => ("", "x32"),

        // Badly named targets, ELF is already known from target OS.
        // Probably too late to fix now though.
        "elf" => ("", ""),
        // Undesirable to expose to user code (yet):
        // https://github.com/rust-lang/rust/pull/131166#issuecomment-2389541917
        "freestanding" => ("", ""),

        _ => return None,
    };

    Some((env, abi))
}

fn parse_arch(full_arch: &str) -> Option<&str> {
    Some(match full_arch {
        arch if arch.starts_with("mipsisa32r6") => "mips32r6", // mipsisa32r6 | mipsisa32r6el
        arch if arch.starts_with("mipsisa64r6") => "mips64r6", // mipsisa64r6 | mipsisa64r6el

        arch if arch.starts_with("mips64") => "mips64", // mips64 | mips64el
        arch if arch.starts_with("mips") => "mips",     // mips | mipsel

        arch if arch.starts_with("loongarch64") => "loongarch64",
        arch if arch.starts_with("loongarch32") => "loongarch32",

        arch if arch.starts_with("powerpc64") => "powerpc64", // powerpc64 | powerpc64le
        arch if arch.starts_with("powerpc") => "powerpc",
        arch if arch.starts_with("ppc64") => "powerpc64",
        arch if arch.starts_with("ppc") => "powerpc",

        arch if arch.starts_with("x86_64") => "x86_64", // x86_64 | x86_64h
        arch if arch.starts_with("i") && arch.ends_with("86") => "x86", // i386 | i586 | i686

        "arm64ec" => "arm64ec", // https://github.com/rust-lang/rust/issues/131172
        arch if arch.starts_with("aarch64") => "aarch64", // arm64e | arm64_32
        arch if arch.starts_with("arm64") => "aarch64", // aarch64 | aarch64_be

        arch if arch.starts_with("arm") => "arm", // arm | armv7s | armeb | ...
        arch if arch.starts_with("thumb") => "arm", // thumbv4t | thumbv7a | thumbv8m | ...

        arch if arch.starts_with("riscv64") => "riscv64",
        arch if arch.starts_with("riscv32") => "riscv32",

        arch if arch.starts_with("wasm64") => "wasm64",
        arch if arch.starts_with("wasm32") => "wasm32", // wasm32 | wasm32v1
        "asmjs" => "wasm32",

        arch if arch.starts_with("nvptx64") => "nvptx64",
        arch if arch.starts_with("nvptx") => "nvptx",

        arch if arch.starts_with("bpf") => "bpf", // bpfeb | bpfel

        // https://github.com/bytecodealliance/wasmtime/tree/v30.0.1/pulley
        arch if arch.starts_with("pulley64") => "pulley64",
        arch if arch.starts_with("pulley32") => "pulley32",

        // https://github.com/Clever-ISA/Clever-ISA
        arch if arch.starts_with("clever") => "clever",

        "sparc" | "sparcv7" | "sparcv8" => "sparc",
        "sparc64" | "sparcv9" => "sparc64",

        "amdgcn" => "amdgpu",
        "avr" => "avr",
        "csky" => "csky",
        "hexagon" => "hexagon",
        "m68k" => "m68k",
        "msp430" => "msp430",
        "r600" => "r600",
        "s390x" => "s390x",
        "xtensa" => "xtensa",

        // Arches supported by gcc, but not LLVM.
        arch if arch.starts_with("alpha") => "alpha", // DEC Alpha
        "hppa" => "hppa", // https://en.wikipedia.org/wiki/PA-RISC, also known as HPPA
        arch if arch.starts_with("sh") => "sh", // SuperH
        _ => return None,
    })
}
