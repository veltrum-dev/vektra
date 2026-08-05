use super::*;
use gpui::{AppContext as _, Modifiers, TestAppContext};

struct CaretTestView {
    state: Entity<InputState>,
    read_only: bool,
    disabled: bool,
}

struct AttachedLayoutTestView {
    state: Entity<InputState>,
    width: Pixels,
}

struct ConstrainedAttachedLayoutTestView {
    state: Entity<InputState>,
}

impl AttachedLayoutTestView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: cx.new(|cx| InputState::new("可清除的长文本", cx)),
            width: px(240.),
        }
    }
}

impl ConstrainedAttachedLayoutTestView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: cx.new(|cx| InputState::new("不能压扁", cx)),
        }
    }
}

impl Render for AttachedLayoutTestView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(self.width).child(
            Input::new("attached-layout-input", self.state.clone())
                .size(ComponentSize::Md)
                .clearable(InputClear::new("清空"))
                .suffix(
                    div()
                        .debug_selector(|| "unit-inline-suffix".into())
                        .w(px(20.))
                        .h(px(16.)),
                )
                .attached_suffix(
                    div()
                        .debug_selector(|| "unit-attached-content".into())
                        .w(px(64.))
                        .h_full(),
                ),
        )
    }
}

impl Render for ConstrainedAttachedLayoutTestView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().flex().flex_col().w(px(240.)).h(px(1.)).child(
            Input::new("constrained-attached-layout-input", self.state.clone())
                .size(ComponentSize::Md)
                .attached_suffix(div().w(px(64.)).h_full()),
        )
    }
}

impl CaretTestView {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            state: cx.new(|cx| InputState::new("abc", cx)),
            read_only: false,
            disabled: false,
        }
    }
}

impl Render for CaretTestView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Input::new("caret-test-input", self.state.clone())
            .read_only(self.read_only)
            .disabled(self.disabled)
    }
}

#[test]
fn single_line_normalization_preserves_content_without_trimming() {
    assert_eq!(normalize_single_line("  a\r\nb\nc\rd  "), "  a b c d  ");
}

#[test]
fn utf8_and_utf16_offsets_are_safe_for_cjk_and_emoji() {
    let text = "A中👨‍👩‍👧‍👦e\u{301}";
    for byte in text
        .char_indices()
        .map(|(byte, _)| byte)
        .chain(std::iter::once(text.len()))
    {
        let utf16 = utf8_to_utf16(text, byte);
        assert_eq!(utf16_to_utf8(text, utf16), byte);
    }

    let family_start = "A中".len();
    let inside_surrogate = utf8_to_utf16(text, family_start) + 1;
    assert_eq!(utf16_to_utf8(text, inside_surrogate), family_start);
}

#[test]
fn grapheme_boundaries_keep_zwj_and_combining_sequences_intact() {
    let text = "a👨‍👩‍👧‍👦e\u{301}中";
    let family_start = 1;
    let family_end = family_start + "👨‍👩‍👧‍👦".len();
    let combining_end = family_end + "e\u{301}".len();

    assert_eq!(next_grapheme_boundary(text, family_start), family_end);
    assert_eq!(previous_grapheme_boundary(text, family_end), family_start);
    assert_eq!(next_grapheme_boundary(text, family_end), combining_end);
    assert_eq!(previous_grapheme_boundary(text, combining_end), family_end);
    assert_eq!(
        nearest_grapheme_boundary(text, family_start + 4),
        family_start
    );
}

#[test]
fn selection_ranges_are_clamped_ordered_and_grapheme_aligned() {
    let text = "a👩🏽‍💻b";
    let emoji_end = 1 + "👩🏽‍💻".len();
    let reversed = Range {
        start: usize::MAX,
        end: 2,
    };
    assert_eq!(normalize_selection(text, reversed), 1..text.len());
    assert_eq!(normalize_selection(text, emoji_end..1), 1..emoji_end);
}

