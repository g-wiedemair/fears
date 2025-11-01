use crate::{
    cargo::{self, CargoOutput, OutputKind},
    error::{Error, ErrorKind},
    tempfile::NamedTempfile,
};
use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    ffi::{OsStr, OsString},
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
    sync::RwLock,
};

pub(crate) type CompilerFamilyLookupCache = HashMap<Box<[Box<OsStr>]>, ToolFamily>;

/// Represents the family of tools this tool belongs to
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ToolFamily {
    GFortran,
    IntelIFX,
    Flang,
    LFortran,
}

/// Configurtion used to represent an invocation of a Fortran compiler
///
/// This can be used to figure out what compiler is in use, what the arguments
/// to it are, and what the environment variables look like for the compiler.
#[derive(Clone, Debug)]
pub struct Tool {
    pub(crate) path: PathBuf,
    pub(crate) args: Vec<OsString>,
    pub(crate) env: Vec<(OsString, OsString)>,
    pub(crate) family: ToolFamily,
    pub(crate) removed_args: Vec<OsString>,
    pub(crate) _has_internal_target_arg: bool,
}

impl Tool {
    pub(crate) fn new(
        path: PathBuf,
        cached_compiler_family: &RwLock<CompilerFamilyLookupCache>,
        cargo_output: &CargoOutput,
        out_dir: Option<&Path>,
    ) -> Self {
        Self::with_features(path, vec![], cached_compiler_family, cargo_output, out_dir)
    }
    pub(crate) fn with_args(
        path: PathBuf,
        args: Vec<String>,
        cached_compiler_family: &RwLock<CompilerFamilyLookupCache>,
        cargo_output: &CargoOutput,
        out_dir: Option<&Path>,
    ) -> Self {
        Self::with_features(path, args, cached_compiler_family, cargo_output, out_dir)
    }

