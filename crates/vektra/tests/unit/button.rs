use super::*;
use crate::{resolved_theme_mode, set_theme_mode};
use gpui::{
    AnyWindowHandle, AppContext, AtlasKey, AtlasTextureId, AtlasTextureKind, AtlasTile, ClickEvent,
    Context, DevicePixels, HeadlessAppContext, InputEvent, KeyUpEvent, Keystroke, Modifiers,
    MouseMoveEvent, NoopTextSystem, PlatformAtlas, PlatformHeadlessRenderer, Render, Scene, Size,
    TestAppContext, TileId, VisualTestContext, point, px, rgb, size,
};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
};
use vektra_theme::{ResolvedThemeMode, ThemeMode};

#[test]
fn defaults_resolve_to_primary_md_and_enabled() {
    let button = Button::new("save").label("Save");
    assert_eq!(button.resolved_variant(), ButtonVariant::Primary);
    assert_eq!(button.resolved_size(), ButtonSize::Md);
    assert!(!button.is_disabled());
}

#[test]
fn tooltip_and_aria_description_are_independent_and_last_tooltip_wins() {
    let button = Button::new("save")
        .label("保存")
        .tooltip("旧提示")
        .tooltip("保存当前修改")
        .aria_description("补充说明");

    assert_eq!(button.tooltip_text().unwrap().as_ref(), "保存当前修改");
    assert_eq!(button.aria_description_text().unwrap().as_ref(), "补充说明");
    assert_eq!(button.label_text().as_ref(), "保存");
}

#[test]
fn tooltip_configuration_is_preserved_by_button() {
    let button = Button::new("save").tooltip(
        Tooltip::new("保存")
            .open(true)
            .arrow(false)
            .color(rgb(0xffffff))
            .bg_color(rgb(0x222222))
            .animated(false),
    );
    let tooltip = button.tooltip_value().unwrap();

    assert_eq!(tooltip.text_value().as_ref(), "保存");
    assert_eq!(tooltip.open_value(), Some(true));
    assert!(!tooltip.arrow_value());
    assert!(tooltip.color_value().is_some());
    assert!(tooltip.bg_color_value().is_some());
    assert!(!tooltip.animated_value());
}

#[test]
fn tooltip_placement_defaults_to_bottom_and_last_call_wins() {
    assert_eq!(
        Button::new("default").tooltip_placement_value(),
        TooltipPlacement::Bottom
    );
    let button = Button::new("placed")
        .tooltip_placement(TooltipPlacement::TopStart)
        .tooltip_placement(TooltipPlacement::LeftEnd);
    assert_eq!(button.tooltip_placement_value(), TooltipPlacement::LeftEnd);
}

#[test]
fn explicit_variant_and_size_are_preserved() {
    let button = Button::new("save")
        .variant(ButtonVariant::Ghost)
        .size(ButtonSize::Lg);
    assert_eq!(button.resolved_variant(), ButtonVariant::Ghost);
    assert_eq!(button.resolved_size(), ButtonSize::Lg);
}

#[test]
fn link_is_the_only_variant_that_underlines_on_hover() {
    assert!(ButtonVariant::Link.underlines_on_hover());
    assert!(!ButtonVariant::Primary.underlines_on_hover());
    assert!(!ButtonVariant::Outline.underlines_on_hover());
    assert!(!ButtonVariant::Ghost.underlines_on_hover());
    assert!(!ButtonVariant::Destructive.underlines_on_hover());
    assert!(!ButtonVariant::Secondary.underlines_on_hover());
}

#[test]
fn width_last_call_wins() {
    let fixed_then_full = Button::new("a").width(px(200.)).full_width();
    assert!(matches!(fixed_then_full.width, Some(ButtonWidth::Full)));

    let full_then_fixed = Button::new("b").full_width().width(px(200.));
    assert!(matches!(full_then_fixed.width, Some(ButtonWidth::Fixed(_))));
}

#[test]
fn icon_slots_default_to_none() {
    let button = Button::new("a");
    assert_eq!(button.start_icon_source(), None);
    assert_eq!(button.end_icon_source(), None);
}

