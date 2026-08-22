//! Dep-graph invariant: `orcker-proxy`'s default-features runtime graph must
//! not pull `anyhow`, OpenSSL / native-tls family, `hyper-tls`,
//! `tokio-native-tls`, or `webpki-roots`. `tokio`, `hyper`, `rustls`
//! are expected. See [`orcker_depcheck`] for the shared `cargo metadata` walk.

#[test]
fn no_forbidden_crates_in_runtime_graph_and_unique_versions() {
    let graph = orcker_depcheck::DepGraph::for_package("orcker-proxy");
    graph.assert_none_of(&[
        "anyhow",
        "openssl",
        "openssl-sys",
        "native-tls",
        "hyper-tls",
        "tokio-native-tls",
        "webpki-roots",
    ]);
    graph.assert_at_most_one_version_each(&["hyper", "rustls", "tokio", "time"]);
}
