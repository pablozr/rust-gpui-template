//! MyApp: the desktop app and composition root.

// A release build is a GUI app on Windows: no console window next to it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod items;
mod shell;

use std::path::PathBuf;
use std::sync::Arc;

use gpui::{App, AppContext as _, Bounds, TitlebarOptions, WindowBounds, WindowOptions};
use gpui_platform::application;
use myapp_core::ItemStore;
use myapp_storage::JsonFileStore;
use myapp_ui::tokens::window;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "warn,myapp=info,myapp_core=info,myapp_storage=info".into()),
        )
        .init();

    let language = std::env::var("MYAPP_LANG")
        .ok()
        .and_then(|tag| i18n::Language::from_tag(&tag));
    if let Some(language) = language {
        i18n::set(language);
    }

    let items_file = app_dir().join("items.json");
    let store: Arc<dyn ItemStore> = Arc::new(JsonFileStore::new(items_file));

    application()
        .with_assets(myapp_ui::Assets)
        .run(move |cx: &mut App| {
            shell::bind_keys(cx);
            items::bind_keys(cx);

            let bounds = Bounds::centered(None, window::DEFAULT_SIZE, cx);
            let opened = cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some(i18n::app::title().into()),
                        ..Default::default()
                    }),
                    app_id: Some("dev.myapp.app".into()),
                    window_min_size: Some(window::MIN_SIZE),
                    ..Default::default()
                },
                |window, cx| cx.new(|cx| shell::Shell::new(store, window, cx)),
            );
            if let Err(error) = opened {
                tracing::error!(%error, "failed to open the main window");
                cx.quit();
                return;
            }
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            cx.activate(true);
        });
}

/// `<OS data dir>/myapp`, or the temp dir when the OS has none.
fn app_dir() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| {
        let fallback = std::env::temp_dir();
        tracing::warn!(?fallback, "no OS data directory; using the temp directory");
        fallback
    });
    base.join("myapp")
}
