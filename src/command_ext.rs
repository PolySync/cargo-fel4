use super::Error;
use config::LoudnessOpts;
use std::ffi::OsStr;
use std::process::Command;
/// Extension methods for `Command` instances to supply common parameters or
/// metadata
pub trait CommandExt
where
    Self: Into<Command>,
{
    /// Add an argument if a predicate returns true, largely for easier chaining
    fn arg_if<P, S: AsRef<OsStr>>(&mut self, predicate: P, arg: S) -> &mut Self
    where
        P: FnOnce() -> bool;
    /// Configures the presence of `--verbose` and `--quiet` flags
    fn add_loudness_args<'c, 'f>(&'c mut self, loudness: &'f LoudnessOpts) -> &'c mut Self;

    /// Execute a command with logging and status-code checking, discarding
    /// most output
    fn run_cmd(&mut self) -> Result<(), Error>;
}

impl CommandExt for Command {
    fn arg_if<P, S: AsRef<OsStr>>(&mut self, predicate: P, arg: S) -> &mut Self
    where
        P: FnOnce() -> bool,
    {
        if predicate() {
            self.arg(arg);
        }
        self
    }

    fn add_loudness_args<'c, 'f>(&'c mut self, loudness: &'f LoudnessOpts) -> &mut Self {
        self.arg_if(|| loudness.quiet, "--quiet")
            .arg_if(|| loudness.verbose, "--verbose")
    }

    fn run_cmd(&mut self) -> Result<(), Error> {
        info!("running: {:?}", self);
        let status = match self.status() {
            Ok(status) => status,
            Err(e) => {
                return Err(Error::ExitStatusError(format!(
                    "failed to execute the command: {}",
                    e
                )));
            }
        };

        if !status.success() {
            return Err(Error::ExitStatusError(format!(
                "command status returned: {}",
                status
            )));
        }

        Ok(())
    }
}
