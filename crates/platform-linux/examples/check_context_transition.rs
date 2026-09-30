//! Controlled metadata-only diagnostic. No keyboard devices or text-content calls.
use core_engine::context::ContextGuard;
use platform_linux::InputContext;
use std::{thread, time::Duration};

fn main() {
    thread::sleep(Duration::from_secs(45));
    let Some(context) = InputContext::connect() else {
        println!("baseline_unavailable");
        std::process::exit(1);
    };
    let Ok(initial) = context.check() else {
        println!("baseline_ineligible");
        std::process::exit(1);
    };
    let guard = ContextGuard::new(Some(initial));
    println!("baseline_ready; second snapshot in 120 seconds");
    thread::sleep(Duration::from_secs(120));
    let next = context.snapshot();
    if guard.unchanged(next.as_ref()) {
        println!("context_unchanged; transition rejection not demonstrated");
        std::process::exit(2);
    }
    println!("context_changed_or_unavailable; original replacement refused");
}