#[test]
fn word_ranges_handle_ascii_cjk_and_punctuation() {
    let text = "hello 世界!";
    assert_eq!(&text[word_range_at(text, 2)], "hello");
    assert_eq!(&text[word_range_at(text, 6)], "世");
    assert_eq!(&text[word_range_at(text, text.len() - 1)], "!");

    let unicode = "👩🏽‍💻 e\u{301}";
    assert_eq!(&unicode[word_range_at(unicode, "👩".len())], "👩🏽‍💻");
    let combining_start = "👩🏽‍💻 ".len();
    assert_eq!(
        &unicode[word_range_at(unicode, combining_start + 1)],
        "e\u{301}"
    );
}

#[test]
fn word_boundaries_move_to_word_starts_and_ends() {
    let text = "one  two 世界";
    assert_eq!(next_word_boundary(text, 0), 3);
    assert_eq!(next_word_boundary(text, 3), 8);
    assert_eq!(previous_word_boundary(text, 8), 5);
    assert_eq!(previous_word_boundary(text, text.len()), 12);
    assert_eq!(next_word_boundary("", 0), 0);
}

#[test]
fn horizontal_scroll_keeps_target_visible_and_is_clamped() {
    assert_eq!(
        ensure_x_visible(px(0.), px(120.), px(200.), px(80.)),
        px(40.)
    );
    assert_eq!(
        ensure_x_visible(px(80.), px(10.), px(200.), px(80.)),
        px(10.)
    );
    assert_eq!(ensure_x_visible(px(50.), px(10.), px(40.), px(80.)), px(0.));
    assert_eq!(
        ensure_x_visible(px(0.), px(201.), px(201.), px(80.)),
        px(121.)
    );
}

#[test]
fn caret_blink_only_runs_for_editable_collapsed_focus() {
    assert!(caret_is_visible(true, true, false, false));
    assert!(caret_should_blink(true, false, false));

    assert!(!caret_is_visible(false, true, false, false));
    assert!(!caret_is_visible(true, false, false, false));
    assert!(!caret_is_visible(true, true, true, false));
    assert!(!caret_is_visible(true, true, false, true));
    assert!(!caret_should_blink(true, true, false));
    assert!(!caret_should_blink(true, false, true));
    assert!(!caret_should_blink(false, false, false));
}

#[test]
fn shortcut_modifiers_use_secondary_without_accepting_extra_keys() {
    let secondary = Modifiers::secondary_key();
    assert!(secondary_shortcut_modifiers(secondary, false));

    let mut secondary_shift = secondary;
    secondary_shift.shift = true;
    assert!(secondary_shortcut_modifiers(secondary_shift, true));
    assert!(!secondary_shortcut_modifiers(secondary_shift, false));

    let mut secondary_alt = secondary;
    secondary_alt.alt = true;
    assert!(!secondary_shortcut_modifiers(secondary_alt, false));

    #[cfg(target_os = "macos")]
    {
        let control = Modifiers {
            control: true,
            ..Modifiers::none()
        };
        assert!(!secondary_shortcut_modifiers(control, false));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let platform = Modifiers {
            platform: true,
            ..Modifiers::none()
        };
        assert!(!secondary_shortcut_modifiers(platform, false));
    }
}

#[test]
fn movement_and_deletion_only_accept_documented_modifier_sets() {
    assert_eq!(
        horizontal_movement_kind(Modifiers::none()),
        Some(HorizontalMovement::Grapheme)
    );
    assert_eq!(deletion_kind(Modifiers::none()), Some(Deletion::Grapheme));

    let shift = Modifiers {
        shift: true,
        ..Modifiers::none()
    };
    assert_eq!(
        horizontal_movement_kind(shift),
        Some(HorizontalMovement::Grapheme)
    );
    assert_eq!(deletion_kind(shift), None);

    #[cfg(target_os = "macos")]
    {
        let word = Modifiers {
            alt: true,
            ..Modifiers::none()
        };
        let line = Modifiers {
            platform: true,
            ..Modifiers::none()
        };
        let function_line = Modifiers {
            function: true,
            ..Modifiers::none()
        };
        let unknown = Modifiers {
            control: true,
            ..Modifiers::none()
        };
        assert_eq!(
            horizontal_movement_kind(word),
            Some(HorizontalMovement::Word)
        );
        assert_eq!(deletion_kind(word), Some(Deletion::Word));
        assert_eq!(
            horizontal_movement_kind(line),
            Some(HorizontalMovement::Line)
        );
        assert_eq!(deletion_kind(line), Some(Deletion::Line));
        assert_eq!(
            horizontal_movement_kind(function_line),
            Some(HorizontalMovement::Line)
        );
        assert_eq!(deletion_kind(function_line), None);
        assert_eq!(horizontal_movement_kind(unknown), None);
        assert_eq!(deletion_kind(unknown), None);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let word = Modifiers {
            control: true,
            ..Modifiers::none()
        };
        let platform = Modifiers {
            platform: true,
            ..Modifiers::none()
        };
        assert_eq!(
            horizontal_movement_kind(word),
            Some(HorizontalMovement::Word)
        );
        assert_eq!(deletion_kind(word), Some(Deletion::Word));
        assert_eq!(horizontal_movement_kind(platform), None);
        assert_eq!(deletion_kind(platform), None);
    }
}

