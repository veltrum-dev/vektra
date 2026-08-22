use gpui::{
    AppContext, ClipboardItem, Context, EntityInputHandler, InteractiveElement, IntoElement,
    KeyDownEvent, KeyUpEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Pixels, Render, SharedString, Styled, Subscription, TestAppContext, Window, div,
    point, px,
};
use vektra::{
    Button, ButtonVariant, Changeable, ComponentSize, Disableable, Focusable, Icon, IconButton,
    IconButtonVariant, IconSource, Input, InputClear, InputEvent, InputState, InputType,
    InputVariant, Sizable, Tooltip, TooltipPlacement,
};
use vektra_theme::{InputVariantKind, InputVisualState};

#[test]
fn input_is_a_root_export_with_standard_capabilities_and_builders() {
    fn changeable<C: Changeable<SharedString>>(component: C) -> C {
        component.on_change(|_, _, _| {})
    }
    fn disable<C: Disableable>(component: C) -> C {
        component.disabled(true)
    }
    fn sizable<C: Sizable>(component: C) -> C {
        component.size(ComponentSize::Sm)
    }
    fn focusable<C: Focusable>(component: C) -> C {
        component.on_focus(|_, _| {}).on_blur(|_, _| {})
    }

    fn accepts_input(state: gpui::Entity<InputState>) {
        let _ = changeable(focusable(sizable(disable(
            Input::new("public-input", state)
                .placeholder("用户名")
                .variant(InputVariant::Filled)
                .input_type(InputType::Email)
                .password_revealed(false)
                .disabled(false)
                .read_only(false)
                .invalid(false)
                .caret_color(gpui::Hsla::red())
                .aria_label("用户名")
                .aria_description("用于登录")
                .prefix(Icon::new(IconSource::asset("components/input/invalid.svg")))
                .suffix(div().id("public-suffix"))
                .attached_suffix(div().id("public-attached-suffix"))
                .clearable(
                    InputClear::new("清空用户名")
                        .tooltip(Tooltip::new("清空"))
                        .tooltip_placement(TooltipPlacement::Top),
                )
                .on_change(|_, _, _| {})
                .on_submit(|_, _, _| {})
                .on_focus(|_, _| {})
                .on_blur(|_, _| {}),
        ))));
    }

    let _ = accepts_input;
}

struct InputView {
    state: gpui::Entity<InputState>,
    emitted: Vec<InputEvent>,
    callbacks: Vec<InputEvent>,
    bubbled_key_downs: usize,
    suffix_clicks: usize,
    attached_clicks: usize,
    variant: InputVariant,
    input_type: InputType,
    password_revealed: bool,
    size: ComponentSize,
    width: Pixels,
    disabled: bool,
    read_only: bool,
    invalid: bool,
    clearable: bool,
    slots: bool,
    attached: bool,
    password_toggle: bool,
    _subscription: Subscription,
}

impl InputView {
    fn new(initial: &str, cx: &mut Context<Self>) -> Self {
        let state = cx.new(|cx| InputState::new(initial, cx));
        let subscription = cx.subscribe(&state, |this, _, event, _| {
            this.emitted.push(event.clone());
        });
        Self {
            state,
            emitted: Vec::new(),
            callbacks: Vec::new(),
            bubbled_key_downs: 0,
            suffix_clicks: 0,
            attached_clicks: 0,
            variant: InputVariant::Outline,
            input_type: InputType::Text,
            password_revealed: false,
            size: ComponentSize::Md,
            width: px(320.),
            disabled: false,
            read_only: false,
            invalid: false,
            clearable: false,
            slots: false,
            attached: false,
            password_toggle: false,
            _subscription: subscription,
        }
    }
}