    fn with_features(
        path: PathBuf,
        args: Vec<String>,
        cached_compiler_family: &RwLock<CompilerFamilyLookupCache>,
        cargo_output: &CargoOutput,
        out_dir: Option<&Path>,
    ) -> Self {
        fn guess_family_from_stdout(
            stdout: &str,
            _path: &Path,
            _args: &[String],
            _cargo_output: &CargoOutput,
        ) -> Result<ToolFamily, Error> {
            let flang = stdout.contains(r#""Flang detected""#);
            let gfortran = !flang && stdout.contains(r#""GNU Fortran Compiler detected""#);
            let ifx = !gfortran && stdout.contains(r#""Intel Fortran Compiler (ifx) detected"#);

            if flang {
                Ok(ToolFamily::Flang)
            } else if gfortran {
                Ok(ToolFamily::GFortran)
            } else if ifx {
                Ok(ToolFamily::IntelIFX)
            } else {
                todo!()
            }
        }

        fn detect_family_inner(
            path: &Path,
            args: &[String],
            cargo_output: &CargoOutput,
            out_dir: Option<&Path>,
        ) -> Result<ToolFamily, Error> {
            let out_dir = out_dir
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(env::temp_dir()));

            // Ensure all the parent directories exist
            fs::create_dir_all(&out_dir).map_err(|err| Error {
                kind: ErrorKind::IOError,
                message: format!("failed to create OUT_DIR '{}': {}", out_dir.display(), err)
                    .into(),
            })?;
            let mut tmp =
                NamedTempfile::new(&out_dir, "detect_compiler_family.f90").map_err(|err| {
                    Error {
                        kind: ErrorKind::IOError,
                        message: format!(
                            "failed to create detect_compiler_family.f90 temp file in '{}': {}",
                            out_dir.display(),
                            err
                        )
                        .into(),
                    }
                })?;
            let mut tmp_file = tmp.take_file().unwrap();
            tmp_file.write_all(include_bytes!("detect_compiler_family.f90"))?;
            tmp_file.flush()?;
            tmp_file.sync_data()?;
            drop(tmp_file);

            // When expanding the file, the compiler prints a lotof information to stderr
            // That is not an error, but related to expanding itself
            let mut compiler_detect_output = cargo_output.clone();
            compiler_detect_output.warnings = compiler_detect_output.debug;

            let mut cmd = Command::new(path);
            cmd.arg("-cpp").arg("-E").arg(tmp.path());

            // The -Wslash-u-filename warning is normally part of stdout
            // But with clang-cl it can be part of stderr instead and exit
            let mut captured_cargo_output = compiler_detect_output.clone();
            captured_cargo_output.output = OutputKind::Capture;
            captured_cargo_output.warnings = true;
            let mut child = cargo::spawn(&mut cmd, &captured_cargo_output)?;

            let mut out = Vec::new();
            let mut err = Vec::new();
            child.stdout.take().unwrap().read_to_end(&mut out)?;
            child.stderr.take().unwrap().read_to_end(&mut err)?;

            let status = child.wait()?;

            let stdout = if [&out, &err]
                .iter()
                .any(|o| String::from_utf8_lossy(o).contains("-Wslash-u-filename"))
            {
                todo!()
            } else {
                if !status.success() {
                    return Err(Error::new(
                        ErrorKind::ToolExecError,
                        format!(
                            "command did not execute successfully (status code {status}): {cmd:?}"
                        ),
                    ));
                }
                out
            };

            let stdout = String::from_utf8_lossy(&stdout);
            guess_family_from_stdout(&stdout, path, args, cargo_output)
        }

        let detect_family = |path: &Path, args: &[String]| -> Result<ToolFamily, Error> {
            let cache_key: Box<[Box<OsStr>]> = [path.as_os_str()]
                .iter()
                .cloned()
                .chain(args.iter().map(OsStr::new))
                .map(Into::into)
                .collect();
            if let Some(family) = cached_compiler_family.read().unwrap().get(&cache_key) {
                return Ok(*family);
            }

            let family = detect_family_inner(path, args, cargo_output, out_dir)?;
            cached_compiler_family
                .write()
                .unwrap()
                .insert(cache_key, family);
            Ok(family)
        };

        let family = detect_family(&path, &args).unwrap_or_else(|_e| todo!());

        Tool {
            path,
            args: Vec::new(),
            env: Vec::new(),
            family,
            removed_args: Vec::new(),
            _has_internal_target_arg: false,
        }
    }

    /// Converts this compiler into a `Command`
    pub fn to_command(&self) -> Command {
        let mut cmd = Command::new(&self.path);
        let value = self
            .args
            .iter()
            .filter(|a| !self.removed_args.contains(a))
            .collect::<Vec<_>>();
        cmd.args(&value);

        for (k, v) in self.env.iter() {
            cmd.env(k, v);
        }
        cmd
    }
    
    pub fn is_like_msvc(&self) -> bool {
        false
    }
    
    pub fn is_like_gnu(&self) -> bool {
        self.family == ToolFamily::GFortran
    }
    
    pub fn is_like_clang(&self) -> bool {
        self.family == ToolFamily::Flang
    }
    
    pub fn is_like_intel(&self) -> bool {
        self.family == ToolFamily::IntelIFX
    }
    
    pub fn is_like_llvm(&self) -> bool {
        self.family == ToolFamily::LFortran
    }

    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    pub(crate) fn push_opt_unless_duplicate(&mut self, flag: OsString) {
        if self.is_duplicate_opt_arg(&flag) {
            eprintln!("Info: Ignoring duplicate option {:?}", &flag);
        } else {
            self.push_fc_arg(flag);
        }
    }

    fn is_duplicate_opt_arg(&self, flag: &OsString) -> bool {
        let flag = flag.to_str().unwrap();
        let mut chars = flag.chars();

        if chars.next() != Some('-') {
            return false;
        }

        // Check for existing optimization flags
        if chars.next() == Some('O') {
            return self
                .args()
                .iter()
                .any(|a| a.to_str().unwrap_or("").chars().nth(1) == Some('O'));
        }

        false
    }

    pub(crate) fn push_fc_arg(&mut self, flag: OsString) {
        self.args.push(flag);
    }
}

impl ToolFamily {
    pub(crate) fn warnings_to_errors_flag(&self) -> &'static str {
        match self {
            ToolFamily::GFortran => "-Werror",
            _ => "",
        }
    }

    pub(crate) fn warning_flags(&self) -> Option<&'static str> {
        match self {
            ToolFamily::GFortran => Some("-Wall"),
            _ => None,
        }
    }

    pub(crate) fn extra_warning_flags(&self) -> Option<&'static str> {
        match self {
            ToolFamily::GFortran => Some("-Wextra"),
            _ => None,
        }
    }
}
