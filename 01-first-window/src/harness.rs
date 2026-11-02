//! What every step's `--test` mode needs: a check counter and a summary.
//!
//! This file is repeated in every step on purpose: each folder is a complete
//! project you can copy and run on its own.
//!
//! The checks run against the Rust model, not a window, so `--test` needs
//! neither a display nor a Slint backend.

#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

static CHECKS: AtomicUsize = AtomicUsize::new(0);
static FAILURES: AtomicUsize = AtomicUsize::new(0);

pub fn is_test() -> bool {
    std::env::args().any(|argument| argument == "--test")
}

/// One assertion. The counts are printed by [`finish`].
pub fn check(condition: bool, message: &str) {
    CHECKS.fetch_add(1, Ordering::Relaxed);
    if !condition {
        FAILURES.fetch_add(1, Ordering::Relaxed);
        println!("  FAIL: {message}");
    }
}

/// A condition the rest of the step depends on: stop here when it is false.
pub fn require(condition: bool, message: &str) {
    check(condition, message);
    if !condition {
        std::process::exit(1);
    }
}

/// Prints the result and returns the process exit code.
pub fn finish(name: &str) -> i32 {
    let checks = CHECKS.load(Ordering::Relaxed);
    let failures = FAILURES.load(Ordering::Relaxed);
    if checks > 0 {
        println!("{name}: {checks} checks, {failures} failures");
        println!("{}", if failures == 0 { "PASS" } else { "FAIL" });
    }
    if failures == 0 {
        0
    } else {
        1
    }
}
