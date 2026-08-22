//! Dep-graph invariant: nothing in `orcker-platform`'s own runtime graph
//! drags in `tokio`, `anyhow`, `reqwest`, or any OpenSSL/native-tls
//! variant. `tokio` already exists elsewhere in the workspace (via
//! `orcker-ipc`'s transport feature), so this assertion is scoped to
//! `orcker-platform`'s own reachable set. See [`orcker_depcheck`] for the
//! shared `cargo metadata` walk.

#[test]
fn no_forbidden_crates_in_runtime_graph() {
    orcker_depcheck::DepGraph::for_package("orcker-platform").assert_none_of(&[
        "tokio",
        "anyhow",
        "reqwest",
        "openssl",
        "openssl-sys",
        "native-tls",
    ]);
}
