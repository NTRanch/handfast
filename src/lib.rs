//! handfast — Handfast core (Rust, wasm32).
//! Tiger Style (tiger-style-rust): explicit contracts, bounded work,
//! disciplined ownership. Fixed-step evaluation; deterministic re-runs.

/// One fixed simulation step (1/60 s, fixed by the shell).
pub fn step() {}

#[cfg(test)]
mod tests {
    #[test]
    fn step_is_callable() {
        super::step();
    }
}