impl Render for InputView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut input = Input::new("target-input", self.state.clone())
            .placeholder("输入中文、emoji 或文字")
            .variant(self.variant)
            .input_type(self.input_type)
            .password_revealed(self.password_revealed)
            .size(self.size)
            .disabled(self.disabled)
            .read_only(self.read_only)
            .invalid(self.invalid)
            .aria_label("测试输入")
            .aria_description("单行文本")
            .on_change_in(cx, |this, value, _, _| {
                this.callbacks.push(InputEvent::Changed(value));
            })
            .on_submit_in(cx, |this, value, _, _| {
                this.callbacks.push(InputEvent::Submitted(value));
            })
            .on_focus_in(cx, |this, _, _| {
                this.callbacks.push(InputEvent::Focused);
            })
            .on_blur_in(cx, |this, _, _| {
                this.callbacks.push(InputEvent::Blurred);
            });
        if self.slots {
            input = input
                .prefix(Icon::new(IconSource::asset("components/input/invalid.svg")))
                .suffix(
                    div().debug_selector(|| "input-test-suffix".into()).child(
                        IconButton::new(
                            "suffix-action",
                            IconSource::asset("components/input/invalid.svg"),
                        )
                        .variant(IconButtonVariant::Ghost)
                        .size(ComponentSize::Xs)
                        .aria_label("后缀操作")
                        .on_click_in(cx, |this, _, _, _| this.suffix_clicks += 1),
                    ),
                );
        }
        if self.password_toggle {
            let revealed = self.password_revealed;
            input = input.suffix(
                div()
                    .debug_selector(|| "input-test-password-toggle".into())
                    .child(
                        IconButton::new(
                            "password-toggle",
                            IconSource::asset("components/input/invalid.svg"),
                        )
                        .variant(IconButtonVariant::Ghost)
                        .size(ComponentSize::Xs)
                        .selected(revealed)
                        .aria_label(if revealed {
                            "隐藏密码"
                        } else {
                            "显示密码"
                        })
                        .on_click_in(cx, |this, _, window, cx| {
                            this.password_revealed = !this.password_revealed;
                            let handle = this.state.read(cx).focus_handle().clone();
                            window.focus(&handle, cx);
                            cx.notify();
                        }),
                    ),
            );
        }
        if self.attached {
            input = input.attached_suffix(
                div().debug_selector(|| "input-test-attached".into()).child(
                    Button::new("attached-action")
                        .label("搜索")
                        .variant(ButtonVariant::Ghost)
                        .size(self.size)
                        .disabled(self.disabled)
                        .on_click_in(cx, |this, _, _, cx| {
                            let value = this.state.read(cx).value().to_owned();
                            this.callbacks.push(InputEvent::Submitted(value.into()));
                            this.attached_clicks += 1;
                        }),
                ),
            );
        }
        if self.clearable {
            input = input.clearable(
                InputClear::new("清空测试输入")
                    .tooltip(Tooltip::new("清空"))
                    .tooltip_placement(TooltipPlacement::Top),
            );
        }
        div()
            .id("input-test-root")
            .debug_selector(|| "input-test-root".into())
            .w(self.width)
            .on_key_down(cx.listener(|this, _: &KeyDownEvent, _, _| {
                this.bubbled_key_downs += 1;
            }))
            .child(input)
    }
}

fn draw(cx: &mut gpui::VisualTestContext) {
    cx.update(|window, cx| window.draw(cx).clear(cx));
}

fn state(view: &gpui::Entity<InputView>, cx: &gpui::VisualTestContext) -> gpui::Entity<InputState> {
    view.read_with(cx, |view, _| view.state.clone())
}

fn value(view: &gpui::Entity<InputView>, cx: &gpui::VisualTestContext) -> String {
    state(view, cx).read_with(cx, |state, _| state.value().to_owned())
}

fn focus_editor(view: &gpui::Entity<InputView>, cx: &mut gpui::VisualTestContext) {
    let state = state(view, cx);
    let handle = state.read_with(cx, |state, _| state.focus_handle().clone());
    cx.update(|window, cx| {
        cx.activate(true);
        window.activate_window();
        window.focus(&handle, cx);
        window.draw(cx).clear(cx);
    });
}

fn key_down(key: &str, cx: &mut gpui::VisualTestContext) {
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke::parse(key).unwrap(),
        is_held: false,
        prefer_character_input: false,
    });
}

fn key_up(key: &str, cx: &mut gpui::VisualTestContext) {
    cx.simulate_event(KeyUpEvent {
        keystroke: Keystroke::parse(key).unwrap(),
    });
}

fn key_cycle(key: &str, cx: &mut gpui::VisualTestContext) {
    key_down(key, cx);
    key_up(key, cx);
}

#[cfg(target_os = "macos")]
fn command(key: &str) -> String {
    format!("cmd-{key}")
}

#[cfg(not(target_os = "macos"))]
fn command(key: &str) -> String {
    format!("ctrl-{key}")
}

#[cfg(target_os = "macos")]
const UNKNOWN_PLATFORM_ARROW: &str = "ctrl-left";

#[cfg(not(target_os = "macos"))]
const UNKNOWN_PLATFORM_ARROW: &str = "cmd-left";

#[gpui::test]
fn unsupported_modifier_combinations_bubble_without_moving_the_caret(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("abc", cx));
    draw(cx);
    focus_editor(&view, cx);
    let input_state = state(&view, cx);

    key_down(UNKNOWN_PLATFORM_ARROW, cx);
    let selected_range = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, false, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(selected_range, 3..3);
    assert_eq!(view.read_with(cx, |view, _| view.bubbled_key_downs), 1);

    key_down("left", cx);
    let selected_range = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, false, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(selected_range, 2..2);
    assert_eq!(view.read_with(cx, |view, _| view.bubbled_key_downs), 1);
}

