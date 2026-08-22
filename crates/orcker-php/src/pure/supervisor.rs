//! Re-export of the shared pure supervision state machine.
//!
//! The state machine moved to `orcker-supervise` so `orcker-services` can drive it
//! too; it is re-exported here so existing `crate::pure::supervisor::*` paths
//! (and the `orcker_php` public API) are unchanged.

pub use orcker_supervise::supervisor::*;
