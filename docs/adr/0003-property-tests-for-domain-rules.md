# 0003. Property tests for domain rules

Status: accepted · 2026-10-08

## Context

Example tests cover only the cases someone imagined. The domain rules (title limits, ids that are unique and never reused, validation when restoring a list) must hold for any input and any sequence of operations. "Towards Impeccable Rust" (Jon Gjengset, Rust Nation UK 2024), condensed in https://github.com/hexuria/impeccable-rust (MIT), asks every test to name its oracle: the independent source of the expected result.

## Decision

`proptest` from crates.io, dev-dependency of `myapp-domain` only.

## Consequences

- `Cargo.lock` holds two copies of `proptest`: the GPUI test support pulls one from git. `cargo deny` warns about multiple versions; `deny.toml` allows that.
- Seeds that fail are saved in `crates/myapp-domain/proptest-regressions/`; commit them when they record a real bug.

## Alternatives considered

- `quickcheck`: weaker shrinking.
- `proptest` at the same git rev as Zed: ties the domain tests to every GPUI bump.
- `gpui::proptest`: needs GPUI and its test support in the domain, which the layers forbid.
