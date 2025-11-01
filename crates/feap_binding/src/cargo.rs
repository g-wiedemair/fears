use crate::error::{Error, ErrorKind};
use std::fmt::Display;
use std::io;
use std::io::{Read, Write};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Clone, Debug)]
pub(crate) struct CargoOutput {
    pub(crate) metadata: bool,
    pub(crate) warnings: bool,
    pub(crate) debug: bool,
    pub(crate) output: OutputKind,
    checked_dbg_var: Arc<AtomicBool>,
}

/// Different strategies for handling compiler output (to stdout)
#[derive(Clone, Debug)]
pub(crate) enum OutputKind {
    /// Forward the output to this process' stdout
    Forward,
    /// Discard the output
    Discard,
    /// Capture the result
    Capture,
}

impl CargoOutput {
    pub(crate) fn new() -> CargoOutput {
        Self {
            metadata: true,
            warnings: true,
            output: OutputKind::Forward,
            debug: match std::env::var_os("CC_ENABLE_DEBUG_OUTPUT") {
                Some(v) => v != "0" && v != "false" && v != "",
                None => false,
            },
            checked_dbg_var: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn print_metadata(&self, s: &dyn Display) {
        if self.metadata {
            println!("{s}");
        }
    }

    pub(crate) fn print_debug(&self, arg: &dyn Display) {
        if self.metadata
            && self
                .checked_dbg_var
                .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
        {
            println!("cargo:rerun-if-env-changed=FC_ENABLE_DEBUG_OUTPUT");
        }
        if self.debug {
            println!("{arg}");
        }
    }

    fn stdio_for_warnings(&self) -> Stdio {
        if self.warnings {
            Stdio::piped()
        } else {
            Stdio::null()
        }
    }

    fn stdio_for_output(&self) -> Stdio {
        match self.output {
            OutputKind::Capture => Stdio::piped(),
            OutputKind::Forward => Stdio::inherit(),
            OutputKind::Discard => Stdio::null(),
        }
    }
}

pub(crate) fn spawn(cmd: &mut Command, cargo_output: &CargoOutput) -> Result<Child, Error> {
    struct ResetStderr<'cmd>(&'cmd mut Command);

    impl Drop for ResetStderr<'_> {
        fn drop(&mut self) {
            self.0.stderr(Stdio::inherit());
        }
    }

    cargo_output.print_debug(&format_args!("running: {cmd:?}"));

    let cmd = ResetStderr(cmd);
    let child = cmd
        .0
        .stderr(cargo_output.stdio_for_warnings())
        .stdout(cargo_output.stdio_for_output())
        .spawn();
    match child {
        Ok(child) => Ok(child),
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
            let extra = if cfg!(windows) {
                " (see https://docs.rs/cc/latest/cc/#compile-time-requirements for help)"
            } else {
                ""
            };
            Err(Error::new(
                ErrorKind::ToolNotFound,
                format!("failed to find tool {:?}: {e}{extra}", cmd.0.get_program()),
            ))
        }
        Err(e) => Err(Error::new(
            ErrorKind::ToolExecError,
            format!("command `{:?}` failed to start: {e}", cmd.0),
        )),
    }
}

pub(crate) struct StderrForwarder {
    inner: Option<(ChildStderr, Vec<u8>)>,
    bytes_buffered: usize,
}

const MIN_BUFFER_CAPACITY: usize = 100;

impl StderrForwarder {
    pub(crate) fn new(child: &mut Child) -> Self {
        Self {
            inner: child
                .stderr
                .take()
                .map(|stderr| (stderr, Vec::with_capacity(MIN_BUFFER_CAPACITY))),
            bytes_buffered: 0,
        }
    }

    fn write_warning(line: &[u8]) {
        let stdout = std::io::stdout();
        let mut stdout = stdout.lock();
        stdout.write_all(b"cargo:warnings=").unwrap();
        stdout.write_all(line).unwrap();
        stdout.write_all(b"\n").unwrap();
    }

    fn forward_available(&mut self) -> bool {
        if let Some((stderr, buffer)) = self.inner.as_mut() {
            loop {
                #[cfg(not(feature = "parallel"))]
                let to_reserve = MIN_BUFFER_CAPACITY;

                if self.bytes_buffered + to_reserve > buffer.len() {
                    buffer.resize(self.bytes_buffered + to_reserve, 0);
                }

                match stderr.read(&mut buffer[self.bytes_buffered..]) {
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        // No data currently, yield back
                        break false;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {
                        // Interrupted, try again
                        continue;
                    }
                    Ok(bytes_read) if bytes_read != 0 => {
                        self.bytes_buffered += bytes_read;
                        let mut consumed = 0;
                        for line in buffer[..self.bytes_buffered].split_inclusive(|&b| b == b'\n') {
                            if let Some((b'\n', line)) = line.split_last() {
                                consumed += line.len() + 1;
                                Self::write_warning(line);
                            }
                        }
                        if consumed > 0 && consumed < self.bytes_buffered {
                            // Remove the consumed bytes from the buffer
                            buffer.copy_within(consumed.., 0);
                        }
                        self.bytes_buffered -= consumed;
                    }
                    res => {
                        // End of stream>: flush remaining data
                        if self.bytes_buffered > 0 {
                            Self::write_warning(&buffer[..self.bytes_buffered]);
                        }
                        if let Err(err) = res {
                            Self::write_warning(format!("Failed to read from child stderr: {err}").as_bytes());
                        }
                        self.inner.take();
                        break true;
                    }
                }
            }
        } else {
            true
        }
    }

    #[cfg(not(feature = "parallel"))]
    fn forward_all(&mut self) {
        let forward_result = self.forward_available();
        assert!(forward_result, "Should have consumed all data");
    }
}

pub(crate) fn wait_on_child(
    cmd: &Command,
    child: &mut Child,
    cargo_output: &CargoOutput,
) -> Result<(), Error> {
    StderrForwarder::new(child).forward_all();

    let status = match child.wait() {
        Ok(s) => s,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::ToolExecError,
                format!("failed to wait on spawned child process `{cmd:?}`: {e}",),
            ));
        }
    };

    cargo_output.print_debug(&status);

    if status.success() {
        Ok(())
    } else {
        Err(Error::new(
            ErrorKind::ToolExecError,
            format!("command did not execute successfully (status code {status}): {cmd:?}"),
        ))
    }
}
