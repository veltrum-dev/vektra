use super::{Radio, navigation_target, radio_group_role, radio_role, toggled_state};
use gpui::{Role, Toggled};

#[test]
fn navigation_wraps_and_supports_home_end() {
    assert_eq!(navigation_target("left", 0, 3), Some(2));
    assert_eq!(navigation_target("up", 1, 3), Some(0));
    assert_eq!(navigation_target("right", 2, 3), Some(0));
    assert_eq!(navigation_target("down", 0, 3), Some(1));
    assert_eq!(navigation_target("home", 2, 3), Some(0));
    assert_eq!(navigation_target("end", 0, 3), Some(2));
    assert_eq!(navigation_target("enter", 0, 3), None);
    assert_eq!(navigation_target("right", 0, 0), None);
}

#[test]
fn accessibility_roles_names_descriptions_and_selected_state_are_mapped() {
    let radio = Radio::new("plan", 1)
        .label("专业版")
        .description("适合团队")
        .aria_label("专业方案")
        .aria_description("每月计费");

    assert_eq!(radio_group_role(), Role::RadioGroup);
    assert_eq!(radio_role(), Role::RadioButton);
    assert_eq!(toggled_state(false), Toggled::False);
    assert_eq!(toggled_state(true), Toggled::True);
    assert_eq!(radio.accessible_label().unwrap().as_ref(), "专业方案");
    assert_eq!(radio.accessible_description().unwrap().as_ref(), "每月计费");
}
