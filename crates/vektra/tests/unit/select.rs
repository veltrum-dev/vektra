use super::*;
use gpui::{
    Context, Element, ElementId, InteractiveElement, IntoElement, Render, Role, Window, div,
};

struct EmptyView;

impl Render for EmptyView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[derive(Clone, PartialEq)]
enum Value {
    A,
    B,
    C,
    D,
}

fn option(id: &'static str, value: Value, disabled: bool) -> SelectOption<Value> {
    SelectOption::new(id, value, id).disabled(disabled)
}

#[test]
fn duplicate_ids_and_values_use_first_canonical_option() {
    let children = vec![
        SelectChild::Option(option("a", Value::A, false)),
        SelectChild::Group(
            SelectGroup::new("group", "Group")
                .option(option("a", Value::B, false))
                .option(option("c", Value::A, false))
                .option(option("d", Value::D, true)),
        ),
        SelectChild::Option(option("e", Value::C, false)),
    ];
    let flat = option_catalog(&children);
    assert_eq!(
        flat.iter().map(|entry| entry.canonical).collect::<Vec<_>>(),
        [true, false, false, true, true]
    );
    assert_eq!(
        flat.iter()
            .map(|entry| (entry.position, entry.set_size, entry.disabled))
            .collect::<Vec<_>>(),
        [
            (0, 5, false),
            (1, 5, true),
            (2, 5, true),
            (3, 5, true),
            (4, 5, false),
        ]
    );
}

#[test]
fn trigger_placeholder_is_not_duplicated_as_an_accessibility_value() {
    let placeholder: SharedString = "选择方案".into();
    assert_eq!(
        trigger_accessibility::<Value>(None, &placeholder),
        TriggerAccessibility {
            value: None,
            placeholder: Some(placeholder.clone()),
        }
    );

    let catalog = option_catalog(&[SelectChild::Option(
        SelectOption::new("pro", Value::C, "专业版").aria_label("专业方案"),
    )]);
    assert_eq!(
        trigger_accessibility(catalog.first(), &placeholder),
        TriggerAccessibility {
            value: Some("专业方案".into()),
            placeholder: None,
        }
    );
}

#[test]
fn removed_active_prefers_same_position_then_previous() {
    let previous = vec![
        OptionSnapshot {
            id: "a".into(),
            disabled: false,
            accessible_name: "a".into(),
        },
        OptionSnapshot {
            id: "b".into(),
            disabled: false,
            accessible_name: "b".into(),
        },
        OptionSnapshot {
            id: "c".into(),
            disabled: false,
            accessible_name: "c".into(),
        },
    ];
    let next = vec![
        OptionSnapshot {
            id: "a".into(),
            disabled: false,
            accessible_name: "a".into(),
        },
        OptionSnapshot {
            id: "c".into(),
            disabled: false,
            accessible_name: "c".into(),
        },
    ];
    assert_eq!(
        reconciled_active_id(&previous, &next, Some(&ElementId::from("b"))),
        Some(ElementId::from("c"))
    );
}

#[test]
fn disabled_accessibility_wrapper_writes_accesskit_state() {
    let element = div()
        .id("disabled-test")
        .role(Role::ComboBox)
        .aria_expanded(false);
    let wrapper = DisabledA11y::new(element, true, None, None);
    let mut node = gpui::accesskit::Node::new(Role::ComboBox);
    wrapper.write_a11y_info(&mut node);
    assert!(node.is_disabled());
    assert_eq!(node.is_expanded(), Some(false));
}

#[test]
fn accessibility_wrapper_associates_an_open_trigger_with_its_popup() {
    let popup_id = gpui::accesskit::NodeId(42);
    let wrapper = DisabledA11y::new(
        div()
            .id("trigger-with-popup")
            .role(Role::ComboBox)
            .aria_expanded(true),
        false,
        None,
        None,
    )
    .controls(popup_id);
    let mut node = gpui::accesskit::Node::new(Role::ComboBox);

    wrapper.write_a11y_info(&mut node);

    assert_eq!(node.controls(), &[popup_id]);
}

#[gpui::test]
fn popup_node_id_is_available_from_the_real_nested_global_id_on_the_first_frame(
    cx: &mut gpui::TestAppContext,
) {
    let (_, cx) = cx.add_window_view(|_, _| EmptyView);
    cx.update(|window, _| {
        let expected = window.with_global_id(ElementId::from("plans"), |_, window| {
            window.with_global_id(ElementId::from("vektra-select-popup"), |global_id, _| {
                accesskit_node_id(global_id)
            })
        });
        let popup_id = select_popup_node_id(ElementId::from("plans"), window);
        assert_eq!(popup_id, expected);

        let wrapper = DisabledA11y::new(
            div().id("plans").role(Role::ComboBox).aria_expanded(true),
            false,
            None,
            None,
        )
        .controls(popup_id);
        let mut node = gpui::accesskit::Node::new(Role::ComboBox);
        wrapper.write_a11y_info(&mut node);
        assert_eq!(node.controls(), &[popup_id]);
    });
}

#[test]
fn locked_gpui_accessibility_primitives_preserve_select_semantics() {
    let trigger = DisabledA11y::new(
        div()
            .id("a11y-trigger")
            .role(Role::ComboBox)
            .aria_label("方案")
            .aria_description("选择方案")
            .aria_value("专业版")
            .aria_expanded(true),
        false,
        None,
        None,
    );
    assert_eq!(trigger.a11y_role(), Some(Role::ComboBox));
    let mut trigger_node = gpui::accesskit::Node::new(Role::ComboBox);
    trigger.write_a11y_info(&mut trigger_node);
    assert_eq!(trigger_node.label(), Some("方案"));
    assert_eq!(trigger_node.description(), Some("选择方案"));
    assert_eq!(trigger_node.value(), Some("专业版"));
    assert_eq!(trigger_node.is_expanded(), Some(true));
    assert!(!trigger_node.is_disabled());

    let option = DisabledA11y::new(
        div()
            .id("a11y-option")
            .role(Role::ListBoxOption)
            .aria_label("团队版")
            .aria_description("最多十人")
            .aria_selected(true),
        true,
        None,
        None,
    );
    assert_eq!(option.a11y_role(), Some(Role::ListBoxOption));
    let mut option_node = gpui::accesskit::Node::new(Role::ListBoxOption);
    option.write_a11y_info(&mut option_node);
    assert_eq!(option_node.label(), Some("团队版"));
    assert_eq!(option_node.description(), Some("最多十人"));
    assert_eq!(option_node.is_selected(), Some(true));
    assert!(option_node.is_disabled());

    for role in [
        Role::ListBox,
        Role::Group,
        Role::Label,
        Role::Status,
        Role::Alert,
    ] {
        let element = div().id(format!("a11y-{role:?}")).role(role);
        assert_eq!(Element::a11y_role(&element), Some(role));
    }
}