#[test]
fn caret_color_override_has_priority_without_changing_theme_default() {
    let theme_color = gpui::Hsla::black();
    let custom = gpui::Hsla::red();
    assert_eq!(resolved_caret_color(theme_color, None), theme_color);
    assert_eq!(resolved_caret_color(theme_color, Some(custom)), custom);
}

#[gpui::test]
fn caret_color_builder_overrides_normal_and_invalid_theme_tokens(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| CaretTestView::new(cx));
    let state = view.read_with(cx, |view, _| view.state.clone());
    cx.update(|window, cx| {
        let theme = theme::current_theme(window, cx);
        let normal = theme.input_state("outline", "normal").unwrap();
        let invalid = theme.input_state("outline", "invalid").unwrap();
        let custom = gpui::Hsla::red();
        let input = Input::new("custom-caret", state).caret_color(custom);

        assert_eq!(input.caret_color, Some(custom));
        assert_eq!(resolved_caret_color(normal.caret, None), normal.caret);
        assert_eq!(
            resolved_caret_color(normal.caret, input.caret_color),
            custom
        );
        assert_eq!(
            resolved_caret_color(invalid.caret, input.caret_color),
            custom
        );
    });
}

#[test]
fn caret_geometry_uses_font_metrics_for_all_input_sizes() {
    for (line_height, ascent, descent) in [
        (px(16.), px(9.), px(3.)),
        (px(20.), px(11.), px(4.)),
        (px(20.), px(12.), px(4.)),
        (px(24.), px(13.), px(5.)),
    ] {
        let line_bounds = Bounds::new(point(px(0.), px(0.)), size(px(120.), line_height));
        let caret = caret_bounds(line_bounds, px(7.25), px(1.), ascent, descent, 2.);
        assert_eq!(caret.size.height, ascent + descent);
        assert_eq!(caret.size.width, px(1.));
        assert!(caret.top() >= line_bounds.top());
        assert!(caret.bottom() <= line_bounds.bottom());
        assert_eq!(caret.origin.x, px(7.5));
    }
}

#[test]
fn underline_alone_removes_shell_radius() {
    for variant in [
        InputVariant::Outline,
        InputVariant::Filled,
        InputVariant::Borderless,
    ] {
        assert_eq!(input_radius(variant, px(6.)), px(6.));
    }
    assert_eq!(input_radius(InputVariant::Underline, px(6.)), px(0.));
}

#[test]
fn borderless_focus_uses_structural_then_keyboard_focus_width() {
    assert_eq!(input_focus_border_width(false, px(1.), px(2.)), px(1.));
    assert_eq!(input_focus_border_width(true, px(1.), px(2.)), px(2.));
}