#[test]
fn icon_slots_can_be_set_independently_and_together() {
    let start = Button::new("a").start_icon(IconSource::asset("icons/settings.svg"));
    assert_eq!(
        start.start_icon_source().unwrap().path(),
        "icons/settings.svg"
    );
    assert_eq!(start.end_icon_source(), None);

    let end = Button::new("a").end_icon(IconSource::asset("icons/custom-end.svg"));
    assert_eq!(end.start_icon_source(), None);
    assert_eq!(
        end.end_icon_source().unwrap().path(),
        "icons/custom-end.svg"
    );

    let both = Button::new("a")
        .start_icon(IconSource::asset("icons/settings.svg"))
        .end_icon(IconSource::asset("icons/custom-end.svg"));
    assert_eq!(
        both.start_icon_source().unwrap().path(),
        "icons/settings.svg"
    );
    assert_eq!(
        both.end_icon_source().unwrap().path(),
        "icons/custom-end.svg"
    );
}

#[test]
fn later_icon_slot_call_wins() {
    let button = Button::new("a")
        .start_icon(IconSource::asset("icons/settings.svg"))
        .start_icon(IconSource::asset("icons/custom-start.svg"))
        .end_icon(IconSource::asset("icons/settings.svg"))
        .end_icon(IconSource::asset("icons/custom-end.svg"));

    assert_eq!(
        button.start_icon_source().unwrap().path(),
        "icons/custom-start.svg"
    );
    assert_eq!(
        button.end_icon_source().unwrap().path(),
        "icons/custom-end.svg"
    );
}

#[test]
fn auto_insert_space_rules() {
    assert_eq!(Button::new("a").label("保存").display_label(), "保 存");
    assert_eq!(
        Button::new("a")
            .label("保存")
            .auto_insert_space(true)
            .display_label(),
        "保 存"
    );
    assert_eq!(
        Button::new("a")
            .label("保存")
            .auto_insert_space(false)
            .display_label(),
        "保存"
    );
    assert_eq!(Button::new("a").label("保 存").display_label(), "保 存");
    assert_eq!(Button::new("a").label("保").display_label(), "保");
    assert_eq!(Button::new("a").label("保存中").display_label(), "保存中");
    assert_eq!(Button::new("a").label("Save").display_label(), "Save");
    assert_eq!(Button::new("a").label("保存1").display_label(), "保存1");
    assert_eq!(Button::new("a").label("保存!").display_label(), "保存!");
    assert_eq!(Button::new("a").label(" 保存").display_label(), " 保存");
    assert_eq!(Button::new("a").label("𠀀𠀁").display_label(), "𠀀 𠀁");
}

#[test]
fn auto_spacing_does_not_change_accessible_label_source() {
    let button = Button::new("a")
        .label("保存")
        .start_icon(IconSource::asset("icons/settings.svg"))
        .end_icon(IconSource::asset("icons/settings.svg"));
    assert_eq!(button.label_text(), "保存");
    assert_eq!(button.display_label(), "保 存");
}

#[test]
fn disabled_state_is_stored_directly() {
    assert!(Button::new("a").disabled(true).is_disabled());
}

#[test]
fn activity_builders_are_mutually_exclusive_and_last_call_wins() {
    assert_eq!(Button::new("idle").activity(), ButtonActivity::Idle);
    assert_eq!(
        Button::new("loading").loading(true).activity(),
        ButtonActivity::Loading
    );
    assert_eq!(
        Button::new("loading-off")
            .loading(true)
            .loading(false)
            .activity(),
        ButtonActivity::Idle
    );
    assert_eq!(
        Button::new("progress-last")
            .loading(true)
            .progress(0.4)
            .activity(),
        ButtonActivity::Progress(0.4)
    );
    assert_eq!(
        Button::new("loading-last")
            .progress(0.4)
            .loading(true)
            .activity(),
        ButtonActivity::Loading
    );
}

#[test]
fn progress_values_are_normalized_before_layout() {
    for (input, expected) in [
        (0., 0.),
        (0.5, 0.5),
        (1., 1.),
        (-0.5, 0.),
        (1.5, 1.),
        (f32::NEG_INFINITY, 0.),
        (f32::INFINITY, 1.),
        (f32::NAN, 0.),
    ] {
        let ButtonActivity::Progress(actual) = Button::new("progress").progress(input).activity()
        else {
            panic!("progress builder 应进入确定进度状态");
        };
        assert_eq!(actual, expected, "输入 {input:?} 的归一化结果不正确");
        assert!(actual.is_finite());
    }
}

#[test]
fn selected_preserves_unconfigured_false_and_true_states() {
    assert_eq!(Button::new("plain").selected_state(), None);
    assert_eq!(
        Button::new("off").selected(false).selected_state(),
        Some(false)
    );
    assert_eq!(
        Button::new("on").selected(true).selected_state(),
        Some(true)
    );
}

