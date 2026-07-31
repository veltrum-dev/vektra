#![cfg_attr(target_family = "wasm", no_main)]

mod demos;
mod fonts;

use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
use vektra::ThemeMode;

fn run_preview(preview_fonts: fonts::LoadedPreviewFonts) {
    let selection = demos::current_selection();
    let language = demos::current_language();
    let theme_mode = current_theme_mode();
    let font_family = preview_fonts.family();

    application()
        .with_assets(vektra::assets::Assets)
        .launch(move |cx: &mut App| {
            if let Err(error) = preview_fonts.register(cx) {
                fonts::show_font_error(&error);
                return;
            }

            fonts::set_font_ready();
            vektra::set_theme_mode(theme_mode, cx);
            demos::bind_keys(cx);

            let bounds = Bounds::centered(None, size(px(720.), px(420.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                {
                    let selection = selection.clone();
                    move |window, cx| {
                        cx.new(|cx| {
                            demos::PreviewApp::new(selection, language, font_family, window, cx)
                        })
                    }
                },
            )
            .expect("Vektra 文档预览窗口应能成功打开");
            cx.activate(true);
        });

    #[cfg(target_family = "wasm")]
    apply_pending_theme();
}

trait LaunchApplication {
    fn launch<F>(self, on_finish_launching: F)
    where
        F: 'static + FnOnce(&mut App);
}

impl LaunchApplication for gpui::Application {
    fn launch<F>(self, on_finish_launching: F)
    where
        F: 'static + FnOnce(&mut App),
    {
        #[cfg(target_family = "wasm")]
        {
            APP_HANDLE.with(|handle| {
                *handle.borrow_mut() = Some(self.run_embedded(on_finish_launching));
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            self.run(on_finish_launching);
        }
    }
}

#[cfg(target_family = "wasm")]
thread_local! {
    static APP_HANDLE: std::cell::RefCell<Option<gpui::ApplicationHandle>> =
        const { std::cell::RefCell::new(None) };
    static PENDING_THEME_MODE: std::cell::RefCell<Option<ThemeMode>> =
        const { std::cell::RefCell::new(None) };
    static THEME_MESSAGE_LISTENER: std::cell::RefCell<Option<wasm_bindgen::closure::Closure<dyn FnMut(web_sys::MessageEvent)>>> =
        const { std::cell::RefCell::new(None) };
    static APPLYING_THEME_MODE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn application() -> gpui::Application {
    #[cfg(target_family = "wasm")]
    {
        let platform = std::rc::Rc::new(gpui_web::WebPlatform::new(false));
        let http_client = std::sync::Arc::new(platform.fetch_http_client());
        gpui::Application::with_platform(platform).with_http_client(http_client)
    }

    #[cfg(not(target_family = "wasm"))]
    {
        gpui_platform::application()
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    let preview_fonts = fonts::load_preview_fonts()
        .unwrap_or_else(|error| panic!("Vektra 文档预览字体加载失败：{error}"));
    run_preview(preview_fonts);
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    init_preview_logging();
    init_theme_message_listener();
    fonts::set_font_loading();
    wasm_bindgen_futures::spawn_local(async {
        match fonts::load_preview_fonts().await {
            Ok(preview_fonts) => run_preview(preview_fonts),
            Err(error) => fonts::show_font_error(&error),
        }
    });
}

fn current_theme_mode() -> ThemeMode {
    #[cfg(target_family = "wasm")]
    {
        pending_theme_mode().unwrap_or_else(demos::current_theme_mode)
    }

    #[cfg(not(target_family = "wasm"))]
    {
        demos::current_theme_mode()
    }
}

#[cfg(target_family = "wasm")]
fn init_theme_message_listener() {
    use wasm_bindgen::JsCast as _;

    let Some(window) = web_sys::window() else {
        return;
    };

    let listener =
        wasm_bindgen::closure::Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let Some(theme_mode) = verified_theme_message(&event) else {
                return;
            };
            request_theme_mode(theme_mode);
        }) as Box<dyn FnMut(_)>);

    window
        .add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
        .ok();
    THEME_MESSAGE_LISTENER.with(|slot| {
        *slot.borrow_mut() = Some(listener);
    });
}

#[cfg(target_family = "wasm")]
fn verified_theme_message(event: &web_sys::MessageEvent) -> Option<ThemeMode> {
    let window = web_sys::window()?;
    let origin = window.location().origin().ok()?;
    if event.origin() != origin {
        return None;
    }

    let parent = window.parent().ok().flatten()?;
    let source = event.source()?;
    if !js_sys::Object::is(&source.into(), &parent.into()) {
        return None;
    }

    let data = event.data();
    let message_type = js_sys::Reflect::get(&data, &"type".into())
        .ok()
        .and_then(|value| value.as_string())?;
    if message_type != "vektra-preview:theme" {
        return None;
    }

    let value = js_sys::Reflect::get(&data, &"value".into())
        .ok()
        .and_then(|value| value.as_string())?;
    match value.as_str() {
        "light" => Some(ThemeMode::Light),
        "dark" => Some(ThemeMode::Dark),
        _ => None,
    }
}

#[cfg(target_family = "wasm")]
fn request_theme_mode(mode: ThemeMode) {
    PENDING_THEME_MODE.with(|pending| {
        *pending.borrow_mut() = Some(mode);
    });

    APP_HANDLE.with(|handle| {
        let borrowed_handle = handle.borrow();
        let Some(handle) = borrowed_handle.as_ref() else {
            return;
        };
        update_theme_mode(handle, mode);
    });
}

#[cfg(target_family = "wasm")]
fn update_theme_mode(handle: &gpui::ApplicationHandle, mode: ThemeMode) {
    APPLYING_THEME_MODE.with(|applying| {
        if applying.get() {
            return;
        }

        applying.set(true);
        handle.update(|cx| {
            vektra::set_theme_mode(mode, cx);
            set_theme_state(mode);
        });
        applying.set(false);
    });
}

#[cfg(target_family = "wasm")]
fn apply_pending_theme() {
    let Some(mode) = pending_theme_mode() else {
        return;
    };

    APP_HANDLE.with(|handle| {
        let borrowed_handle = handle.borrow();
        let Some(handle) = borrowed_handle.as_ref() else {
            return;
        };
        update_theme_mode(handle, mode);
    });
}

#[cfg(target_family = "wasm")]
fn pending_theme_mode() -> Option<ThemeMode> {
    PENDING_THEME_MODE.with(|pending| *pending.borrow())
}

#[cfg(target_family = "wasm")]
fn set_theme_state(mode: ThemeMode) {
    let Some(theme) = theme_mode_value(mode) else {
        return;
    };
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };

    body.set_attribute("data-vektra-preview-theme", theme).ok();

    let state = js_sys::Object::new();
    js_sys::Reflect::set(&state, &"theme".into(), &theme.into()).ok();
    for (attribute, property) in [
        ("data-vektra-preview-demo-id", "demoId"),
        ("data-vektra-preview-status", "status"),
        ("data-vektra-preview-clicks", "clicks"),
        ("data-vektra-preview-last-clicked", "lastClicked"),
        ("data-vektra-preview-font-status", "fontStatus"),
        ("data-vektra-preview-font-family", "fontFamily"),
    ] {
        if let Some(value) = body.get_attribute(attribute) {
            js_sys::Reflect::set(&state, &property.into(), &value.into()).ok();
        }
    }
    demos::set_window_preview_state(&state);
}

#[cfg(target_family = "wasm")]
fn theme_mode_value(mode: ThemeMode) -> Option<&'static str> {
    match mode {
        ThemeMode::Light => Some("light"),
        ThemeMode::Dark => Some("dark"),
        ThemeMode::System => None,
    }
}

#[cfg(target_family = "wasm")]
fn init_preview_logging() {
    struct PreviewLogger;

    impl log::Log for PreviewLogger {
        fn enabled(&self, _: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            if !self.enabled(record.metadata()) {
                return;
            }

            let body = record.args().to_string();
            if record.target() == "gpui::window"
                && (body == "deferring re-entrant window draw request"
                    || body == "RefCell already borrowed")
            {
                return;
            }

            let message = format!("[{}] {}: {}", record.level(), record.target(), body);
            let message = wasm_bindgen::JsValue::from_str(&message);
            match record.level() {
                log::Level::Error => web_sys::console::error_1(&message),
                log::Level::Warn => web_sys::console::warn_1(&message),
                log::Level::Info => web_sys::console::info_1(&message),
                log::Level::Debug | log::Level::Trace => web_sys::console::log_1(&message),
            }
        }

        fn flush(&self) {}
    }

    static LOGGER: PreviewLogger = PreviewLogger;
    log::set_logger(&LOGGER).ok();
    log::set_max_level(if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    });
}
