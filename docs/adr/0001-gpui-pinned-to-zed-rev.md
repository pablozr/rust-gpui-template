# 0001. GPUI pinned to a Zed revision

Status: accepted · 2026-10-08

## Context

GPUI changes between Zed commits, and the crates.io release is behind. Several apps from the same author build on one revision, so a component or a fix found in one applies to the others.

## Decision

`gpui` and `gpui_platform` come from the Zed repository at revision `244023605536a412ab6b8d5b658466b89fb15401`, with `gpui_platform` features `font-kit`, `wayland` and `x11`. Both lines in `[workspace.dependencies]` always carry the same revision.

## Consequences

- Every GPUI signature must be read from the pinned source, not from memory.
- The first build compiles GPUI and takes minutes.
- `cargo test` enables the `test-support` feature of `gpui`, which pulls `proptest` from git; `deny.toml` allows the `proptest-rs` organisation for it.

### Bumping the revision

1. Open a dedicated pull request.
2. Change the revision on both lines of `[workspace.dependencies]`.
3. Run `cargo update -p gpui -p gpui_platform`.
4. Read `git log <old>..<new> -- crates/gpui` in the checkout under `~/.cargo/git/checkouts/zed-*`.
5. Run the checks, and run the app on every OS.
6. Keep the other apps on the same revision in step.

### Toolchain

`rust-toolchain.toml` follows `stable`; the minimum supported version is `rust-version = "1.98"`, the toolchain Zed used at this revision. To pin an exact compiler, set `channel` to a version such as `"1.99.0"`.

## Alternatives considered

- `gpui` 0.2.2 from crates.io: last published in October 2025.
- `gpui-pre` 0.3.8: a weekly snapshot of Zed main that needs an exact `=` pin, together with `gpui-kit` 0.7.1 for ready components. Its types do not mix with the `gpui` of the Zed repository.
