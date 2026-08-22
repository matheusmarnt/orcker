//! Process-agnostic supervision substrate.
//!
//! Holds the parts of process supervision that are not specific to any
//! particular supervised program: the trait seams the supervisor depends on
//! ([`ProcessSpawner`], [`ChildHandle`], [`Clock`], [`HealthProbe`]), the
//! production tokio-backed implementations of the
//! infrastructure traits ([`SystemClock`], [`TokioProcessSpawner`]), the generic
//! [`Listen`] address, and the **pure** supervision state machine
//! ([`supervisor`]).
//!
//! Its only consumer is [`crate::manager`], which supervises one `cloudflared`
//! child per site. The state machine's timing/restart policy is not baked in:
//! it is supplied per call via [`supervisor::SupervisorPolicy`], so a
//! fast-to-start child and a slow-cold-boot one can drive the same logic with
//! different policies.
//!
//! Lives here rather than in a crate of its own because the tunnel is the last
//! consumer left; a second one extracts it back out.

pub mod error;
pub mod listen;
pub mod real;
pub mod supervisor;
pub mod traits;

pub use error::{ExitReason, SpawnFailureReason};
pub use listen::Listen;
pub use real::{kill_process_group, SystemClock, TokioChild, TokioProcessSpawner};
pub use supervisor::{
    backoff_for, transition, Action, Elapsed, ErrorTag, Event, KillSignal, PoolState, StopProtocol,
    SupervisorPolicy,
};
pub use traits::{ChildHandle, Clock, HealthProbe, ProcessSpawner};

// Compile-time `Send + 'static` guard for the production infrastructure impls.
const _: () = {
    const fn assert_send_static<T: Send + Sync + 'static>() {}
    assert_send_static::<TokioProcessSpawner>();
    assert_send_static::<SystemClock>();
};
