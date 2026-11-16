//! Problem and crash reports (design agreed in the Worker reports, 02:56 UTC).
//!
//! A report is one small JSON document, format 1, written to the Capsule's
//! `reports/pending/`. This crate captures and keeps reports; it sends
//! nothing. Where reports go is not decided, so nothing leaves the Mac.
//!
//! - The format and what never goes in a report come from the SDK's
//!   `isonomic-reports`, shared with the engine, and are re-exported here.
//! - [`store::Store`]: pending reports and their retention.
//! - [`settings`]: the "Problem reports" setting; [`kept::Kept`]: both, for the notice.
//! - [`guard::LaunchGuard`]: the crash-loop guard.
//! - [`log::WindowLog`]: the window's own log, whose tail a report carries.
//! - [`panic`]: the panic hook; [`watchdog`]: hangs.

pub mod guard;
pub mod kept;
pub mod log;
pub mod panic;
pub mod settings;
pub mod store;
pub mod watchdog;

pub use isonomic_reports::{
    backtrace, filter, scrub, Build, Component, Kind, Report, System, FORMAT, LOG_LINES,
};
