# Architecture

## Layers

| Crate | Depends on (workspace) | External crates |
|---|---|---|
| `myapp-domain` | none | `thiserror` |
| `myapp-core` | `myapp-domain` | `thiserror`, `tracing` |
| `myapp-storage` | `myapp-core` | `serde`, `serde_json` |
| `myapp-ui` | none | `gpui` |
| `myapp` (app) | `myapp-core`, `myapp-storage`, `myapp-ui` | any, declared in `[workspace.dependencies]` |
| `architecture` | none | `toml` |

The app reaches the domain through `myapp_core::domain`. The storage crate does the same.

## The dependency rule

Dependencies point inward: the domain knows nothing, the core knows the domain, adapters know the core, and only the app knows everything. GPUI exists only in `myapp-ui` and the app. Storage is wired only in `apps/myapp/src/main.rs`.

`tests/architecture` enforces it:

- `dependencies.rs`: `every_member_is_registered_and_uses_only_allowed_workspace_crates`, `inner_layers_use_only_allow_listed_external_crates`, `gpui_is_confined_to_the_app_and_the_ui_kit`, `every_member_inherits_the_workspace_lints`, `external_dependencies_come_from_the_workspace_table`.
- `sources.rs`: `gpui_is_not_referenced_outside_the_app_and_the_ui_kit`, `domain_and_core_do_no_io`, `storage_is_wired_only_in_main`, `color_literals_live_only_in_tokens`, `no_mod_rs_files`, `source_files_stay_under_600_lines`.

## How a command flows

1. A key press or click becomes a GPUI action on `ItemsView` (`AddItem`, `ToggleSelected`, `RemoveSelected`).
2. The handler calls `ItemsState::apply`, which calls `myapp_core::items::apply` on the list.
3. If the list changed and `request_save` says no save is running, `ItemsView::save` snapshots the list and runs `core::items::save` with `cx.background_spawn`.
4. The result returns through `this.update(cx, ..)` to `ItemsState::finish_saving`. If the list changed meanwhile, one more save runs with the latest list.
5. A failure becomes a `Problem`, shown as a banner with a retry button when `is_retryable`.

## Adding a feature

1. Domain types and rules in `myapp-domain`.
2. Use cases and, if there is I/O, a port in `myapp-core`, with a fake under `fake`.
3. The adapter in `myapp-storage` (or a new adapter crate, with an ADR), wired only in `main.rs`.
4. In the app: `feature.rs` and `feature/state.rs` with the actions, `bind_keys` and strings in `i18n.rs`.
5. Tests in every layer you touched.

To remove the `items` example, delete `apps/myapp/src/items.rs`, `apps/myapp/src/items/`, the item modules of the domain, core and storage crates, their strings in `i18n.rs`, and adjust `main.rs` and `shell.rs`.
