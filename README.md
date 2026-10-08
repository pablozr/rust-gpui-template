<!-- template:start -->
# Rust + GPUI Desktop Template

A production-ready starting point for native desktop apps in Rust with [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui), the UI framework behind the Zed editor. It ships a small, complete app instead of an empty skeleton, so every layer, convention and check is already wired and tested.

[![CI](https://github.com/pablozr/rust-gpui-template/actions/workflows/ci.yml/badge.svg)](https://github.com/pablozr/rust-gpui-template/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust 1.98+](https://img.shields.io/badge/rust-1.98%2B-orange.svg)
![Platforms](https://img.shields.io/badge/platforms-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)

## Highlights

- **Layered architecture, enforced by tests.** Domain, core, adapters, UI kit and app are separate crates. `tests/architecture` fails the build when a layer reaches where it should not: GPUI outside the UI, I/O in the domain, colors outside the design tokens, files over 600 lines.
- **A real feature end to end.** A persisted item list with keyboard navigation, async loading and saving off the UI thread, coalesced writes, a flush on quit, loading/empty/error states and English/Portuguese strings.
- **Strict by default.** Workspace-wide lints (no `unwrap`, no silently dropped `Result`, no `#[allow]` without a reason), clippy with `-D warnings`, unit, GPUI and property tests.
- **Ready for coding agents.** [CLAUDE.md](CLAUDE.md) states the rules for humans and agents alike: how to verify work, what needs approval, Rust and GPUI pitfalls, and what counts as slop.
- **CI and releases included.** Checks on Windows, macOS and Linux, dependency and workflow audits, and installers (NSIS, deb, AppImage, dmg) built on every version tag.

## Use this template

1. Click **Use this template** on GitHub, then clone your new repository.
2. Rename the project. The working tree must be clean:

   ```powershell
   powershell -File tools/rename.ps1 -Name acme-notes -DisplayName "Acme Notes" -Identifier com.example.acmenotes
   ```

   Add `-DryRun` to preview. The script renames crates, folders, environment variables and the app identifier, and removes this section of the README.
3. Run the checks with `./tools/check.ps1` (Windows) or `tools/check.sh` (Linux, macOS).
4. Commit, and start replacing the example feature with your own.

The full checklist, including what to set before the first release, is in [docs/TEMPLATE.md](docs/TEMPLATE.md).
<!-- template:end -->

## Requirements

| Platform | Needs |
|---|---|
| All | Rust stable (installed automatically from `rust-toolchain.toml`) |
| Windows | Visual Studio Build Tools with the MSVC toolchain and Windows SDK 10.0.20348 or newer |
| macOS | Xcode |
| Linux | The packages below |

```sh
sudo apt-get install -y gcc g++ clang libfontconfig-dev libwayland-dev libxkbcommon-dev \
  libxkbcommon-x11-dev libx11-xcb-dev libxcb1-dev libvulkan-dev libzstd-dev libssl-dev libasound2-dev
```

## Getting started

```sh
cargo run -p myapp
```

The first build takes a few minutes because it compiles GPUI. Later builds reuse it.

| Environment variable | Effect | Example |
|---|---|---|
| `MYAPP_LANG` | Interface language | `en`, `pt-BR` |
| `RUST_LOG` | Log filter | `myapp=debug` |

### Keyboard shortcuts

`Ctrl` on Windows and Linux, `Cmd` on macOS.

| Keys | Action |
|---|---|
| `Ctrl+N` | Add an item |
| `Up` / `Down` | Move the selection |
| `Space` | Mark the selected item as done or not done |
| `Delete` / `Backspace` | Remove the selected item |
| `Ctrl+R` | Retry a failed load or save |
| `Tab` / `Shift+Tab` | Move the keyboard focus |
| `Ctrl+Shift+T` | Switch between dark and light theme |
| `Ctrl+Q` | Quit |

## Project structure

```
apps/myapp/              Composition root and feature views (GPUI)
crates/myapp-domain/     Types and rules: no I/O, no async, no UI
crates/myapp-core/       Use cases, ports (traits) and the Problem type
crates/myapp-storage/    Adapters: the JSON file store
crates/myapp-ui/         Design tokens, theme and UI primitives
tests/architecture/      Tests that keep the layers apart
tools/                   Check and rename scripts
docs/                    Architecture, vocabulary and decision records
```

## Architecture

Dependencies point inward. The domain knows nothing, the core knows the domain, adapters implement the core's ports, and only the app knows every crate.

```
apps/myapp ──► myapp-ui
     │
     ├──────► myapp-storage ──┐
     │                        ▼
     └──────────────────► myapp-core ──► myapp-domain
```

Each crate has one row in the `LAYERS` table of `tests/architecture`, which lists what it may depend on and whether it may use GPUI or do I/O. Adding a crate means adding its row. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) explains how a command flows through the layers and how to add a feature.

## Quality checks

Run before every commit:

```sh
./tools/check.ps1    # Windows
tools/check.sh       # Linux and macOS
```

Both run rustfmt, clippy with `-D warnings` and every test, including the architecture tests.

| Where | What runs |
|---|---|
| Local | rustfmt, clippy, unit, GPUI, property and architecture tests |
| CI, every push and pull request | The local checks on Windows, macOS and Linux, plus [cargo-deny](https://github.com/EmbarkStudios/cargo-deny), [typos](https://github.com/crate-ci/typos) and [zizmor](https://github.com/zizmorcore/zizmor) |
| CI, nightly (opt-in) | [cargo-mutants](https://mutants.rs) on the crates without GPUI; set the repository variable `MUTANTS_NIGHTLY=true` to enable it |
| CI, version tags (`v*`) | Installers built with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager) |

## Documentation

| Document | Contents |
|---|---|
| [CLAUDE.md](CLAUDE.md) | Rules for contributors and coding agents |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Layers, command flow and how to add a feature |
| [docs/CONTEXT.md](docs/CONTEXT.md) | Domain vocabulary |
| [docs/adr/](docs/adr) | Architecture decision records |

## Troubleshooting

- **GPUI recompiles when switching between `cargo run` and `cargo test`.** They build GPUI with different features, so the first switch rebuilds it once.
- **`os error 4551` on Windows.** Smart App Control blocked a freshly built build script or proc-macro DLL. Find the file named in the error and rebuild that package once. Never turn the protection off.
- **The build fails on Linux with missing `wayland`, `xkbcommon` or `vulkan` libraries.** Install the packages listed under Requirements.

## License

[MIT](LICENSE)