#[gpui::test]
fn attached_suffix_is_full_height_outermost_and_stable_when_editor_shrinks(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, cx| AttachedLayoutTestView::new(cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let input = cx.debug_bounds("vektra-input").unwrap();
    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    let clear = cx.debug_bounds("vektra-input-clear").unwrap();
    let suffix = cx.debug_bounds("unit-inline-suffix").unwrap();
    let attached = cx.debug_bounds("vektra-input-attached-suffix").unwrap();
    let attached_content = cx.debug_bounds("unit-attached-content").unwrap();
    let border_width = cx.update(|window, cx| theme::current_theme(window, cx).input.border_width);

    assert_eq!(input.size.height, px(36.));
    assert_eq!(attached.size.height, input.size.height - border_width * 2.);
    assert_eq!(attached.top(), input.top() + border_width);
    assert_eq!(attached.bottom(), input.bottom() - border_width);
    assert_eq!(attached_content.size.height, attached.size.height);
    assert!(editor.right() <= clear.left());
    assert!(clear.right() <= suffix.left());
    assert!(suffix.right() <= attached.left());
    assert_eq!(attached.right(), input.right() - border_width);
    let wide_editor_width = editor.size.width;
    let attached_width = attached.size.width;
    let attached_height = attached.size.height;

    view.update(cx, |view, cx| {
        view.width = px(150.);
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let narrow_editor = cx.debug_bounds("vektra-input-editor").unwrap();
    let narrow_attached = cx.debug_bounds("vektra-input-attached-suffix").unwrap();
    assert!(narrow_editor.size.width < wide_editor_width);
    assert_eq!(narrow_attached.size.width, attached_width);
    assert_eq!(narrow_attached.size.height, attached_height);
}

#[gpui::test]
fn attached_suffix_keeps_the_input_semantic_height_in_a_constrained_flex_column(
    cx: &mut TestAppContext,
) {
    let (_, cx) = cx.add_window_view(|_, cx| ConstrainedAttachedLayoutTestView::new(cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));

    let input = cx.debug_bounds("vektra-input").unwrap();
    assert_eq!(input.size.height, px(36.));
}

#[test]
fn marked_text_runs_only_underline_composition_range() {
    let base = TextRun {
        len: 8,
        font: gpui::Font::default(),
        color: gpui::Hsla::black(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let runs = marked_text_runs(base, Some(&(2..6)));
    assert_eq!(
        runs.iter().map(|run| run.len).collect::<Vec<_>>(),
        [2, 4, 2]
    );
    assert!(runs[0].underline.is_none());
    assert!(runs[1].underline.is_some());
    assert!(runs[2].underline.is_none());
}

#[test]
fn accessibility_runs_chunk_long_unicode_text_and_preserve_selection() {
    let text = "中".repeat(MAX_CHARS_PER_TEXT_RUN + 10);
    let (runs, selection) = build_a11y_text_runs(&text, 0, text.len(), accesskit::NodeId);
    assert_eq!(runs.len(), 2);
    assert_eq!(
        runs[0].1.value().unwrap().chars().count(),
        MAX_CHARS_PER_TEXT_RUN
    );
    assert_eq!(runs[1].1.value().unwrap().chars().count(), 10);
    assert_eq!(selection.anchor.character_index, 0);
    assert_eq!(selection.focus.character_index, 10);
}

#[test]
fn variants_expose_stable_theme_keys() {
    assert_eq!(InputVariant::default(), InputVariant::Outline);
    assert_eq!(InputVariant::Outline.token_key(), "outline");
    assert_eq!(InputVariant::Filled.token_key(), "filled");
    assert_eq!(InputVariant::Borderless.token_key(), "borderless");
    assert_eq!(InputVariant::Underline.token_key(), "underline");
}

#[gpui::test]
fn caret_blink_uses_discrete_timer_ticks_and_resets_deterministically(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| CaretTestView::new(cx));
    let state = view.read_with(cx, |view, _| view.state.clone());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(state.read_with(cx, |state, _| state.last_caret.is_none()));

    let handle = state.read_with(cx, |state, _| state.focus_handle.clone());
    let initial_generation = state.read_with(cx, |state, _| state.caret_blink_generation);
    cx.update(|window, cx| {
        cx.activate(true);
        window.activate_window();
        window.focus(&handle, cx);
    });
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let focused_generation = state.read_with(cx, |state, _| state.caret_blink_generation);
    assert_ne!(focused_generation, initial_generation);
    assert!(state.read_with(cx, |state, _| state.last_caret_blinking));
    assert!(state.read_with(cx, |state, _| state.caret_blink_task.is_some()));
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );
    assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);

    cx.executor()
        .advance_clock(CARET_BLINK_INTERVAL - Duration::from_millis(1));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );

    cx.executor().advance_clock(Duration::from_millis(1));
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        0.
    );
    assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);

    cx.executor().advance_clock(CARET_BLINK_INTERVAL);
    cx.run_until_parked();
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );

    let scale_factor = cx.update(|window, _| window.scale_factor());
    state.read_with(cx, |state, _| {
        let caret = state.last_caret.unwrap().0;
        let line = state.last_layout.as_ref().unwrap();
        let viewport = state.last_bounds.unwrap();
        assert_eq!(caret.size.width, px(1.));
        assert_eq!(
            caret.size.height,
            snap_to_device_pixel(line.ascent + line.descent, scale_factor)
        );
        assert!(caret.size.height < viewport.size.height);
        assert!(caret.top() >= viewport.top());
        assert!(caret.bottom() <= viewport.bottom());
    });

    let before_input = state.read_with(cx, |state, _| state.caret_blink_generation);
    cx.simulate_input("d");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_ne!(
        state.read_with(cx, |state, _| state.caret_blink_generation),
        before_input
    );
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );

    let before_move = state.read_with(cx, |state, _| state.caret_blink_generation);
    cx.simulate_keystrokes("left");
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_ne!(
        state.read_with(cx, |state, _| state.caret_blink_generation),
        before_move
    );
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );

    let before_mouse = state.read_with(cx, |state, _| state.caret_blink_generation);
    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    cx.simulate_click(editor.center(), Modifiers::none());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert_ne!(
        state.read_with(cx, |state, _| state.caret_blink_generation),
        before_mouse
    );
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );

    let before_clear = state.read_with(cx, |state, _| state.caret_blink_generation);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.user_clear(window, cx));
        window.draw(cx).clear(cx);
    });
    assert_ne!(
        state.read_with(cx, |state, _| state.caret_blink_generation),
        before_clear
    );
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );
}

