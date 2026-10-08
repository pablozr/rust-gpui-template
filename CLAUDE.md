# Rules

For anyone changing this repository, human or agent. Every rule here is something that is easy to get wrong. Architecture: docs/ARCHITECTURE.md. Decisions: docs/adr/. Vocabulary: docs/CONTEXT.md.

## Done means verified
- Run `./tools/check.ps1` (Windows) or `tools/check.sh`: rustfmt check, clippy on the whole workspace with `-D warnings`, every test including `tests/architecture`. Never clippy only the crate you touched.
- Report evidence: the command and its real result. If a check could not run, say which and why.
- UI changes are not done when they compile: run `cargo run -p myapp` and look at the changed surface in both themes.
- Name the test that would fail if the behavior regressed. If none would, write it.
- Small commits, one topic, Conventional Commits. Code, tests and docs for a change go in the same commit. `Cargo.lock` is committed; checks run with `--locked`.

## Ask before
Propose options and a recommendation, then wait, before: adding or replacing a dependency; adding a crate, layer or cross-layer interface; a new pattern (concurrency, errors, state, persistence); bumping the GPUI rev; deviating from an ADR. When approved, add a short ADR (docs/adr/0000-template.md) in the same commit and add the new crate's row to the `LAYERS` table in `tests/architecture`.

## Layers
- `myapp-domain`: types and rules. No I/O, async, serde or GPUI.
- `myapp-core`: use cases, ports (traits), `Problem`. Synchronous; does I/O only through ports. Re-exports the domain.
- `myapp-storage`: adapters implementing ports. Wired only in `apps/myapp/src/main.rs`.
- `myapp-ui`: tokens, theme, primitives. Knows no business types.
- `apps/myapp`: composition root and feature views. Views turn user intent into core calls and render state; business rules never live in a view.
- A feature view keeps its state as plain data in `feature/state.rs`, tested without GPUI.
- Dependencies point inward; `tests/architecture` enforces edges, GPUI confinement, I/O-free layers, adapter wiring, color literals, file length and no `mod.rs`.

## Rust
- Errors: a `thiserror` enum per library crate; `anyhow` only in the app binary. Propagate with `?`. The core maps failures to `Problem`; views show `i18n::problems::describe`.
- No `unwrap`/`expect` outside tests (clippy enforces). A proven invariant may use `expect("why it holds")` under `#[expect(clippy::expect_used, reason = "...")]`.
- Never drop a `Result`: no `let _ =`, no `.ok();`. Handle it, propagate it, or log it with context.
- `#[expect(lint, reason = "...")]`, never `#[allow]`.
- No `mod.rs`: a module with children is `name.rs` plus `name/`.
- Full words in names (`index`, `button`, not `ix`, `btn`). No `Manager`, `Helper`, `Util` or `Service`: name what the thing does.
- Prefer functions and plain data. A trait needs two implementations or a test seam (a port has the adapter and the fake).
- Three similar lines beat a premature helper. No speculative generics, options or extension points.
- Public items get a one-line doc saying what, not how. Comments only explain a non-obvious why: no history, ticket numbers or narration.
- Lines up to 100 columns. rustfmt silently gives up on long expressions: extract a named function.
- `unsafe` needs an approved ADR and a `// SAFETY:` comment.
- Files stay under 600 lines (tested). Split by responsibility.

## GPUI
- The API changes between Zed revisions. Check every signature in the pinned source (`~/.cargo/git/checkouts/zed-*/2440236/crates/gpui`, examples in `crates/gpui/examples`, tests in `examples/testing.rs`), never from memory or other crates' docs.
- Argument order: `window` before `cx`, callbacks last.
- Inside a closure use the `cx` it receives, never an outer one. Never update an entity from inside its own update: it panics.
- A dropped `Task` is cancelled: keep it in a field, `.await` it, or `.detach_and_log_err(cx)`. Replacing a stored task cancels the old one; do it on purpose.
- Call `cx.notify()` after changing state that `render` reads; never inside `render`.
- `render` only reads state and builds elements: no I/O, no business logic, no cloned collections, no spawned tasks.
- Blocking or heavy work goes to `cx.background_spawn`; results come back with `this.update(cx, ..)` inside `cx.spawn`.
- Regions that redraw often (lists, timers) get their own entity.
- Ids of repeated elements come from domain ids (`("item", id.get())`), never from positions.
- Long lists use `uniform_list`; inside a flex parent it needs `.flex_1().min_h_0()` or it never scrolls.
- Every user action is a GPUI action with a key binding in a key context, registered in the feature's `bind_keys`.
- Tests: plain-data state first; `#[gpui::test]` with `add_window_view`, `simulate_keystrokes` and `run_until_parked` for actions, focus and async.

## UI
- Raw values (colors, sizes, spacing, radii, durations) exist only in `crates/myapp-ui/src/tokens.rs`, for both themes. Views read them through `Theme::of(cx)` and the token modules.
- Reuse `myapp-ui` primitives; a new pattern goes there, not into a view.
- Every surface handles loading, empty (saying what fills it), recoverable error (with a real action) and content.
- Every control is keyboard-focusable with a visible focus ring and has an `aria_label`; icon-only controls also get a tooltip.
- Every string a person reads comes from `i18n`. A missing translation is a compile error; keep it that way.

## No slop
- Do what was asked. No unrequested features, refactors, files, abstractions, options or docs.
- No placeholder or commented-out code, no `todo!()`, no invented data shown as real.
- No tautological tests: assert behavior a caller relies on, not constants or the fake itself.
- Docs describe what exists. Delete or fix docs that stop matching the code in the same commit.
- No dependency for what a few lines of std do.

## Windows
- Smart App Control can block freshly built build scripts or proc-macro DLLs (`os error 4551`, sometimes `E0463`). Find the blocked artifact before acting; never disable SAC or delete other projects' build output. Avoid `build.rs`.
- The first GPUI build takes minutes. `cargo run` and `cargo test` build GPUI with different features, so switching between them recompiles it once.
