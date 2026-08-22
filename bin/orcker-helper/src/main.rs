//! Privileged one-shot binary for Orcker.
//!
//! The daemon (`orckerd`) runs unprivileged. Operations that require root
//! are sent here as typed `HelperInvocation`s over a frozen argv
//! contract. This binary validates everything (defence in depth),
//! performs exactly one operation, and exits with a `sysexits.h` code
//! the daemon can interpret.

#![forbid(unsafe_code)]

mod cli;
mod error;
mod exec;
mod ops;
mod privilege;
mod validate;

use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        eprintln!("orcker-helper: not supported on this OS");
        return ExitCode::from(78);
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        run()
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn run() -> ExitCode {
    let parsed = match cli::parse(std::env::args_os()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("orcker-helper: {e}");
            return ExitCode::from(error::exit_code(&e));
        }
    };

    let _ = std::env::set_current_dir("/");

    if !parsed.skip_priv_check && !privilege::is_privileged() {
        let e = error::HelperError::NotPrivileged;
        eprintln!("orcker-helper: {e}");
        return ExitCode::from(error::exit_code(&e));
    }

    if let Err(e) = exec::dispatch(parsed.invocation) {
        eprintln!("orcker-helper: {e}");
        return ExitCode::from(error::exit_code(&e));
    }
    ExitCode::SUCCESS
}