#[gpui::test]
fn text_input_selection_clipboard_history_and_grapheme_deletion_are_single_line(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("", cx));
    draw(cx);
    focus_editor(&view, cx);
    cx.simulate_input("A中👩🏽‍💻e\u{301}");
    assert_eq!(value(&view, cx), "A中👩🏽‍💻e\u{301}");

    key_down("backspace", cx);
    assert_eq!(value(&view, cx), "A中👩🏽‍💻");
    key_down("backspace", cx);
    assert_eq!(value(&view, cx), "A中");

    cx.write_to_clipboard(ClipboardItem::new_string("x\r\ny\nz".into()));
    cx.simulate_keystrokes(&command("v"));
    assert_eq!(value(&view, cx), "A中x y z");
    cx.simulate_keystrokes(&command("z"));
    assert_eq!(value(&view, cx), "A中");
    cx.simulate_keystrokes(&command("shift-z"));
    assert_eq!(value(&view, cx), "A中x y z");

    cx.simulate_keystrokes(&command("a"));
    cx.simulate_keystrokes(&command("x"));
    assert_eq!(value(&view, cx), "");
    assert_eq!(
        cx.read_from_clipboard().unwrap().text().unwrap(),
        "A中x y z"
    );

    let callback_changes = view.read_with(cx, |view, _| {
        view.callbacks
            .iter()
            .filter(|event| matches!(event, InputEvent::Changed(_)))
            .count()
    });
    let emitted_changes = view.read_with(cx, |view, _| {
        view.emitted
            .iter()
            .filter(|event| matches!(event, InputEvent::Changed(_)))
            .count()
    });
    assert_eq!(callback_changes, emitted_changes);
}

#[gpui::test]
fn hidden_password_blocks_copy_and_cut_but_allows_paste_then_reveal_restores_clipboard(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("机密👩🏽‍💻", cx);
        view.input_type = InputType::Password;
        view
    });
    draw(cx);
    focus_editor(&view, cx);
    cx.simulate_keystrokes(&command("a"));

    cx.write_to_clipboard(ClipboardItem::new_string("保留剪贴板".into()));
    cx.simulate_keystrokes(&command("c"));
    assert_eq!(
        cx.read_from_clipboard().unwrap().text().unwrap(),
        "保留剪贴板"
    );
    cx.simulate_keystrokes(&command("x"));
    assert_eq!(value(&view, cx), "机密👩🏽‍💻");
    assert_eq!(
        cx.read_from_clipboard().unwrap().text().unwrap(),
        "保留剪贴板"
    );

    cx.write_to_clipboard(ClipboardItem::new_string("粘贴值e\u{301}".into()));
    cx.simulate_keystrokes(&command("v"));
    assert_eq!(value(&view, cx), "粘贴值e\u{301}");
    assert!(view.read_with(cx, |view, _| {
        view.emitted
            .contains(&InputEvent::Changed("粘贴值e\u{301}".into()))
            && view
                .callbacks
                .contains(&InputEvent::Changed("粘贴值e\u{301}".into()))
    }));

    key_down("enter", cx);
    assert!(view.read_with(cx, |view, _| {
        view.emitted
            .contains(&InputEvent::Submitted("粘贴值e\u{301}".into()))
            && view
                .callbacks
                .contains(&InputEvent::Submitted("粘贴值e\u{301}".into()))
    }));

    view.update(cx, |view, cx| {
        view.password_revealed = true;
        cx.notify();
    });
    draw(cx);
    cx.simulate_keystrokes(&command("a"));
    cx.simulate_keystrokes(&command("c"));
    assert_eq!(
        cx.read_from_clipboard().unwrap().text().unwrap(),
        "粘贴值e\u{301}"
    );
    cx.simulate_keystrokes(&command("x"));
    assert_eq!(value(&view, cx), "");
}

#[gpui::test]
fn hidden_read_only_password_never_copies_or_cuts(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("只读机密", cx);
        view.input_type = InputType::Password;
        view.read_only = true;
        view.invalid = true;
        view
    });
    draw(cx);
    assert!(cx.debug_bounds("vektra-input-invalid").is_some());
    focus_editor(&view, cx);
    cx.simulate_keystrokes(&command("a"));
    cx.write_to_clipboard(ClipboardItem::new_string("原剪贴板".into()));
    cx.simulate_keystrokes(&command("c"));
    cx.simulate_keystrokes(&command("x"));

    assert_eq!(value(&view, cx), "只读机密");
    assert_eq!(
        cx.read_from_clipboard().unwrap().text().unwrap(),
        "原剪贴板"
    );
}

#[gpui::test]
fn hidden_disabled_password_rejects_edit_and_selection_and_retains_invalid_state(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("禁用机密", cx);
        view.input_type = InputType::Password;
        view.disabled = true;
        view.invalid = true;
        view
    });
    draw(cx);

    assert!(cx.debug_bounds("vektra-input-invalid").is_some());
    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    cx.simulate_click(editor.center(), Modifiers::none());
    cx.simulate_input("不应写入");

    assert_eq!(value(&view, cx), "禁用机密");
    let input_state = state(&view, cx);
    assert!(cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, false, window, cx).is_none()
        })
    }));
}

