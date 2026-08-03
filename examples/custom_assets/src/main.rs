#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{
    App, AppContext, AssetSource, Bounds, Context, FocusHandle, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, Render, SharedString, Styled, Window, WindowBounds, WindowOptions,
    actions, div, px, size,
};
use gpui_platform::application;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use vektra::{Button, Icon, IconButton, IconName, ThemeMode, current_theme, set_theme_mode};

actions!(vektra_custom_assets_example, [Tab, TabPrev]);

#[derive(RustEmbed)]
#[folder = "custom_assets/assets"]
#[include = "icons/**/*.svg"]
struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        Ok(Self::get(path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter(|asset_path| asset_path.starts_with(path))
            .map(Into::into)
            .collect())
    }
}

#[derive(Debug, Clone, Copy, vektra::IntoIconSource)]
enum AppIconName {
    /// 默认映射到 `icons/logo.svg`。
    Logo,

    /// 默认复合词映射到 `icons/favorite_filled.svg`。
    FavoriteFilled,

    /// enum 名和文件名不一致时，显式映射到 `icons/heart.svg`。
    #[icon(path = "icons/heart.svg")]
    Favorite,
}

struct CustomAssetsExample {
    clicks: usize,
    focus_handle: FocusHandle,
}

impl CustomAssetsExample {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);
        Self {
            clicks: 0,
            focus_handle,
        }
    }

    fn on_tab(&mut self, _: &Tab, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_next(cx);
    }

    fn on_tab_prev(&mut self, _: &TabPrev, window: &mut Window, cx: &mut Context<Self>) {
        window.focus_prev(cx);
    }
}

impl Render for CustomAssetsExample {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = current_theme(window, cx);
        div()
            .id("vektra-custom-assets-example")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::on_tab))
            .on_action(cx.listener(Self::on_tab_prev))
            .size_full()
            .bg(theme.semantic.background)
            .text_color(theme.semantic.foreground)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.))
                    .p(px(20.))
                    .max_w(px(560.))
                    .child(div().text_size(px(24.)).child("自定义资源"))
                    .child(format!(
                        "自定义图标来自 examples/custom_assets/assets，Settings 来自 Vektra 回退资源。点击次数：{}",
                        self.clicks
                    ))
                    .child(
                        div()
                            .flex()
                            .gap(px(14.))
                            .items_center()
                            .child(Icon::new(AppIconName::Logo))
                            .child(Icon::new(AppIconName::FavoriteFilled))
                            .child(Icon::new(AppIconName::Favorite))
                            .child(Icon::new(IconName::Settings)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .items_center()
                            .child(
                                Button::new("favorite-button")
                                    .label("收藏")
                                    .start_icon(AppIconName::Favorite)
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    }),
                            )
                            .child(
                                IconButton::new("logo-action", AppIconName::Logo)
                                    .aria_label("Logo")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        this.clicks += 1;
                                        cx.notify();
                                    }),
                            )
                            .child(
                                IconButton::new("settings-action", IconName::Settings)
                                    .aria_label("设置")
                                    .on_click_in(cx, |this, _, _, cx| {
                                        set_theme_mode(ThemeMode::Dark, cx);
                                        this.clicks += 1;
                                        cx.notify();
                                    }),
                            ),
                    ),
            )
    }
}

fn run_example() {
    let assets = vektra::assets::Assets::with_overrides(AppAssets);
    application().with_assets(assets).run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("tab", Tab, None),
            KeyBinding::new("shift-tab", TabPrev, None),
        ]);
        let bounds = Bounds::centered(None, size(px(560.), px(360.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| cx.new(|cx| CustomAssetsExample::new(window, cx)),
        )
        .expect("Custom Assets 示例窗口应能成功打开");
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