#[test]
fn accessibility_toggle_state_is_only_present_when_selected_is_configured() {
    assert_eq!(toggled_state(None), None);
    assert_eq!(toggled_state(Some(false)), Some(Toggled::False));
    assert_eq!(toggled_state(Some(true)), Some(Toggled::True));
}

#[test]
fn determinate_progress_accessibility_value_uses_percentage_range() {
    assert_eq!(progress_percent(0.), 0.);
    assert_eq!(progress_percent(0.42), 42.);
    assert_eq!(progress_percent(1.), 100.);
}

#[test]
fn activity_id_is_stably_derived_from_button_id() {
    let button = Button::new("save").loading(true);
    assert_eq!(
        button.activity_id(),
        ElementId::from((ElementId::from("save"), "activity"))
    );
}

#[test]
fn callback_can_be_reused_for_keyboard_event_shape() {
    let count = Rc::new(Cell::new(0));
    let seen_keyboard = Rc::new(Cell::new(false));
    let handler = {
        let count = count.clone();
        let seen_keyboard = seen_keyboard.clone();
        Rc::new(
            move |event: &ClickEvent, _window: &mut Window, _cx: &mut App| {
                count.set(count.get() + 1);
                seen_keyboard.set(event.is_keyboard());
            },
        )
    };

    let event = keyboard_click(KeyboardButton::Enter);
    assert!(event.is_keyboard());
    let _button = Button::new("a").on_click(move |event, window, cx| handler(event, window, cx));
    assert_eq!(count.get(), 0);
}

struct TestView {
    count: usize,
    disabled: bool,
}

struct LinkUnderlineView;

#[derive(Debug, Clone, Copy)]
enum TestActivity {
    Idle,
    Loading,
    Progress,
}

struct ActivityTestView {
    activity: TestActivity,
    disabled: bool,
    selected: Option<bool>,
    business_count: usize,
    parent_click_count: usize,
    parent_key_count: usize,
}

struct StateMatrixView;

impl Render for TestView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().id("test-root").size(px(180.)).child(
            Button::new("target")
                .label("Hit")
                .width(px(120.))
                .disabled(self.disabled)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.count += 1;
                    cx.notify();
                })),
        )
    }
}

impl Render for LinkUnderlineView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("link-underline-root")
            .size(px(320.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                Button::new("link-fixed")
                    .label("链接")
                    .variant(ButtonVariant::Link)
                    .width(px(120.)),
            )
            .child(
                Button::new("link-full")
                    .label("链接")
                    .variant(ButtonVariant::Link)
                    .full_width(),
            )
            .child(
                Button::new("link-icons")
                    .label("链接图标")
                    .variant(ButtonVariant::Link)
                    .start_icon(IconSource::asset("icons/settings.svg"))
                    .end_icon(IconSource::asset("icons/settings.svg")),
            )
            .child(
                Button::new("link-disabled")
                    .label("禁用")
                    .variant(ButtonVariant::Link)
                    .disabled(true),
            )
            .child(
                Button::new("primary")
                    .label("主要按钮")
                    .variant(ButtonVariant::Primary),
            )
    }
}

impl Render for ActivityTestView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let button = Button::new("activity-target")
            .label("提交")
            .width(px(120.))
            .disabled(self.disabled)
            .on_click_in(cx, |this, _, _, cx| {
                this.business_count += 1;
                cx.notify();
            });
        let button = match self.activity {
            TestActivity::Idle => button,
            TestActivity::Loading => button.loading(true),
            TestActivity::Progress => button.progress(0.42),
        };
        let button = match self.selected {
            Some(selected) => button.selected(selected),
            None => button,
        };

        div()
            .id("activity-root")
            .size(px(180.))
            .on_click(cx.listener(|this, _, _, _| {
                this.parent_click_count += 1;
            }))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, _| {
                if is_plain_key(event, "enter") || is_plain_key(event, "space") {
                    this.parent_key_count += 1;
                }
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _, _| {
                if is_plain_key_up(event, "enter") || is_plain_key_up(event, "space") {
                    this.parent_key_count += 1;
                }
            }))
            .child(button)
    }
}

