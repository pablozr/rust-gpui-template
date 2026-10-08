# MyApp

A desktop app built with Rust and GPUI: a small item list that goes through every layer, ready to be renamed and grown.

<!-- template:start -->
Starting a new project: see [docs/TEMPLATE.md](docs/TEMPLATE.md).
<!-- template:end -->

## Prerequisites

- Windows: Visual Studio Build Tools with the MSVC toolchain and a Windows SDK 10.0.20348 or newer.
- Linux: `sudo apt-get install -y gcc g++ clang libfontconfig-dev libwayland-dev libxkbcommon-dev libxkbcommon-x11-dev libx11-xcb-dev libxcb1-dev libvulkan-dev libzstd-dev libssl-dev libasound2-dev`
- macOS: Xcode.

## Run

```powershell
cargo run -p myapp
$env:MYAPP_LANG = 'pt-BR'      # interface language: en or pt-BR
$env:RUST_LOG = 'myapp=debug'  # log filter
```

Shortcuts: `Ctrl/Cmd+N` adds an item, `Space` ticks the selected one, `Delete` removes it, `Up`/`Down` move the selection, `Ctrl/Cmd+Shift+T` switches the theme, `Ctrl/Cmd+Q` quits.

## Checks

```powershell
./tools/check.ps1    # Windows
tools/check.sh       # Linux and macOS
```

Both run rustfmt, clippy with `-D warnings` and every test, including the architecture tests.

## Layout

```
apps/myapp/              composition root and feature views
crates/myapp-domain/     types and rules
crates/myapp-core/       use cases, ports, Problem
crates/myapp-storage/    adapters (JSON file)
crates/myapp-ui/         tokens, theme, primitives
tests/architecture/      tests that keep the layers apart
```

Read [CLAUDE.md](CLAUDE.md) for the rules, [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for how the parts fit, [docs/CONTEXT.md](docs/CONTEXT.md) for the vocabulary and [docs/adr/](docs/adr) for the decisions.

## Troubleshooting

- The first build takes minutes: it compiles GPUI. Later builds reuse it.
- `cargo run` and `cargo test` build GPUI with different features, so switching between them recompiles it once.
- Windows Smart App Control can block freshly built build scripts or proc-macro DLLs (`os error 4551`). Find the blocked file in the error, rebuild that package once, and never turn the protection off.
- On Linux the window needs the Wayland or X11 libraries listed above at build time.
