mod dry_run;

// Re-export #[derive(Model)].
#[cfg(feature = "derive")]
extern crate dry_run_derive;

/// Derive macro available if letterflow_model is built with `features = ["derive"]`.
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use dry_run_derive::{dry_run, force, DryRun};