impl Render for StateMatrixView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut buttons = Vec::new();
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::Destructive,
            ButtonVariant::Secondary,
            ButtonVariant::Link,
        ] {
            for size in [
                ButtonSize::Xs,
                ButtonSize::Sm,
                ButtonSize::Md,
                ButtonSize::Lg,
            ] {
                let id = format!("{variant:?}-{size:?}");
                buttons.push(
                    Button::new(format!("{id}-selected"))
                        .label("较长 selected 文本")
                        .variant(variant)
                        .size(size)
                        .start_icon(IconSource::asset("icons/settings.svg"))
                        .end_icon(IconSource::asset("icons/settings.svg"))
                        .selected(true)
                        .width(px(180.)),
                );
                buttons.push(
                    Button::new(format!("{id}-loading"))
                        .label("Loading")
                        .variant(variant)
                        .size(size)
                        .start_icon(IconSource::asset("icons/settings.svg"))
                        .end_icon(IconSource::asset("icons/settings.svg"))
                        .loading(true),
                );
                buttons.push(
                    Button::new(format!("{id}-progress"))
                        .label("Progress")
                        .variant(variant)
                        .size(size)
                        .start_icon(IconSource::asset("icons/settings.svg"))
                        .end_icon(IconSource::asset("icons/settings.svg"))
                        .progress(0.58),
                );
                buttons.push(
                    Button::new(format!("{id}-combined"))
                        .label("Combined")
                        .variant(variant)
                        .size(size)
                        .selected(true)
                        .progress(0.82)
                        .disabled(true)
                        .full_width(),
                );
            }
        }

        div()
            .id("state-matrix")
            .size_full()
            .flex()
            .flex_wrap()
            .gap(px(4.))
            .children(buttons)
    }
}

#[derive(Clone, Debug, Default)]
struct UnderlineRecorder(Rc<RefCell<Vec<UnderlineSnapshot>>>);

#[derive(Clone, Debug)]
struct UnderlineSnapshot {
    count: usize,
    thicknesses: Vec<f32>,
}

impl UnderlineRecorder {
    fn push(&self, scene: &Scene) {
        self.0.borrow_mut().push(UnderlineSnapshot {
            count: scene.underlines.len(),
            thicknesses: scene
                .underlines
                .iter()
                .map(|underline| underline.thickness.0)
                .collect(),
        });
    }

    fn latest(&self) -> UnderlineSnapshot {
        self.0
            .borrow()
            .last()
            .cloned()
            .expect("headless renderer 应至少记录一帧")
    }
}

struct RecordingRenderer {
    recorder: UnderlineRecorder,
    atlas: Arc<RecordingAtlas>,
}

impl RecordingRenderer {
    fn new(recorder: UnderlineRecorder) -> Self {
        Self {
            recorder,
            atlas: Arc::new(RecordingAtlas::default()),
        }
    }
}

impl PlatformHeadlessRenderer for RecordingRenderer {
    fn render_scene_to_image(
        &mut self,
        scene: &Scene,
        size: Size<DevicePixels>,
    ) -> gpui::Result<image::RgbaImage> {
        self.recorder.push(scene);
        let width = u32::try_from(size.width.0).unwrap_or(1).max(1);
        let height = u32::try_from(size.height.0).unwrap_or(1).max(1);
        Ok(image::RgbaImage::new(width, height))
    }

    fn render_scene(&mut self, scene: &Scene, _: Size<DevicePixels>) -> gpui::Result<()> {
        self.recorder.push(scene);
        Ok(())
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.atlas.clone()
    }
}

#[derive(Default)]
struct RecordingAtlas {
    state: Mutex<RecordingAtlasState>,
}

#[derive(Default)]
struct RecordingAtlasState {
    next_id: u32,
    tiles: HashMap<AtlasKey, AtlasTile>,
}

impl PlatformAtlas for RecordingAtlas {
    fn get_or_insert_with<'a>(
        &self,
        key: &AtlasKey,
        build: &mut dyn FnMut() -> gpui::Result<Option<(Size<DevicePixels>, Cow<'a, [u8]>)>>,
    ) -> gpui::Result<Option<AtlasTile>> {
        let state = self.state.lock().unwrap();
        if let Some(&tile) = state.tiles.get(key) {
            return Ok(Some(tile));
        }
        drop(state);

        let Some((size, _)) = build()? else {
            return Ok(None);
        };

        let mut state = self.state.lock().unwrap();
        state.next_id += 1;
        let texture_id = state.next_id;
        state.next_id += 1;
        let tile_id = state.next_id;

        let tile = AtlasTile {
            texture_id: AtlasTextureId {
                index: texture_id,
                kind: AtlasTextureKind::Monochrome,
            },
            tile_id: TileId(tile_id),
            padding: 0,
            bounds: gpui::Bounds {
                origin: Default::default(),
                size,
            },
        };
        state.tiles.insert(key.clone(), tile);
        Ok(Some(tile))
    }