#[gpui::test]
fn password_toggle_restores_editor_focus_without_changing_value_selection_or_events(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("中👨‍👩‍👧‍👦e\u{301}", cx);
        view.input_type = InputType::Password;
        view.password_toggle = true;
        view
    });
    draw(cx);
    focus_editor(&view, cx);
    key_down("left", cx);
    let input_state = state(&view, cx);
    let selection_before = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx).unwrap()
        })
    });
    let events_before = view.read_with(cx, |view, _| view.emitted.clone());

    let toggle = cx.debug_bounds("input-test-password-toggle").unwrap();
    cx.simulate_click(toggle.center(), Modifiers::none());
    draw(cx);

    assert!(view.read_with(cx, |view, _| view.password_revealed));
    assert_eq!(value(&view, cx), "中👨‍👩‍👧‍👦e\u{301}");
    let selection_after = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx).unwrap()
        })
    });
    assert_eq!(selection_after.range, selection_before.range);
    assert_eq!(selection_after.reversed, selection_before.reversed);
    assert_eq!(
        view.read_with(cx, |view, _| view.emitted.clone()),
        events_before
    );
    let handle = input_state.read_with(cx, |state, _| state.focus_handle().clone());
    assert!(cx.update(|window, _| handle.is_focused(window)));
}

#[gpui::test]
fn hidden_password_mouse_keyboard_delete_and_history_use_real_grapheme_boundaries(
    cx: &mut TestAppContext,
) {
    let original = "A中👨‍👩‍👧‍👦e\u{301}";
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new(original, cx);
        view.input_type = InputType::Password;
        view
    });
    draw(cx);
    focus_editor(&view, cx);

    key_down("left", cx);
    key_down("backspace", cx);
    assert_eq!(value(&view, cx), "A中e\u{301}");
    cx.simulate_keystrokes(&command("z"));
    assert_eq!(value(&view, cx), original);
    cx.simulate_keystrokes(&command("shift-z"));
    assert_eq!(value(&view, cx), "A中e\u{301}");
    cx.simulate_keystrokes(&command("z"));

    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    let start = point(editor.left() + px(1.), editor.center().y);
    let end = point(editor.right() - px(1.), editor.center().y);
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: start,
        modifiers: Modifiers::none(),
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_mouse_move(end, None, Modifiers::none());
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: end,
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    let input_state = state(&view, cx);
    let selected = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(selected, 0..original.encode_utf16().count());
    assert_eq!(value(&view, cx), original);
}

#[gpui::test]
fn ime_preedit_is_silent_commit_is_once_and_enter_does_not_submit(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("前", cx));
    draw(cx);
    focus_editor(&view, cx);
    let state = state(&view, cx);

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                state,
                None,
                "zhong",
                Some(5..5),
                window,
                cx,
            );
        });
    });
    assert_eq!(
        state.read_with(cx, |state, _| state.value().to_owned()),
        "前zhong"
    );
    assert!(view.read_with(cx, |view, _| view.emitted.is_empty()));

    key_down("enter", cx);
    assert!(!view.read_with(cx, |view, _| {
        view.emitted
            .iter()
            .any(|event| matches!(event, InputEvent::Submitted(_)))
    }));

    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            EntityInputHandler::replace_text_in_range(state, None, "中", window, cx);
        });
    });
    assert_eq!(
        state.read_with(cx, |state, _| state.value().to_owned()),
        "前中"
    );
    assert_eq!(
        view.read_with(cx, |view, _| {
            view.emitted
                .iter()
                .filter(|event| matches!(event, InputEvent::Changed(_)))
                .count()
        }),
        1
    );
    assert_eq!(
        view.read_with(cx, |view, _| {
            view.callbacks
                .iter()
                .filter(|event| matches!(event, InputEvent::Changed(_)))
                .cloned()
                .collect::<Vec<_>>()
        }),
        [InputEvent::Changed("前中".into())]
    );
}

#[gpui::test]
fn programmatic_updates_end_composition_without_user_events(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("初始", cx));
    draw(cx);
    let state = state(&view, cx);
    cx.update(|window, cx| {
        state.update(cx, |state, cx| {
            EntityInputHandler::replace_and_mark_text_in_range(
                state,
                None,
                "preedit",
                Some(7..7),
                window,
                cx,
            );
            state.set_value("程序值", cx);
            assert!(EntityInputHandler::marked_text_range(state, window, cx).is_none());
            state.clear(cx);
            state.reset("重置", cx);
        });
    });

    assert_eq!(
        state.read_with(cx, |state, _| state.value().to_owned()),
        "重置"
    );
    assert!(view.read_with(cx, |view, _| view.emitted.is_empty()));
    assert!(view.read_with(cx, |view, _| view.callbacks.is_empty()));
}

#[gpui::test]
fn programmatic_set_value_clears_user_undo_and_redo_history(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("初始", cx));
    draw(cx);
    focus_editor(&view, cx);
    cx.simulate_input("用户编辑");
    assert_eq!(value(&view, cx), "初始用户编辑");

    let input_state = state(&view, cx);
    input_state.update(cx, |state, cx| state.set_value("外部同步", cx));
    cx.simulate_keystrokes(&command("z"));
    assert_eq!(value(&view, cx), "外部同步");
    cx.simulate_keystrokes(&command("shift-z"));
    assert_eq!(value(&view, cx), "外部同步");
}