#[gpui::test]
fn caret_rendering_is_static_or_absent_for_special_states(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| CaretTestView::new(cx));
    let state = view.read_with(cx, |view, _| view.state.clone());
    cx.update(|window, cx| window.draw(cx).clear(cx));
    let handle = state.read_with(cx, |state, _| state.focus_handle.clone());
    cx.update(|window, cx| {
        cx.activate(true);
        window.activate_window();
        window.focus(&handle, cx);
        state.update(cx, |state, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                state,
                None,
                "preedit",
                Some(7..7),
                window,
                cx,
            );
        });
        window.draw(cx).clear(cx);
    });
    assert!(!state.read_with(cx, |state, _| state.last_caret_blinking));
    assert!(state.read_with(cx, |state, _| state.caret_blink_task.is_none()));
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );
    assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);

    cx.update(|window, cx| {
        state.update(cx, |state, cx| state.set_value("reduced", cx));
        cx.set_reduce_motion(true);
        window.draw(cx).clear(cx);
    });
    assert!(!state.read_with(cx, |state, _| state.last_caret_blinking));
    assert!(state.read_with(cx, |state, _| state.caret_blink_task.is_none()));
    assert_eq!(
        state.read_with(cx, |state, _| state.last_caret.unwrap().1),
        1.
    );
    assert_eq!(cx.update(|window, cx| window.simulate_next_frame(cx)), 0);

    state.update(cx, |state, cx| state.select_all(cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(state.read_with(cx, |state, _| state.last_caret.is_none()));

    state.update(cx, |state, cx| state.move_to(state.value.len(), cx));
    view.update(cx, |view, cx| {
        view.read_only = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(state.read_with(cx, |state, _| state.last_caret.is_none()));

    view.update(cx, |view, cx| {
        view.read_only = false;
        view.disabled = true;
        cx.notify();
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    assert!(state.read_with(cx, |state, _| state.last_caret.is_none()));

    view.update(cx, |view, cx| {
        view.disabled = false;
        cx.notify();
    });
    cx.update(|window, cx| {
        window.blur();
        window.draw(cx).clear(cx);
    });
    assert!(state.read_with(cx, |state, _| state.last_caret.is_none()));
}