    fn remove(&self, key: &AtlasKey) {
        self.state.lock().unwrap().tiles.remove(key);
    }
}

fn test_view(
    cx: &mut TestAppContext,
    disabled: bool,
) -> (gpui::Entity<TestView>, &mut VisualTestContext) {
    cx.add_window_view(|_, _| TestView { count: 0, disabled })
}

fn activity_view(
    cx: &mut TestAppContext,
    activity: TestActivity,
    disabled: bool,
    selected: Option<bool>,
) -> (gpui::Entity<ActivityTestView>, &mut VisualTestContext) {
    cx.add_window_view(|_, _| ActivityTestView {
        activity,
        disabled,
        selected,
        business_count: 0,
        parent_click_count: 0,
        parent_key_count: 0,
    })
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn headless_link_view() -> (HeadlessAppContext, AnyWindowHandle, UnderlineRecorder) {
    let recorder = UnderlineRecorder::default();
    let renderer_recorder = recorder.clone();
    let mut cx = HeadlessAppContext::with_platform(
        Arc::new(NoopTextSystem::new()),
        Arc::new(()),
        move || Some(Box::new(RecordingRenderer::new(renderer_recorder.clone()))),
    );
    let window = cx
        .open_window(size(px(320.), px(220.)), |_, cx| {
            cx.new(|_| LinkUnderlineView)
        })
        .expect("headless Button 测试窗口应能成功打开")
        .into();

    (cx, window, recorder)
}

fn render_underlines(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    recorder: &UnderlineRecorder,
) -> UnderlineSnapshot {
    cx.update_window(window, |_, window, cx| {
        window.draw(cx).clear(cx);
    })
    .expect("headless Button 测试窗口应可重绘");
    let _image = cx
        .capture_screenshot(window)
        .expect("测试 renderer 应能捕获当前 frame");
    recorder.latest()
}

fn dispatch_mouse_move(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    position: gpui::Point<gpui::Pixels>,
) {
    cx.update_window(window, |_, window, cx| {
        window.dispatch_event(
            MouseMoveEvent {
                position,
                modifiers: Modifiers::none(),
                pressed_button: None,
            }
            .to_platform_input(),
            cx,
        );
    })
    .expect("headless Button 测试窗口应能派发鼠标事件");
    cx.run_until_parked();
}

#[gpui::test]
fn mouse_activation_runs_once(cx: &mut TestAppContext) {
    let (view, cx) = test_view(cx, false);
    draw(cx);
    cx.simulate_click(point(px(24.), px(18.)), Modifiers::none());

    assert_eq!(view.read_with(cx, |view, _| view.count), 1);
}

#[gpui::test]
fn disabled_button_does_not_activate(cx: &mut TestAppContext) {
    let (view, cx) = test_view(cx, true);
    draw(cx);
    cx.simulate_click(point(px(24.), px(18.)), Modifiers::none());
    cx.simulate_keystrokes("enter space");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.count), 0);
}

#[gpui::test]
fn enter_and_space_activate_focused_button_once(cx: &mut TestAppContext) {
    let (view, cx) = test_view(cx, false);
    draw(cx);
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.count), 2);
}

#[gpui::test]
fn selected_button_still_activates_with_mouse_enter_and_space(cx: &mut TestAppContext) {
    let (view, cx) = activity_view(cx, TestActivity::Idle, false, Some(true));
    draw(cx);
    cx.simulate_click(point(px(24.), px(18.)), Modifiers::none());
    cx.update(|window, cx| window.focus_next(cx));
    cx.simulate_keystrokes("enter");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.business_count), 3);
    assert_eq!(view.read_with(cx, |view, _| view.parent_click_count), 0);
    assert_eq!(view.read_with(cx, |view, _| view.parent_key_count), 0);
}

#[gpui::test]
fn loading_and_progress_consume_mouse_enter_and_space_without_activation(cx: &mut TestAppContext) {
    for activity in [TestActivity::Loading, TestActivity::Progress] {
        let (view, cx) = activity_view(cx, activity, false, None);
        draw(cx);
        cx.simulate_click(point(px(24.), px(18.)), Modifiers::none());
        cx.update(|window, cx| window.focus_next(cx));
        cx.simulate_keystrokes("enter space");
        cx.simulate_event(KeyUpEvent {
            keystroke: Keystroke::parse("space").unwrap(),
        });

        assert_eq!(view.read_with(cx, |view, _| view.business_count), 0);
        assert_eq!(view.read_with(cx, |view, _| view.parent_click_count), 0);
        assert_eq!(view.read_with(cx, |view, _| view.parent_key_count), 0);
    }
}

