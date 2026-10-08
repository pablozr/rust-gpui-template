# 0002. Synchronous core on the background executor

Status: accepted · 2026-10-08

## Context

The core does blocking I/O through ports. The UI thread must never wait for it, and the app has no long-lived connections or streams that would justify an async runtime.

## Decision

The core is synchronous and has no async runtime. The app runs its functions with `cx.background_spawn` and brings the result back with `this.update(cx, ..)` inside `cx.spawn`. There is no Tokio.

## Consequences

- Core tests are plain function calls against `fake::MemoryStore`, so they are deterministic.
- Saves are coalesced: while one runs, further changes only mark the state dirty (`Saving::InFlightDirty`), and one more save runs with the latest list.
- The view tests use `#[gpui::test]` with `add_window_view`, `simulate_keystrokes` and `run_until_parked`.

## Alternatives considered

- The core as an actor on a Tokio thread, talking through channels. Adopt it when the app needs long connections or streaming.