#[gpui::test]
fn ime_selection_is_normalized_against_the_updated_full_value(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("AexB", cx));
    draw(cx);
    let input_state = state(&view, cx);

    for (initial, selected, replacement, expected, marked) in [
        ("AexB", 2..3, "\u{301}", "Ae\u{301}B", 2..3),
        ("A👨xB", 3..4, "\u{200d}👩", "A👨\u{200d}👩B", 3..6),
    ] {
        cx.update(|window, cx| {
            input_state.update(cx, |state, cx| {
                state.reset(initial, cx);
                EntityInputHandler::set_selected_text_range(state, selected, window, cx);
                EntityInputHandler::replace_and_mark_text_in_range(
                    state,
                    None,
                    replacement,
                    Some(0..0),
                    window,
                    cx,
                );
                let selection =
                    EntityInputHandler::selected_text_range(state, true, window, cx).unwrap();
                assert_eq!(selection.range, 1..1);
                assert!(!selection.reversed);
                assert_eq!(
                    EntityInputHandler::marked_text_range(state, window, cx),
                    Some(marked)
                );
            });
        });
        assert_eq!(value(&view, cx), expected);
    }
}

#[gpui::test]
fn ime_bounds_reject_stale_display_and_recover_after_redraw(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("iiii", cx));
    draw(cx);
    let input_state = state(&view, cx);
    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    let bounds = |input_state: &gpui::Entity<InputState>, cx: &mut gpui::VisualTestContext| {
        cx.update(|window, cx| {
            input_state.update(cx, |state, cx| {
                EntityInputHandler::bounds_for_range(state, 0..4, editor, window, cx)
            })
        })
    };
    assert!(bounds(&input_state, cx).is_some());

    cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            state.set_value("WWWW", cx);
            assert!(
                EntityInputHandler::bounds_for_range(state, 0..4, editor, window, cx,).is_none()
            );
        });
    });
    draw(cx);
    assert!(bounds(&input_state, cx).is_some());

    view.update(cx, |view, cx| {
        view.input_type = InputType::Password;
        cx.notify();
    });
    input_state.update(cx, |state, cx| state.set_value("甲乙", cx));
    draw(cx);
    cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            state.set_value("甲乙丙丁", cx);
            assert!(
                EntityInputHandler::bounds_for_range(state, 0..4, editor, window, cx,).is_none()
            );
        });
    });
    draw(cx);
    assert!(bounds(&input_state, cx).is_some());
}

#[gpui::test]
fn word_and_line_deletion_respect_platform_modifiers_and_undo(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("hello 世界! e\u{301}", cx));
    draw(cx);
    focus_editor(&view, cx);

    #[cfg(target_os = "macos")]
    key_down("alt-backspace", cx);
    #[cfg(not(target_os = "macos"))]
    key_down("ctrl-backspace", cx);
    assert_eq!(value(&view, cx), "hello 世界! ");
    cx.simulate_keystrokes(&command("z"));
    assert_eq!(value(&view, cx), "hello 世界! e\u{301}");

    cx.simulate_keystrokes(&command("a"));
    key_down("left", cx);
    #[cfg(target_os = "macos")]
    key_down("alt-delete", cx);
    #[cfg(not(target_os = "macos"))]
    key_down("ctrl-delete", cx);
    assert_eq!(value(&view, cx), " 世界! e\u{301}");
    cx.simulate_keystrokes(&command("z"));
    assert_eq!(value(&view, cx), "hello 世界! e\u{301}");

    let input_state = state(&view, cx);
    cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::set_selected_text_range(state, 6..8, window, cx);
        });
    });
    #[cfg(target_os = "macos")]
    key_down("alt-backspace", cx);
    #[cfg(not(target_os = "macos"))]
    key_down("ctrl-backspace", cx);
    assert_eq!(value(&view, cx), "hello ! e\u{301}");

    view.update(cx, |view, cx| {
        view.read_only = true;
        cx.notify();
    });
    draw(cx);
    #[cfg(target_os = "macos")]
    key_down("cmd-backspace", cx);
    #[cfg(not(target_os = "macos"))]
    key_down("ctrl-backspace", cx);
    assert_eq!(value(&view, cx), "hello ! e\u{301}");

    view.update(cx, |view, cx| {
        view.read_only = false;
        view.disabled = true;
        cx.notify();
    });
    draw(cx);
    key_down("backspace", cx);
    assert_eq!(value(&view, cx), "hello ! e\u{301}");

    #[cfg(target_os = "macos")]
    {
        view.update(cx, |view, cx| {
            view.disabled = false;
            cx.notify();
        });
        draw(cx);
        focus_editor(&view, cx);
        key_down("end", cx);
        key_down("cmd-backspace", cx);
        assert_eq!(value(&view, cx), "");
        cx.simulate_keystrokes(&command("z"));
        assert_eq!(value(&view, cx), "hello ! e\u{301}");
        cx.simulate_keystrokes(&command("a"));
        key_down("left", cx);
        key_down("cmd-delete", cx);
        assert_eq!(value(&view, cx), "");
    }
}

