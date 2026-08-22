//! `orcker` CLI entry point. Parses args, builds a single-threaded tokio
//! runtime, and delegates to [`orcker::run`].

use std::process::ExitCode;

use clap::Parser;

use orcker::cli::Cli;

fn main() -> ExitCode {
    #[cfg(unix)]
    if let Some(code) = orcker::composer_shim::dispatch() {
        return code;
    }
    #[cfg(unix)]
    if let Some(code) = orcker::cover_shim::dispatch() {
        return code;
    }
    #[cfg(unix)]
    if let Some(code) = orcker::laravel_shim::dispatch() {
        return code;
    }
    #[cfg(unix)]
    if let Some(code) = orcker::cli_shim::dispatch() {
        return code;
    }
    #[cfg(unix)]
    if let Some(code) = orcker::wp_shim::dispatch() {
        return code;
    }
    if let Some(code) = orcker::apply::run_from_env() {
        return code;
    }
    if let Some(code) = orcker::apply::run_install_deb_from_args() {
        return code;
    }
    if let Some(code) = orcker::apply::run_install_pacman_from_args() {
        return code;
    }
    if let Some(code) = orcker::apply::run_install_rpm_from_args() {
        return code;
    }

    let cli = Cli::parse();
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("orcker: cannot build tokio runtime: {e}");
            return ExitCode::from(70);
        }
    };
    runtime.block_on(orcker::run(cli))
}