#[gpui::test]
fn disabled_busy_selected_button_remains_inactive(cx: &mut TestAppContext) {
    let (view, cx) = activity_view(cx, TestActivity::Progress, true, Some(true));
    draw(cx);
    cx.simulate_click(point(px(24.), px(18.)), Modifiers::none());
    cx.simulate_keystrokes("enter space");
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse("space").unwrap(),
    });

    assert_eq!(view.read_with(cx, |view, _| view.business_count), 0);
    assert_eq!(view.read_with(cx, |view, _| view.parent_click_count), 0);
    assert_eq!(view.read_with(cx, |view, _| view.parent_key_count), 0);
}

#[gpui::test]
fn loading_renders_with_reduce_motion_enabled(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_reduce_motion(true));
    let (_view, cx) = activity_view(cx, TestActivity::Loading, false, None);
    draw(cx);
}

#[gpui::test]
fn theme_mode_switch_resolves_new_theme(cx: &mut TestAppContext) {
    let (_view, cx) = test_view(cx, false);
    cx.update(|window, cx| {
        set_theme_mode(ThemeMode::Dark, cx);
        assert_eq!(resolved_theme_mode(window, cx), ResolvedThemeMode::Dark);
        set_theme_mode(ThemeMode::Light, cx);
        assert_eq!(resolved_theme_mode(window, cx), ResolvedThemeMode::Light);
    });
}

#[gpui::test]
fn renders_in_light_and_dark(cx: &mut TestAppContext) {
    let (_view, cx) = test_view(cx, false);
    cx.update(|_, cx| set_theme_mode(ThemeMode::Light, cx));
    draw(cx);
    cx.update(|_, cx| set_theme_mode(ThemeMode::Dark, cx));
    draw(cx);
}

#[gpui::test]
fn all_variants_sizes_and_new_states_render_in_light_dark_and_system(cx: &mut TestAppContext) {
    let (_view, cx) = cx.add_window_view(|_, _| StateMatrixView);
    for mode in [ThemeMode::Light, ThemeMode::Dark, ThemeMode::System] {
        cx.update(|_, cx| set_theme_mode(mode, cx));
        draw(cx);
    }
}

#[gpui::test]
fn link_hover_underline_is_drawn_only_for_enabled_link_text() {
    let (mut cx, window, recorder) = headless_link_view();

    let normal = render_underlines(&mut cx, window, &recorder);
    assert_eq!(normal.count, 0, "Link normal 状态不应绘制下划线");

    dispatch_mouse_move(&mut cx, window, point(px(24.), px(18.)));
    let fixed_hover = render_underlines(&mut cx, window, &recorder);
    assert_eq!(fixed_hover.count, 1, "fixed width Link hover 应绘制下划线");
    assert!(
        fixed_hover
            .thicknesses
            .iter()
            .all(|thickness| *thickness > 0.),
        "Link hover 应绘制非零厚度下划线",
    );

    dispatch_mouse_move(&mut cx, window, point(px(310.), px(210.)));
    let after_leave = render_underlines(&mut cx, window, &recorder);
    assert_eq!(after_leave.count, 0, "鼠标离开 Link 后下划线应消失");

    dispatch_mouse_move(&mut cx, window, point(px(24.), px(50.)));
    let full_width_hover = render_underlines(&mut cx, window, &recorder);
    assert_eq!(
        full_width_hover.count, 1,
        "full_width Link hover 应绘制下划线"
    );

    dispatch_mouse_move(&mut cx, window, point(px(24.), px(90.)));
    let icon_link_hover = render_underlines(&mut cx, window, &recorder);
    assert_eq!(
        icon_link_hover.count, 1,
        "带 start/end icon 的 Link hover 应只为文字绘制一条下划线",
    );

    dispatch_mouse_move(&mut cx, window, point(px(24.), px(130.)));
    let disabled_hover = render_underlines(&mut cx, window, &recorder);
    assert_eq!(
        disabled_hover.count, 0,
        "disabled Link hover 不应绘制下划线"
    );

    dispatch_mouse_move(&mut cx, window, point(px(24.), px(170.)));
    let primary_hover = render_underlines(&mut cx, window, &recorder);
    assert_eq!(
        primary_hover.count, 0,
        "非 Link variant hover 不应绘制下划线"
    );
}
