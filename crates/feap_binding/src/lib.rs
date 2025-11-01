mod cache;
mod cargo;
mod error;
mod target;
mod tempfile;
mod tool;
mod util;

use crate::util::JoinOsStrs;
use crate::{
    cache::BuildCache,
    cargo::{CargoOutput, OutputKind},
    error::{Error, ErrorKind},
    target::TargetInfo,
    tool::{Tool, ToolFamily},
    util::{AsmFileExt, CmdAddOutputFileArgs, OptionOsStrDisplay},
};
use std::{
    borrow::Cow,
    collections::hash_map,
    env,
    ffi::OsStr,
    fs,
    hash::Hasher,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::Arc,
};

/// A builder for compilation of a native library
///
/// A `Build` is the main type of the crate and is used to control all the
/// various configuration options and such of a compile.
#[derive(Clone, Debug)]
pub struct Build {
    files: Vec<Arc<Path>>,
    objects: Vec<Arc<Path>>,
    include_directories: Vec<Arc<Path>>,
    definitions: Vec<(Arc<str>, Option<Arc<str>>)>,
    no_default_flags: bool,
    opt_level: Option<Arc<str>>,
    std: Option<Arc<str>>,
    flags: Vec<Arc<OsStr>>,
    shared_flag: Option<bool>,
    static_flag: Option<bool>,
    warnings: Option<bool>,
    extra_warnings: Option<bool>,
    warnings_into_errors: bool,
    asm_flags: Vec<Arc<OsStr>>,
    ar_flags: Vec<Arc<OsStr>>,
    link_lib_modifiers: Vec<Arc<OsStr>>,
    target: Option<Arc<str>>,
    host: Option<Arc<str>>,
    out_dir: Option<Arc<Path>>,
    env: Vec<(Arc<OsStr>, Arc<OsStr>)>,
    compiler: Option<Arc<Path>>,
    archiver: Option<Arc<Path>>,
    cargo_output: CargoOutput,
    emit_rerun_if_env_changed: bool,
    build_cache: Arc<BuildCache>,
}

impl Default for Build {
    fn default() -> Self {
        Self::new()
    }
}

impl Build {
    /// Construct a new instance of a blank set of configurations
    /// The builder is finished with the [`compile`] function
    pub fn new() -> Build {
        Build {
            files: Vec::new(),
            objects: Vec::new(),
            include_directories: Vec::new(),
            definitions: Vec::new(),
            no_default_flags: false,
            opt_level: None,
            std: None,
            flags: Vec::new(),
            shared_flag: None,
            static_flag: None,
            warnings: None,
            extra_warnings: None,
            warnings_into_errors: false,
            asm_flags: Vec::new(),
            ar_flags: Vec::new(),
            link_lib_modifiers: Vec::new(),
            target: None,
            host: None,
            out_dir: None,
            env: Vec::new(),
            compiler: None,
            archiver: None,
            cargo_output: CargoOutput::new(),
            emit_rerun_if_env_changed: true,
            build_cache: Arc::default(),
        }
    }

    /// Add a file which will be compiled
    pub fn file<P: AsRef<Path>>(&mut self, p: P) -> &mut Build {
        self.files.push(p.as_ref().into());
        self
    }

    /// Add a directory to the `-I` or include path for headers
    pub fn include<P: AsRef<Path>>(&mut self, dir: P) -> &mut Build {
        self.include_directories.push(dir.as_ref().into());
        self
    }

    /// Add an arbitrary flag to the invocation of the compiler
    pub fn flag(&mut self, flag: impl AsRef<OsStr>) -> &mut Build {
        self.flags.push(flag.as_ref().into());
        self
    }

    /// Add a flag to the invocation of the ar
    pub fn ar_flag(&mut self, flag: impl AsRef<OsStr>) -> &mut Build {
        self.ar_flags.push(flag.as_ref().into());
        self
    }

    /// Add a flag that will only be used with assembly files
    pub fn asm_flag(&mut self, flag: impl AsRef<OsStr>) -> &mut Build {
        self.asm_flags.push(flag.as_ref().into());
        self
    }

