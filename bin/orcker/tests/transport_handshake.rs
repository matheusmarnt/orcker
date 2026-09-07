//! `transport::exchange_at`'s connect-time `Hello`/`Welcome` preflight
//! (SPEC-0034): transparent when the daemon is compatible, and a
//! short-circuit - the real request is never sent - when it is not.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

#[cfg(unix)]
mod tests {
    use std::time::Duration;

    use tokio::sync::watch;

    use orcker_ipc::{ErrorCode, Request, Response};

    fn make_dirs(tmp: &std::path::Path) -> orcker_platform::PlatformDirs {
        orcker_platform::PlatformDirs {
            config: tmp.join("c"),
            data: tmp.join("d"),
            state: tmp.join("s"),
            cache: tmp.join("ca"),
            runtime: tmp.join("r"),
        }
    }

    fn default_config() -> orcker_config::Config {
        let mut cfg = orcker_config::Config::default();
        cfg.ports.http = 0;
        cfg.ports.https = 0;
        cfg.dns_port = 0;
        cfg
    }

    /// The common case: a same-build daemon replies `Welcome`, and the real
    /// request goes through exactly as it did before this spec - the
    /// preflight is invisible to every existing CLI command.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn compatible_hello_is_transparent_to_the_real_exchange() {
        let tmp = tempfile::tempdir().unwrap();
        let dirs = make_dirs(tmp.path());
        let cfg_path = dirs.config.join("orcker.toml");

        let daemon = orckerd::startup::bring_up_with_dirs(dirs.clone(), default_config(), cfg_path)
            .await
            .expect("bring_up_with_dirs");
        let sock = dirs.runtime.join("orcker.sock");

        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let ipc_task = tokio::spawn(orckerd::ipc_server::run(
            daemon.ipc_listener,
            daemon.state.clone(),
            shutdown_rx,
        ));
        let keep_alive = (
            daemon.lock,
            daemon.dns_bound,
            daemon.http_listener,
            daemon.https_listener,
        );
        tokio::time::sleep(Duration::from_millis(100)).await;

        let resp = orcker::transport::exchange_at(&sock, &Request::Ping)
            .await
            .expect("exchange_at");
        assert!(matches!(resp, Response::Pong), "got {resp:?}");

        drop(keep_alive);
        shutdown_tx.send_replace(true);
        let _ = tokio::time::timeout(Duration::from_secs(10), ipc_task).await;
    }

    /// A daemon that answers `Hello` with anything other than `Welcome` (here,
    /// a fake listener standing in for a genuinely mismatched daemon) must
    /// never receive the real request - `exchange_at` returns the mismatch
    /// reply immediately instead.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn mismatched_hello_short_circuits_without_sending_the_real_request() {
        use interprocess::local_socket::tokio::Listener;
        use interprocess::local_socket::traits::tokio::{Listener as _, Stream as _};
        use interprocess::local_socket::{GenericFilePath, ListenerOptions, ToFsName};
        use orcker_ipc::{read_message, write_message, FrameDecoder, DEFAULT_MAX_FRAME};

        let tmp = tempfile::tempdir().unwrap();
        let sock = tmp.path().join("fake.sock");
        let name = sock.clone().to_fs_name::<GenericFilePath>().unwrap();
        let listener: Listener = ListenerOptions::new().name(name).create_tokio().unwrap();

        let server = tokio::spawn(async move {
            let stream = listener.accept().await.expect("accept");
            let (reader, writer) = stream.split();
            let mut reader = reader;
            let mut writer = writer;
            let mut decoder = FrameDecoder::new();

            let hello: Request = read_message(&mut reader, &mut decoder)
                .await
                .expect("read")
                .expect("hello frame");
            assert!(matches!(hello, Request::Hello { .. }), "got {hello:?}");

            write_message(
                &mut writer,
                &Response::Error {
                    code: ErrorCode::VersionMismatch,
                    message: "fake mismatch for test".into(),
                },
                DEFAULT_MAX_FRAME,
            )
            .await
            .expect("write mismatch reply");

            // `exchange_at` must give up on the connection after a non-Welcome
            // reply: the next read must see a clean close, never a second
            // (real-request) frame.
            let second = read_message::<_, Request>(&mut reader, &mut decoder).await;
            assert!(
                matches!(second, Ok(None)),
                "expected the client to close after the mismatch, got {second:?}"
            );
        });

        let resp = orcker::transport::exchange_at(&sock, &Request::Ping)
            .await
            .expect("exchange_at");
        assert!(
            matches!(
                resp,
                Response::Error {
                    code: ErrorCode::VersionMismatch,
                    ..
                }
            ),
            "got {resp:?}"
        );

        server.await.expect("server task panicked");
    }
}