#[gpui::test]
fn disabled_rejects_focus_and_selection_while_read_only_allows_selection_only(
    cx: &mut TestAppContext,
) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("原值", cx);
        view.disabled = true;
        view.clearable = true;
        view
    });
    draw(cx);
    let input_state = state(&view, cx);
    cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            state.select_all(cx);
            EntityInputHandler::replace_text_in_range(state, None, "修改", window, cx);
            assert!(EntityInputHandler::selected_text_range(state, false, window, cx).is_none());
        });
    });
    assert_eq!(
        input_state.read_with(cx, |state, _| state.value().to_owned()),
        "原值"
    );

    let (read_only, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("只读值", cx);
        view.read_only = true;
        view.clearable = true;
        view
    });
    draw(cx);
    focus_editor(&read_only, cx);
    let read_only_state = state(&read_only, cx);
    cx.update(|window, cx| {
        read_only_state.update(cx, |state, cx| {
            state.select_all(cx);
            assert_eq!(
                EntityInputHandler::selected_text_range(state, false, window, cx)
                    .unwrap()
                    .range,
                0..3
            );
            EntityInputHandler::replace_text_in_range(state, None, "修改", window, cx);
        });
    });
    assert_eq!(
        read_only_state.read_with(cx, |state, _| state.value().to_owned()),
        "只读值"
    );
}

#[gpui::test]
fn clear_is_one_user_change_and_restores_editor_focus(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("可清空", cx);
        view.clearable = true;
        view
    });
    draw(cx);
    focus_editor(&view, cx);
    cx.run_until_parked();
    draw(cx);

    let clear_bounds = cx.debug_bounds("vektra-input-clear").unwrap();
    assert_eq!(clear_bounds.size.width, px(24.));
    assert_eq!(clear_bounds.size.height, px(24.));
    cx.simulate_mouse_move(clear_bounds.center(), None, Modifiers::none());
    cx.executor()
        .advance_clock(std::time::Duration::from_millis(500));
    cx.run_until_parked();
    draw(cx);
    assert!(cx.debug_bounds("vektra-tooltip-bubble").is_some());

    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: clear_bounds.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
        first_mouse: false,
    });
    assert_eq!(value(&view, cx), "可清空");
    cx.simulate_event(MouseUpEvent {
        button: MouseButton::Left,
        position: clear_bounds.center(),
        modifiers: Modifiers::none(),
        click_count: 1,
    });
    assert_eq!(value(&view, cx), "");
    assert_eq!(
        view.read_with(cx, |view, _| view.emitted.clone()),
        [InputEvent::Focused, InputEvent::Changed("".into())]
    );
    let handle = state(&view, cx).read_with(cx, |state, _| state.focus_handle().clone());
    assert!(cx.update(|window, _| handle.is_focused(window)));

    state(&view, cx).update(cx, |state, cx| state.set_value("键盘", cx));
    draw(cx);
    cx.update(|window, cx| {
        window.focus_next(cx);
    });
    key_cycle("space", cx);
    assert_eq!(value(&view, cx), "");
    assert_eq!(
        view.read_with(cx, |view, _| {
            view.emitted
                .iter()
                .filter(|event| matches!(event, InputEvent::Changed(_)))
                .count()
        }),
        2
    );
}

#[gpui::test]
fn single_shift_double_and_triple_click_update_selection_without_changed(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("hello 世界 👩🏽‍💻 e\u{301}", cx));
    draw(cx);
    let editor = cx.debug_bounds("vektra-input-editor").unwrap();

    let mouse_down = |position, modifiers, click_count| MouseDownEvent {
        button: MouseButton::Left,
        position,
        modifiers,
        click_count,
        first_mouse: false,
    };
    let mouse_up = |position, modifiers, click_count| MouseUpEvent {
        button: MouseButton::Left,
        position,
        modifiers,
        click_count,
    };

    let start = point(editor.left() + px(1.), editor.center().y);
    let end = point(editor.right() - px(1.), editor.center().y);
    cx.simulate_event(mouse_down(start, Modifiers::none(), 1));
    cx.simulate_event(mouse_up(start, Modifiers::none(), 1));
    cx.simulate_event(mouse_down(
        end,
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
        1,
    ));
    cx.simulate_event(mouse_up(
        end,
        Modifiers {
            shift: true,
            ..Modifiers::none()
        },
        1,
    ));
    let input_state = state(&view, cx);
    let shifted = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx)
                .unwrap()
                .range
        })
    });
    assert!(!shifted.is_empty());

    let hello = point(editor.left() + px(12.), editor.center().y);
    cx.simulate_event(mouse_down(hello, Modifiers::none(), 2));
    cx.simulate_event(mouse_up(hello, Modifiers::none(), 2));
    let word = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(word, 0..5);

    cx.simulate_event(mouse_down(editor.center(), Modifiers::none(), 3));
    cx.simulate_event(mouse_up(editor.center(), Modifiers::none(), 3));
    let all = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(all, 0.."hello 世界 👩🏽‍💻 e\u{301}".encode_utf16().count());
    assert!(!view.read_with(cx, |view, _| {
        view.emitted
            .iter()
            .any(|event| matches!(event, InputEvent::Changed(_)))
    }));
}