    /// Configures the compiler to be used
    pub fn compiler<P: AsRef<Path>>(&mut self, compiler: P) -> &mut Build {
        self.compiler = Some(compiler.as_ref().into());
        self
    }

    /// Adds a native library modifier
    pub fn link_lib_modifier(&mut self, link_lib_modifier: impl AsRef<OsStr>) -> &mut Build {
        self.link_lib_modifiers
            .push(link_lib_modifier.as_ref().into());
        self
    }

    /// Run the compiler, generating the file `output`
    ///
    /// The `output` string argument determines the file name for the compiled
    /// library. The rust compiler will create an assembly named "lib"+output+".a".
    /// MSVC will create a file named output+".lib".
    ///
    /// The choice of `output` is close to arbitrary, but:
    /// - must be nonempty
    /// - must not contain a path separator
    /// - must be unique across all `compile` invocations
    pub fn compile(&self, output: &str) {
        if let Err(e) = self.try_compile(output) {
            Self::fail(&e.message);
        }
    }

    /// Run the compiler, generating the file `output`
    pub fn try_compile(&self, output: &str) -> Result<(), Error> {
        let mut output_components = Path::new(output).components();
        match (output_components.next(), output_components.next()) {
            (Some(Component::Normal(_)), None) => { /* valid */ }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    "argument of `compile` must be a single normal component",
                ));
            }
        }

        let (lib_name, gnu_lib_name) = if output.starts_with("lib") && output.ends_with(".a") {
            (&output[3..output.len() - 2], output.to_owned())
        } else {
            let mut gnu = String::with_capacity(5 + output.len());
            gnu.push_str("lib");
            gnu.push_str(output);
            gnu.push_str(".a");
            (output, gnu)
        };
        let dst = self.get_out_dir()?;

        let objects = Self::objects_from_files(&self.files, &dst)?;

        self.compile_objects(&objects)?;
        self.assemble(lib_name, &dst.join(gnu_lib_name), &objects)?;

        let target = self.get_target()?;
        if target.env == "msvc" {
            todo!()
        }

        if self.link_lib_modifiers.is_empty() {
            self.cargo_output
                .print_metadata(&format_args!("cargo:rustc-link-lib=static={}", lib_name));
        } else {
            self.cargo_output.print_metadata(&format_args!(
                "cargo:rustc-link-lib=static:{}={}",
                JoinOsStrs {
                    slice: &self.link_lib_modifiers,
                    delimiter: ','
                },
                lib_name
            ));
        }
        self.cargo_output.print_metadata(&format_args!(
            "cargo:rustc-link-search=native={}",
            dst.display()
        ));

        Ok(())
    }

    fn assemble(&self, _lib_name: &str, dst: &Path, objs: &[Object]) -> Result<(), Error> {
        // Delete the destination if it exists as we want to create on the first iteration instead of appending
        let _ = fs::remove_file(dst);

        // Add objects to the archive in limited-length batches.
        let objs = objs
            .iter()
            .map(|o| o.dst.as_path())
            .chain(self.objects.iter().map(std::ops::Deref::deref))
            .collect::<Vec<_>>();
        for chunk in objs.chunks(100) {
            self.assemble_progressive(dst, chunk)?;
        }

        let target = self.get_target()?;
        if target.env == "msvc" {
            todo!()
        } else {
            // Non-msvc targets need a separate step to add the symbol table
            // to archives since our construction command of `cq` doesn't add it for us
            let mut ar = self.try_get_archiver()?;

            // Note: We add `s` even if flags were passed
            Self::run(ar.arg("s").arg(dst), &self.cargo_output)?;
        }

        Ok(())
    }

    fn assemble_progressive(&self, dst: &Path, objs: &[&Path]) -> Result<(), Error> {
        let target = self.get_target()?;

        let (mut cmd, program, _any_flags) = self.try_get_archiver_and_flags()?;
        if target.env == "msvc" && !program.to_string_lossy().contains("llvm-ar") {
            todo!()
        } else {
            // Set an environment variable to tell the OSX archiver to ensure
            // that all dates listed in the archive are zero
            cmd.env("ZERO_AR_DATE", "1");

            Self::run(cmd.arg("cq").arg(dst).args(objs), &self.cargo_output)?;
        }

        Ok(())
    }

    fn try_get_archiver(&self) -> Result<Command, Error> {
        Ok(self.try_get_archiver_and_flags()?.0)
    }

    fn try_get_archiver_and_flags(&self) -> Result<(Command, PathBuf, bool), Error> {
        let (mut cmd, name) = self.get_base_archiver()?;
        let mut any_flags = false;
        if let Some(flags) = self.getenv_flags("ARFLAGS")? {
            any_flags = true;
            cmd.args(flags);
        }
        for flag in &self.ar_flags {
            any_flags = true;
            cmd.arg(&**flag);
        }
        Ok((cmd, name, any_flags))
    }

    fn get_base_archiver(&self) -> Result<(Command, PathBuf), Error> {
        if let Some(ref a) = self.archiver {
            let archiver = &**a;
            return Ok((self.cmd(archiver), archiver.into()));
        }

        self.get_base_archiver_variant("AR", "ar")
    }

    fn get_base_archiver_variant(
        &self,
        env: &str,
        tool: &str,
    ) -> Result<(Command, PathBuf), Error> {
        let target = self.get_target()?;
        let mut name = PathBuf::new();
        let tool_opt: Option<Command> = self
            .env_tool(env)
            .map(|(tool, _wrapper, args)| {
                name.clone_from(&tool);
                let mut cmd = self.cmd(tool);
                cmd.args(args);
                cmd
            })
            .or_else(|| None);

        let tool = match tool_opt {
            Some(t) => t,
            None => {
                if target.env == "msvc" {
                    todo!()
                } else if self.get_is_cross_compile()? {
                    todo!()
                } else {
                    name = tool.into();
                    self.cmd(&name)
                }
            }
        };

        Ok((tool, name))
    }

    fn cmd<P: AsRef<OsStr>>(&self, prog: P) -> Command {
        let mut cmd = Command::new(prog);
        for (a, b) in self.env.iter() {
            cmd.env(a, b);
        }
        cmd
    }

    pub(crate) fn run(cmd: &mut Command, cargo_output: &CargoOutput) -> Result<(), Error> {
        let mut child = cargo::spawn(cmd, cargo_output)?;
        cargo::wait_on_child(cmd, &mut child, cargo_output)
    }

    pub(crate) fn run_output(
        cmd: &mut Command,
        cargo_output: &CargoOutput,
    ) -> Result<Vec<u8>, Error> {
        // We specifically need the output to be captured, so override default
        let mut captured_cargo_output = cargo_output.clone();
        captured_cargo_output.output = OutputKind::Capture;
        let mut child = cargo::spawn(cmd, &captured_cargo_output)?;

        let mut stdout = Vec::new();
        child
            .stdout
            .take()
            .unwrap()
            .read_to_end(&mut stdout)
            .unwrap();

        // Don't care about this output, use the normal settings
        cargo::wait_on_child(cmd, &mut child, cargo_output)?;

        Ok(stdout)
    }

    fn compile_objects(&self, objs: &[Object]) -> Result<(), Error> {
        util::check_disabled()?;

        #[cfg(feature = "parallel")]
        todo!();

        for obj in objs {
            let mut cmd = self.create_compile_object_cmd(obj)?;
            Self::run(&mut cmd, &self.cargo_output)?;
        }
        Ok(())
    }

    fn create_compile_object_cmd(&self, obj: &Object) -> Result<Command, Error> {
        let asm_ext = AsmFileExt::from_path(&obj.src);
        let is_asm = asm_ext.is_some();
        let target = self.get_target()?;
        let msvc = target.env == "msvc";
        let compiler = self.try_get_compiler()?;

        let is_assembler_msvc = msvc && asm_ext == Some(AsmFileExt::DotAsm);
        let mut cmd = if is_assembler_msvc {
            todo!()
        } else {
            let mut cmd = compiler.to_command();
            for (a, b) in self.env.iter() {
                cmd.env(a, b);
            }
            cmd
        };
        let is_arm = target.is_arm();
        util::command_add_output_file(
            &mut cmd,
            &obj.dst,
            CmdAddOutputFileArgs {
                is_assembler_msvc,
                msvc: false,
                flang: compiler.family == ToolFamily::Flang,
                gfortran: compiler.family == ToolFamily::GFortran,
                ifx: compiler.family == ToolFamily::IntelIFX,
                lfortran: compiler.family == ToolFamily::LFortran,
                is_asm,
                is_arm,
            },
        );

        // armasm and armasm64 don't requrie -c option
        if !is_assembler_msvc || !is_arm {
            cmd.arg("-c");
        }
        if is_asm {
            cmd.args(self.asm_flags.iter().map(std::ops::Deref::deref));
        }

        cmd.arg(&obj.src);

        if cfg!(target_os = "macos") {
            self.fix_env_for_apple_os(&mut cmd)?;
        }

        Ok(cmd)
    }

    fn fix_env_for_apple_os(&self, cmd: &mut Command) -> Result<(), Error> {
        let target = self.get_target()?;
        if cfg!(target_os = "macos") && target.os == "macos" {
            cmd.env_remove("IPHONEOS_DEPLOYMENT_TARGET");
        }
        Ok(())
    }

    /// Get the compiler that's in use for this configuration
    fn try_get_compiler(&self) -> Result<Tool, Error> {
        let opt_level = self.get_opt_level()?;
        let target = self.get_target()?;

        let mut cmd = self.get_base_compiler()?;

        // The flags below are added in roughly the following order:
        // - Default flags: controlled by feap_binding
        // - `rustc`-inherited flags
        // - Builder flags: controlled by the developer using feap_binding
        // - Environment flags: controlled by the end user

        // Disables non-English messages from localized linkers
        cmd.env.push(("LC_ALL".into(), "C".into()));

        // Disable default flag generation
        let no_defaults = self.no_default_flags || self.getenv_boolean("CRATE_CC_NO_DEFAULTS");
        if !no_defaults {
            self.add_default_flags(&mut cmd, &target, &opt_level)?;
        }

        if let Some(ref std) = self.std {
            let separator = '=';
            cmd.push_fc_arg(format!("-std{}{}", separator, std).into());
        }

        for directory in self.include_directories.iter() {
            cmd.args.push("-I".into());
            cmd.args.push(directory.as_os_str().into());
        }

        if self.warnings_into_errors {
            let warnings_to_errors_flags = cmd.family.warnings_to_errors_flag().into();
            cmd.push_fc_arg(warnings_to_errors_flags);
        }

        let envflags = self.getenv_flags("FCFLAGS")?;
        if self.warnings.unwrap_or(envflags.is_none()) {
            if let Some(wflags) = cmd.family.warning_flags() {
                cmd.push_fc_arg(wflags.into());
            }
        }
        if self.extra_warnings.unwrap_or(envflags.is_none()) {
            if let Some(wflags) = cmd.family.extra_warning_flags() {
                cmd.push_fc_arg(wflags.into());
            }
        }

        for flag in self.flags.iter() {
            cmd.args.push((**flag).into());
        }

        for (key, value) in self.definitions.iter() {
            if let Some(ref value) = *value {
                cmd.args.push(format!("-D{key}={value}").into());
            } else {
                cmd.args.push(format!("-D{key}").into());
            }
        }

        if let Some(flags) = &envflags {
            for arg in flags {
                cmd.push_fc_arg(arg.into());
            }
        }

        Ok(cmd)
    }

    fn add_default_flags(
        &self,
        cmd: &mut Tool,
        target: &TargetInfo<'_>,
        opt_level: &str,
    ) -> Result<(), Error> {
        // Optimizing
        cmd.push_opt_unless_duplicate(format!("-O{opt_level}").into());

        if !cmd.is_like_msvc() {
            if target.arch == "x86" {
                cmd.args.push("-m32".into());
            } else if target.abi == "x32" {
                cmd.args.push("-mx32".into());
            } else if target.os == "aix" {
                if cmd.family == ToolFamily::GFortran {
                    cmd.args.push("-maix64".into());
                } else {
                    cmd.args.push("-m64".into());
                }
            } else if target.arch == "x86_64" || target.arch == "powerpc64" {
                cmd.args.push("-m64".into());
            }
        }

        // Target flags
        match cmd.family {
            ToolFamily::Flang => {}
            ToolFamily::GFortran => {
                if target.vendor == "kmc" {
                    todo!()
                }

                if self.static_flag.is_none() {
                    let features = self.getenv("CARGO_CFG_TARGET_FEATURE");
                    let features = features.as_deref().unwrap_or_default();
                    if features.to_string_lossy().contains("crt-static") {
                        cmd.args.push("-static".into());
                    }
                }

                // armv7 targets tet to use armv7 instructions
                if (target.full_arch.starts_with("armv7") || target.full_arch.starts_with("thumb7"))
                    && (target.os == "linux" || target.vendor == "kmc")
                {
                    todo!()
                }

                // (x86 Android doesn't say "eabi")
                if target.os == "android" && target.full_arch.contains("v7") {
                    todo!()
                }

                if target.full_arch.contains("neon") {
                    todo!()
                }

                if target.full_arch == "armv4t" && target.os == "linux" {
                    todo!()
                }

                if target.full_arch == "armv5te" && target.os == "linux" {
                    todo!()
                }

                // For us arm == armv6 by default
                if target.full_arch == "arm" && target.os == "linux" {
                    todo!()
                }

                // Turn codegen down on i586 to avoid some instructions.
                if target.full_arch == "i586" && target.os == "linux" {
                    todo!()
                }

                // Set codegen level for i686 correctly
                if target.full_arch == "i686" && target.os == "linux" {
                    todo!()
                }

                if target.arch == "x86" && target.env == "musl" {
                    todo!()
                }

                if target.arch == "arm" && target.os == "none" && target.abi == "eabihf" {
                    todo!()
                }
                if target.full_arch.starts_with("thumb") {
                    todo!()
                }
                if target.full_arch.starts_with("thumbv6m") {
                    todo!()
                }
                if target.full_arch.starts_with("thumbv6m") {
                    todo!()
                }
                if target.full_arch.starts_with("thumbv7em") {
                    todo!()
                }
                if target.full_arch.starts_with("thumbv7m") {
                    todo!()
                }
                if target.full_arch.starts_with("thumbv8m.base") {
                    todo!()
                }
                if target.full_arch.starts_with("thumbv8m.main") {
                    todo!()
                }
                if target.full_arch.starts_with("armebv7r")
                    || target.full_arch.starts_with("armv7r")
                {
                    todo!()
                }
                if target.full_arch.starts_with("armv7a") {
                    todo!()
                }

                if target.arch == "riscv32" || target.arch == "riscv64" {
                    todo!()
                }
            }
            ToolFamily::IntelIFX => {
                todo!()
            }
            ToolFamily::LFortran => {
                todo!()
            }
        }

        if target.vendor == "apple" {
            self.apple_flags(cmd)?;
        }

        if self.static_flag.unwrap_or(false) {
            cmd.args.push("-static".into());
        }
        if self.shared_flag.unwrap_or(false) {
            cmd.args.push("-shared".into());
        }

        Ok(())
    }

    fn apple_flags(&self, cmd: &mut Tool) -> Result<(), Error> {
        let target = self.get_target()?;

        // This is a Darwin/Apple-specific flag that works both on GCC and Clang, but it is only
        // necessary on GCC since we specify `-target` on Clang.
        // https://gcc.gnu.org/onlinedocs/gcc/Darwin-Options.html#:~:text=arch
        // https://clang.llvm.org/docs/CommandGuide/clang.html#cmdoption-arch
        if cmd.is_like_gnu() {
            let arch = util::map_darwin_target_from_rust_to_compiler_architecture(&target);
            cmd.args.push("-arch".into());
            cmd.args.push(arch.into());
        }

        // Pass the deployment target via `-mmacosx-version-min=`, `-miphoneos-version-min=` and
        // similar. Also necessary on GCC, as it forces a compilation error if the compiler is not
        // configured for Darwin: https://gcc.gnu.org/onlinedocs/gcc/Darwin-Options.html
        //
        // On visionOS and Mac Catalyst, there is no -m*-version-min= flag:
        // https://github.com/llvm/llvm-project/issues/88271
        // And the workaround to use `-mtargetos=` cannot be used with the `--target` flag that we
        // otherwise specify. So we avoid emitting that, and put the version in `--target` instead.
        if cmd.is_like_gnu() || !(target.os == "visionos" || target.env == "macabi") {
            let min_version = self.apple_deployment_target(&target);
            cmd.args
                .push(target.apple_version_flag(&min_version).into());
        }

        Ok(())
    }

    fn apple_deployment_target(&self, target: &TargetInfo<'_>) -> Arc<str> {
        let sdk = target.apple_sdk_name();
        if let Some(ret) = self
            .build_cache
            .apple_version_cache
            .read()
            .expect("apple_versions_cache lock failed")
            .get(sdk)
            .cloned()
        {
            return ret;
        }

        let default_deplayment_from_sdk = || -> Option<Arc<str>> {
            let version = Self::run_output(
                self.cmd("xcrun")
                    .arg("--show-sdk-version")
                    .arg("--sdk")
                    .arg(sdk),
                &self.cargo_output,
            )
            .ok()?;

            Some(Arc::from(std::str::from_utf8(&version).ok()?.trim()))
        };

        let deployment_from_env = |name: &str| -> Option<Arc<str>> {
            // Note: self.env isn't hit in production codepaths
            self.env
                .iter()
                .find(|(k, _)| &**k == OsStr::new(name))
                .map(|(_, v)| v)
                .cloned()
                .or_else(|| self.getenv(name))?
                .to_str()
                .map(Arc::from)
        };

        let version: Arc<str> = match target.os {
            "macos" => deployment_from_env("MACOSX_DEPLOYMENT_TARGET")
                .or_else(default_deplayment_from_sdk)
                .unwrap_or_else(|| {
                    if target.arch == "aarch64" {
                        "11.0".into()
                    } else {
                        "10.7".into()
                    }
                }),
            _ => todo!("Not implemented yet"),
        };

        self.build_cache
            .apple_version_cache
            .write()
            .expect("apple_version_cache lock failed")
            .insert(sdk.into(), version.clone());

        version
    }

    fn get_base_compiler(&self) -> Result<Tool, Error> {
        let out_dir = self.get_out_dir().ok();
        let out_dir = out_dir.as_deref();

        if let Some(c) = &self.compiler {
            return Ok(Tool::new(
                (**c).to_owned(),
                &self.build_cache.cached_compiler_family,
                &self.cargo_output,
                out_dir,
            ));
        }

        let target = self.get_target()?;

        let env = "FC";
        let gfortran = "gfortran";
        let flang = "flang";
        let default = gfortran;

        let tool_opt = self
            .env_tool(env)
            .map(|(tool, _wrapper, args)| {
                let t = Tool::with_args(
                    tool,
                    args.clone(),
                    &self.build_cache.cached_compiler_family,
                    &self.cargo_output,
                    out_dir,
                );
                t
            })
            .or_else(|| None);

        let tool = match tool_opt {
            Some(t) => t,
            None => {
                let compiler = if cfg!(windows) && target.os == "windows" {
                    let fc = if target.abi == "llvm" {
                        flang
                    } else {
                        gfortran
                    };
                    format!("{fc}.exe")
                } else {
                    default.to_string()
                };

                Tool::new(
                    PathBuf::from(compiler),
                    &self.build_cache.cached_compiler_family,
                    &self.cargo_output,
                    out_dir,
                )
            }
        };

        Ok(tool)
    }

    /// Returns compiler path, optional modifier name from whitelist, and arguments ved
    fn env_tool(&self, name: &str) -> Option<(PathBuf, Option<Arc<OsStr>>, Vec<String>)> {
        let _tool = self.getenv_with_target_prefixes(name).ok()?;

        todo!()
    }

    fn get_opt_level(&self) -> Result<Cow<'_, str>, Error> {
        match &self.opt_level {
            Some(ol) => Ok(Cow::Borrowed(ol)),
            None => self.getenv_unwrap_str("OPT_LEVEL").map(Cow::Owned),
        }
    }

    fn get_target(&self) -> Result<TargetInfo<'_>, Error> {
        match &self.target {
            Some(t) if Some(&**t) != self.getenv_unwrap_str("TARGET").ok().as_deref() => {
                TargetInfo::from_rustc_target(t)
            }
            // Fetch target information from environment if not set
            _ => self
                .build_cache
                .target_info_parser
                .parse_from_cargo_environment_variables(),
        }
    }

    fn get_raw_target(&self) -> Result<Cow<'_, str>, Error> {
        match &self.target {
            Some(t) => Ok(Cow::Borrowed(t)),
            None => self.getenv_unwrap_str("TARGET").map(Cow::Owned),
        }
    }

    fn get_is_cross_compile(&self) -> Result<bool, Error> {
        let target = self.get_raw_target()?;
        let host: Cow<'_, str> = match &self.host {
            Some(h) => Cow::Borrowed(h),
            None => Cow::Owned(self.getenv_unwrap_str("HOST")?),
        };
        Ok(host != target)
    }

    /// Find the destination object path for each file in the input source files,
    /// and store them in the output Object
    fn objects_from_files(files: &[Arc<Path>], dst: &Path) -> Result<Vec<Object>, Error> {
        let mut objects = Vec::with_capacity(files.len());
        for file in files {
            let basename = file
                .file_name()
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidArgument,
                        "No file_name for object file path!",
                    )
                })?
                .to_string_lossy();
            let dirname = file
                .parent()
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::InvalidArgument,
                        "No parent for object file path!",
                    )
                })?
                .to_string_lossy();

            // Hash the dirname. This should prevent conflicts if we have multiple object files with
            // the same filename in different subfolders
            let mut hasher = hash_map::DefaultHasher::new();

            // Make the dirname relative to avoid full system paths and making the output system-dependent
            let dirname = if let Some(root) = std::env::var_os("CARGO_MANIFEST_DIR") {
                let root = root.to_string_lossy();
                Cow::Borrowed(dirname.strip_prefix(&*root).unwrap_or(&dirname))
            } else {
                dirname
            };

            hasher.write(dirname.as_bytes());
            if let Some(extension) = file.extension() {
                hasher.write(extension.to_string_lossy().as_bytes());
            }
            let obj = dst
                .join(format!("{:016x}-{}", hasher.finish(), basename))
                .with_extension("o");

            match obj.parent() {
                Some(s) => fs::create_dir_all(s)?,
                None => {
                    return Err(Error::new(
                        ErrorKind::InvalidArgument,
                        "dst is an invalid path with no parent",
                    ));
                }
            }

            objects.push(Object::new(file.to_path_buf(), obj));
        }

        Ok(objects)
    }

    fn get_out_dir(&self) -> Result<Cow<'_, Path>, Error> {
        // todo: temp
        // unsafe {
        //     std::env::set_var(
        //         "OUT_DIR",
        //         "/Users/wig/dev/fears/target/debug/build/feap_fenda-218cb84ee14d149b/out",
        //     );
        //     std::env::set_var("TARGET", "aarch64-apple-darwin");
        //     std::env::set_var("HOST", "aarch64-apple-darwin");
        //     std::env::set_var("OPT_LEVEL", "0");
        // }

        match &self.out_dir {
            Some(p) => Ok(Cow::Borrowed(&**p)),
            None => self
                .getenv("OUT_DIR")
                .as_deref()
                .map(PathBuf::from)
                .map(Cow::Owned)
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::EnvVarNotFound,
                        "Environment variable OUT_DIR not defined",
                    )
                }),
        }
    }

    fn getenv(&self, v: &str) -> Option<Arc<OsStr>> {
        // Returns true for environment variables cargo sets for build scripts:
        // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
        fn provided_by_cargo(envvar: &str) -> bool {
            match envvar {
                v if v.starts_with("CARGO") || v.starts_with("RUSTC") => true,
                "HOST" | "TARGET" | "RUSTDOC" | "OUT_DIR" | "OPT_LEVEL" | "DEBUG" | "PROFILE"
                | "NUM_JOBS" | "RUSTFLAGS" => true,
                _ => false,
            }
        }

        if let Some(val) = self.build_cache.env_cache.read().unwrap().get(v).cloned() {
            return val;
        }
        // Excluding `PATH` prevents spurious rebuilds on Windows, see
        // <https://github.com/rust-lang/cc-rs/pull/1215> for details.
        if self.emit_rerun_if_env_changed && !provided_by_cargo(v) && v != "PARH" {
            self.cargo_output
                .print_metadata(&format_args!("cargo:rerun-if-env-changed={v}"));
        }
        let r = self
            .env
            .iter()
            .find(|(k, _)| k.as_ref() == v)
            .map(|(_, value)| value.clone())
            .or_else(|| env::var_os(v).map(Arc::from));
        self.cargo_output.print_metadata(&format_args!(
            "{} = {}",
            v,
            OptionOsStrDisplay(r.as_deref())
        ));
        self.build_cache
            .env_cache
            .write()
            .unwrap()
            .insert(v.into(), r.clone());

        r
    }

    /// Get a single-valued environment variable with target variants
    fn getenv_with_target_prefixes(&self, env: &str) -> Result<Arc<OsStr>, Error> {
        // Take from first environment variable
        let res = self
            .target_envs(env)?
            .iter()
            .filter_map(|env| self.getenv(env))
            .next();

        match res {
            Some(res) => Ok(res),
            None => Err(Error::new(
                ErrorKind::EnvVarNotFound,
                format!("could not find environment variable {env}"),
            )),
        }
    }

    /// The list of environment variables to check for a given env, in order of priority
    fn target_envs(&self, env: &str) -> Result<[String; 4], Error> {
        let target = self.get_raw_target()?;
        let kind = if self.get_is_cross_compile()? {
            "TARGET"
        } else {
            "HOST"
        };
        let target_u = target.replace('-', "_");

        Ok([
            format!("{env}_{target}"),
            format!("{env}_{target_u}"),
            format!("{kind}_{env}"),
            env.to_string(),
        ])
    }

    fn getenv_boolean(&self, v: &str) -> bool {
        match self.getenv(v) {
            Some(s) => &*s != "0" && &*s != "false" && !s.is_empty(),
            None => false,
        }
    }

    fn getenv_flags(&self, env: &str) -> Result<Option<Vec<String>>, Error> {
        let mut any_set = false;
        let mut res: Vec<String> = Vec::new();
        for env in self.target_envs(env)?.iter().rev() {
            if let Some(var) = self.getenv(env) {
                any_set = true;
                res.extend(
                    var.to_str()
                        .unwrap()
                        .split_ascii_whitespace()
                        .map(ToString::to_string),
                );
            }
        }

        Ok(if any_set { Some(res) } else { None })
    }

    fn getenv_unwrap(&self, v: &str) -> Result<Arc<OsStr>, Error> {
        match self.getenv(v) {
            Some(s) => Ok(s),
            None => Err(Error::new(
                ErrorKind::EnvVarNotFound,
                format!("Environment variable {v} not found!"),
            )),
        }
    }

    fn getenv_unwrap_str(&self, v: &str) -> Result<String, Error> {
        let env = self.getenv_unwrap(v)?;
        env.to_str().map(String::from).ok_or_else(|| {
            Error::new(
                ErrorKind::EnvVarNotFound,
                format!("Environment variable {v} is not valid utf-8."),
            )
        })
    }

    fn fail(s: &str) -> ! {
        eprintln!("\n\nerror occurred in feap_binding: {s}\n\n");
        std::process::exit(1);
    }
}

/// Represents an object
/// This is a source file -> object file pair
#[derive(Clone, Debug)]
struct Object {
    src: PathBuf,
    dst: PathBuf,
}

impl Object {
    fn new(src: PathBuf, dst: PathBuf) -> Object {
        Object { src, dst }
    }
}
