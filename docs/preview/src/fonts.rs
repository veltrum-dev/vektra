use gpui::{App, FontWeight, font, px};
use std::borrow::Cow;

pub(crate) const FONT_FAMILY: &str = "Noto Sans SC";
pub(crate) const FONT_URL: &str = "assets/fonts/noto-sans-sc/NotoSansSC-VF.ttf";

pub(crate) struct LoadedPreviewFonts {
    bytes: Vec<u8>,
}

impl LoadedPreviewFonts {
    pub(crate) fn family(&self) -> &'static str {
        FONT_FAMILY
    }

    pub(crate) fn register(self, cx: &mut App) -> Result<(), String> {
        cx.text_system()
            .add_fonts(vec![Cow::Owned(self.bytes)])
            .map_err(|error| format!("字体注册到 GPUI text system 失败：{error:#}"))?;
        validate_registered_font(cx)
    }
}

fn validate_registered_font(cx: &mut App) -> Result<(), String> {
    let font_names = cx.text_system().all_font_names();
    if !font_names.iter().any(|name| name == FONT_FAMILY) {
        return Err(format!(
            "字体注册到 GPUI text system 失败：未找到 family `{FONT_FAMILY}`，实际 family 列表包含：{font_names:?}"
        ));
    }

    let regular = font(FONT_FAMILY);
    let regular_id = cx.text_system().resolve_font(&regular);

    let mut semibold = font(FONT_FAMILY);
    semibold.weight = FontWeight::SEMIBOLD;
    let semibold_id = cx.text_system().resolve_font(&semibold);

    for ch in [
        '中', '文', '按', '钮', '点', '击', '次', '数', '增', '加', '禁', '用', '预', '览',
    ] {
        cx.text_system()
            .typographic_bounds(regular_id, px(16.), ch)
            .map_err(|error| format!("Regular 中文字形验证失败：字符 `{ch}`，{error:#}"))?;
        cx.text_system()
            .typographic_bounds(semibold_id, px(16.), ch)
            .map_err(|error| format!("SemiBold 中文字形验证失败：字符 `{ch}`，{error:#}"))?;
    }

    Ok(())
}

#[cfg(target_family = "wasm")]
pub(crate) async fn load_preview_fonts() -> Result<LoadedPreviewFonts, String> {
    use wasm_bindgen::JsCast as _;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window().ok_or_else(|| "无法获取浏览器 window".to_owned())?;
    let response_value = JsFuture::from(window.fetch_with_str(FONT_URL))
        .await
        .map_err(|error| format!("字体资源加载失败：{FONT_URL}，Fetch 错误：{error:?}"))?;
    let response: web_sys::Response = response_value
        .dyn_into()
        .map_err(|_| format!("字体资源加载失败：{FONT_URL}，响应类型不是 Response"))?;

    if !response.ok() {
        return Err(format!(
            "字体资源加载失败：{}，HTTP {} {}",
            FONT_URL,
            response.status(),
            response.status_text()
        ));
    }

    let array_buffer = JsFuture::from(
        response
            .array_buffer()
            .map_err(|error| format!("字体资源加载失败：{FONT_URL}，读取字节失败：{error:?}"))?,
    )
    .await
    .map_err(|error| format!("字体资源加载失败：{FONT_URL}，读取字节失败：{error:?}"))?;
    let bytes = js_sys::Uint8Array::new(&array_buffer).to_vec();

    if bytes.is_empty() {
        return Err(format!("字体资源加载失败：{FONT_URL}，文件为空"));
    }

    Ok(LoadedPreviewFonts { bytes })
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn load_preview_fonts() -> Result<LoadedPreviewFonts, String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FONT_URL);
    let bytes = std::fs::read(&path)
        .map_err(|error| format!("字体资源加载失败：{}，{error}", path.display()))?;
    if bytes.is_empty() {
        return Err(format!("字体资源加载失败：{}，文件为空", path.display()));
    }
    Ok(LoadedPreviewFonts { bytes })
}

#[cfg(target_family = "wasm")]
pub(crate) fn set_font_loading() {
    set_font_state("loading");
}

#[cfg(target_family = "wasm")]
pub(crate) fn set_font_ready() {
    set_font_state("ready");
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn set_font_ready() {}

#[cfg(target_family = "wasm")]
pub(crate) fn show_font_error(error: &str) {
    set_font_state("error");
    web_sys::console::error_1(&format!("字体资源加载失败：{FONT_URL}；{error}").into());

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(error_element) = document.get_element_by_id("error") else {
        return;
    };

    error_element.set_text_content(Some(&format!(
        "字体资源加载失败：无法加载 Vektra GPUI 组件预览所需的中文字体。\n资源：{FONT_URL}\n原因：{error}"
    )));
    error_element.set_attribute("style", "display: grid").ok();
}

#[cfg(not(target_family = "wasm"))]
pub(crate) fn show_font_error(_: &str) {}

#[cfg(target_family = "wasm")]
fn set_font_state(status: &str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(body) = document.body() else {
        return;
    };

    body.set_attribute("data-vektra-preview-font-status", status)
        .ok();
    body.set_attribute("data-vektra-preview-font-family", FONT_FAMILY)
        .ok();

    let state = js_sys::Object::new();
    js_sys::Reflect::set(&state, &"fontStatus".into(), &status.into()).ok();
    js_sys::Reflect::set(&state, &"fontFamily".into(), &FONT_FAMILY.into()).ok();
    crate::demos::set_window_preview_state(&state);
}