#[gpui::test]
fn read_only_allows_double_and_triple_click_selection(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("read only", cx);
        view.read_only = true;
        view
    });
    draw(cx);
    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: point(editor.left() + px(10.), editor.center().y),
        modifiers: Modifiers::none(),
        click_count: 2,
        first_mouse: false,
    });
    let input_state = state(&view, cx);
    let word = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(word, 0..4);

    cx.simulate_event(MouseDownEvent {
        button: MouseButton::Left,
        position: editor.center(),
        modifiers: Modifiers::none(),
        click_count: 3,
        first_mouse: false,
    });
    let all = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx)
                .unwrap()
                .range
        })
    });
    assert_eq!(all, 0.."read only".len());
    assert_eq!(value(&view, cx), "read only");
}

#[gpui::test]
fn interactive_suffix_isolated_from_editor_value_and_submission(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("保持", cx);
        view.slots = true;
        view.clearable = true;
        view
    });
    draw(cx);
    let suffix = cx.debug_bounds("input-test-suffix").unwrap();
    let input_state = state(&view, cx);
    let selection_before = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx).unwrap()
        })
    });
    cx.simulate_click(suffix.center(), Modifiers::none());

    assert_eq!(view.read_with(cx, |view, _| view.suffix_clicks), 1);
    assert_eq!(
        input_state.read_with(cx, |state, _| state.value().to_owned()),
        "保持"
    );
    let selection_after = cx.update(|window, cx| {
        input_state.update(cx, |state, cx| {
            EntityInputHandler::selected_text_range(state, true, window, cx).unwrap()
        })
    });
    assert_eq!(selection_before.range, selection_after.range);
    assert_eq!(selection_before.reversed, selection_after.reversed);
    assert!(view.read_with(cx, |view, _| view.emitted.is_empty()));
}

#[gpui::test]
fn clear_precedes_suffix_in_layout_and_tab_order(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("可清除", cx);
        view.clearable = true;
        view.slots = true;
        view.attached = true;
        view
    });
    draw(cx);

    let clear = cx.debug_bounds("vektra-input-clear").unwrap();
    let suffix = cx.debug_bounds("input-test-suffix").unwrap();
    let attached = cx.debug_bounds("vektra-input-attached-suffix").unwrap();
    assert!(clear.right() <= suffix.left());
    assert!(suffix.right() <= attached.left());

    focus_editor(&view, cx);
    cx.update(|window, cx| window.focus_next(cx));
    key_cycle("space", cx);
    assert_eq!(value(&view, cx), "");
    assert_eq!(view.read_with(cx, |view, _| view.suffix_clicks), 0);

    state(&view, cx).update(cx, |state, cx| state.set_value("再次", cx));
    draw(cx);
    focus_editor(&view, cx);
    cx.update(|window, cx| {
        window.focus_next(cx);
        window.focus_next(cx);
    });
    key_cycle("space", cx);
    assert_eq!(value(&view, cx), "再次");
    assert_eq!(view.read_with(cx, |view, _| view.suffix_clicks), 1);

    focus_editor(&view, cx);
    cx.update(|window, cx| {
        window.focus_next(cx);
        window.focus_next(cx);
        window.focus_next(cx);
    });
    key_cycle("enter", cx);
    assert_eq!(value(&view, cx), "再次");
    assert_eq!(view.read_with(cx, |view, _| view.attached_clicks), 1);
}

