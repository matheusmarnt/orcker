//! Loopback port allocation for container projects.
//!
//! A container project's nginx publishes on a host port bound to `127.0.0.1`
//! only, and the proxy forwards `<site>.<tld>` to it. The port must be stable
//! across daemon restarts, so it is allocated once and persisted; this module
//! owns the pure choice of *which* port, with the "is anything listening?"
//! question pushed behind [`PortProbe`].

use std::collections::BTreeSet;

use crate::error::CoreError;

/// First port of the container-project range.
pub const FIRST_PROJECT_PORT: u16 = 20000;

/// Last port of the container-project range (inclusive).
pub const LAST_PROJECT_PORT: u16 = 29999;

/// Answers whether a loopback port is currently free.
///
/// The real implementation binds `127.0.0.1:<port>` and drops the listener;
/// that is an I/O effect, so it lives at the daemon edge. Tests inject a fake.
pub trait PortProbe {
    /// Whether nothing is listening on `127.0.0.1:<port>` right now.
    fn is_free(&self, port: u16) -> bool;
}

/// Picks the lowest port in `FIRST_PROJECT_PORT..=LAST_PROJECT_PORT` that is
/// neither already allocated (`taken`) nor reported busy by `probe`.
///
/// Deterministic: the same `taken` set and the same probe answers always yield
/// the same port. `taken` carries the persisted allocations, so a port stays
/// reserved for its project even while that project's containers are down.
///
/// # Errors
///
/// [`CoreError::PortRangeExhausted`] when every port in the range is taken or
/// busy.
pub fn allocate_port(taken: &BTreeSet<u16>, probe: &dyn PortProbe) -> Result<u16, CoreError> {
    (FIRST_PROJECT_PORT..=LAST_PROJECT_PORT)
        .find(|port| !taken.contains(port) && probe.is_free(*port))
        .ok_or(CoreError::PortRangeExhausted {
            first: FIRST_PROJECT_PORT,
            last: LAST_PROJECT_PORT,
        })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    struct Busy(BTreeSet<u16>);

    impl PortProbe for Busy {
        fn is_free(&self, port: u16) -> bool {
            !self.0.contains(&port)
        }
    }

    /// One allocation case: label, persisted ports, probe-busy ports, expected
    /// port (`None` when the range is exhausted).
    type Case = (&'static str, &'static [u16], &'static [u16], Option<u16>);

    fn set(ports: &[u16]) -> BTreeSet<u16> {
        ports.iter().copied().collect()
    }

    #[test]
    fn allocation_matrix() {
        let cases: &[Case] = &[
            (
                "empty range hands out the first port",
                &[],
                &[],
                Some(20000),
            ),
            ("skips a persisted allocation", &[20000], &[], Some(20001)),
            (
                "skips a port the probe reports busy",
                &[],
                &[20000],
                Some(20001),
            ),
            (
                "skips persisted and busy together",
                &[20000, 20001, 20003],
                &[20002, 20004],
                Some(20005),
            ),
            (
                "a busy port below the range is irrelevant",
                &[],
                &[19999],
                Some(20000),
            ),
        ];

        for (label, taken, busy, expected) in cases {
            let taken = set(taken);
            let probe = Busy(set(busy));
            let got = allocate_port(&taken, &probe).ok();
            assert_eq!(got, *expected, "{label}");
            assert_eq!(
                allocate_port(&taken, &probe).ok(),
                got,
                "{label}: allocation must be deterministic"
            );
        }
    }

    #[test]
    fn exhausted_range_is_a_typed_error() {
        let taken: BTreeSet<u16> = (FIRST_PROJECT_PORT..=LAST_PROJECT_PORT).collect();
        let probe = Busy(BTreeSet::new());
        assert_eq!(
            allocate_port(&taken, &probe),
            Err(CoreError::PortRangeExhausted {
                first: FIRST_PROJECT_PORT,
                last: LAST_PROJECT_PORT,
            })
        );
    }

    #[test]
    fn allocation_never_leaves_the_range() {
        let taken: BTreeSet<u16> = (FIRST_PROJECT_PORT..LAST_PROJECT_PORT).collect();
        let probe = Busy(BTreeSet::new());
        assert_eq!(allocate_port(&taken, &probe), Ok(LAST_PROJECT_PORT));
    }
}
