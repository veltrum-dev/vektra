use super::*;
use gpui::{Element, ElementId, InteractiveElement, Role, div};

#[derive(Clone, PartialEq)]
enum Value {
    A,
    B,
    C,
}

fn option(id: &'static str, value: Value, disabled: bool) -> SelectOption<Value> {
    SelectOption::new(id, value, id).disabled(disabled)
}

#[test]
fn duplicate_ids_and_values_use_first_canonical_option() {
    let children = vec![
        SelectChild::Option(option("a", Value::A, false)),
        SelectChild::Option(option("a", Value::B, false)),
        SelectChild::Option(option("c", Value::A, false)),
        SelectChild::Option(option("d", Value::C, false)),
    ];
    let flat = flat_options(&children);
    assert_eq!(
        flat.iter().map(|entry| entry.canonical).collect::<Vec<_>>(),
        [true, false, false, true]
    );
}

#[test]
fn removed_active_prefers_same_position_then_previous() {
    let previous = vec![
        OptionSnapshot {
            id: "a".into(),
            disabled: false,
        },
        OptionSnapshot {
            id: "b".into(),
            disabled: false,
        },
        OptionSnapshot {
            id: "c".into(),
            disabled: false,
        },
    ];
    let next = vec![
        OptionSnapshot {
            id: "a".into(),
            disabled: false,
        },
        OptionSnapshot {
            id: "c".into(),
            disabled: false,
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