#[gpui::test]
fn attached_suffix_mouse_enter_space_and_editor_enter_share_submission(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("查询值", cx);
        view.attached = true;
        view
    });
    draw(cx);

    focus_editor(&view, cx);
    key_down("enter", cx);

    cx.update(|window, cx| window.focus_next(cx));
    key_cycle("enter", cx);
    key_cycle("space", cx);

    let attached = cx.debug_bounds("input-test-attached").unwrap();
    cx.simulate_click(attached.center(), Modifiers::none());

    let submissions = view.read_with(cx, |view, _| {
        view.callbacks
            .iter()
            .filter_map(|event| match event {
                InputEvent::Submitted(value) => Some(value.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
    });
    assert_eq!(
        submissions,
        [
            SharedString::from("查询值"),
            SharedString::from("查询值"),
            SharedString::from("查询值"),
            SharedString::from("查询值")
        ]
    );
    assert_eq!(view.read_with(cx, |view, _| view.attached_clicks), 3);
}

#[gpui::test]
fn focus_blur_submit_and_escape_follow_one_semantic_path(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| InputView::new("提交值", cx));
    draw(cx);
    focus_editor(&view, cx);
    key_down("enter", cx);
    key_down("escape", cx);
    cx.update(|window, _| window.blur());

    let emitted = view.read_with(cx, |view, _| view.emitted.clone());
    assert_eq!(emitted.len(), 3);
    assert!(emitted.contains(&InputEvent::Focused));
    assert!(emitted.contains(&InputEvent::Submitted("提交值".into())));
    assert!(emitted.contains(&InputEvent::Blurred));
    let callbacks = view.read_with(cx, |view, _| view.callbacks.clone());
    assert_eq!(callbacks.len(), 3);
    assert!(callbacks.contains(&InputEvent::Focused));
    assert!(callbacks.contains(&InputEvent::Submitted("提交值".into())));
    assert!(callbacks.contains(&InputEvent::Blurred));
}

#[gpui::test]
fn variants_sizes_and_invalid_state_render_without_panicking(cx: &mut TestAppContext) {
    for variant in [
        InputVariant::Outline,
        InputVariant::Filled,
        InputVariant::Borderless,
        InputVariant::Underline,
    ] {
        for size in [
            ComponentSize::Xs,
            ComponentSize::Sm,
            ComponentSize::Md,
            ComponentSize::Lg,
        ] {
            let (_, cx) = cx.add_window_view(|_, cx| {
                let mut view = InputView::new("很长的中文和 emoji 👨‍👩‍👧‍👦 text", cx);
                view.variant = variant;
                view.size = size;
                view.invalid = true;
                view.slots = true;
                view.attached = true;
                view.clearable = true;
                view
            });
            draw(cx);
        }
    }
}

#[gpui::test]
fn narrow_layout_shrinks_editor_keeps_attached_suffix_stable_and_contains_shell(
    cx: &mut TestAppContext,
) {
    let widths = [px(180.), px(80.), px(40.)];
    let extreme_width = widths[2];
    let mut editor_widths = Vec::with_capacity(widths.len());
    let mut attached_size = None;

    for width in widths {
        let (_, cx) = cx.add_window_view(|_, cx| {
            let mut view = InputView::new("很长的中文和 emoji 👨‍👩‍👧‍👦 text", cx);
            view.size = ComponentSize::Xs;
            view.width = width;
            view.attached = true;
            view
        });
        draw(cx);

        let root = cx.debug_bounds("input-test-root").unwrap();
        let shell = cx.debug_bounds("vektra-input").unwrap();
        let content = cx.debug_bounds("vektra-input-content").unwrap();
        let editor = cx.debug_bounds("vektra-input-editor").unwrap();
        let attached = cx.debug_bounds("vektra-input-attached-suffix").unwrap();

        assert_eq!(root.size.width, width, "测试夹具应应用请求宽度");
        assert_eq!(shell.size.width, width, "Input 外壳不应撑破父级宽度");
        assert_eq!(shell.left(), root.left());
        assert_eq!(shell.right(), root.right());
        assert_eq!(content.right(), attached.left());
        assert!(editor.left() >= content.left());
        assert!(editor.right() <= content.right());

        let current_attached_size = (attached.size.width, attached.size.height);
        if let Some(expected) = attached_size {
            assert_eq!(
                current_attached_size, expected,
                "attached suffix 不应随编辑区变窄而收缩"
            );
        } else {
            attached_size = Some(current_attached_size);
        }
        editor_widths.push(editor.size.width);
    }

    assert!(editor_widths[0] > editor_widths[1]);
    assert!(editor_widths[1] > editor_widths[2]);
    assert!(
        extreme_width < attached_size.unwrap().0,
        "极窄场景应小于不可收缩的 attached suffix"
    );
}

#[gpui::test]
fn borderless_pointer_and_keyboard_focus_keep_invalid_marker(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| {
        let mut view = InputView::new("invalid", cx);
        view.variant = InputVariant::Borderless;
        view.invalid = true;
        view
    });
    draw(cx);
    assert!(cx.debug_bounds("vektra-input-invalid").is_some());
    cx.update(|window, cx| {
        let theme = vektra::current_theme(window, cx);
        let pointer =
            theme.input_state(InputVariantKind::Borderless, InputVisualState::FocusVisible);
        let invalid = theme.input_state(InputVariantKind::Borderless, InputVisualState::Invalid);
        let invalid_focus = theme.input_state(
            InputVariantKind::Borderless,
            InputVisualState::InvalidFocusVisible,
        );
        assert!(!pointer.border.is_transparent());
        assert!(!invalid_focus.border.is_transparent());
        assert_eq!(invalid.status, invalid_focus.status);
    });

    let editor = cx.debug_bounds("vektra-input-editor").unwrap();
    cx.simulate_click(editor.center(), Modifiers::none());
    draw(cx);
    assert!(cx.debug_bounds("vektra-input-invalid").is_some());

    let handle = state(&view, cx).read_with(cx, |state, _| state.focus_handle().clone());
    cx.update(|window, _| window.blur());
    key_down("tab", cx);
    cx.update(|window, cx| window.focus(&handle, cx));
    draw(cx);
    assert!(cx.debug_bounds("vektra-input-invalid").is_some());
}
