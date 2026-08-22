//! Dep-graph invariant: nothing in `orcker-helper`'s own runtime graph
//! drags in `tokio`, `reqwest`, or any OpenSSL/native-tls variant.
//! `anyhow` is allowed in binaries per project convention. See
//! [`orcker_depcheck`] for the shared `cargo metadata` walk.

#[test]
fn no_forbidden_crates_in_runtime_graph() {
    orcker_depcheck::DepGraph::for_package("orcker-helper").assert_none_of(&[
        "tokio",
        "reqwest",
        "openssl",
        "openssl-sys",
        "native-tls",
    ]);
}
