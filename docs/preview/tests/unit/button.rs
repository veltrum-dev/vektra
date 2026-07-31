use super::*;

#[test]
fn record_click_updates_count_and_recent_label() {
    let mut demo = ButtonDemo::new(PreviewLang::ZhCn);

    demo.record_click("保存".into());
    demo.record_click("下一步".into());

    assert_eq!(demo.clicks(), 2);
    assert_eq!(demo.last_clicked().as_ref(), "下一步");
}

#[test]
fn initial_recent_label_uses_preview_language() {
    assert_eq!(
        ButtonDemo::new(PreviewLang::ZhCn).last_clicked().as_ref(),
        "暂无"
    );
    assert_eq!(
        ButtonDemo::new(PreviewLang::EnUs).last_clicked().as_ref(),
        "None"
    );
}

#[test]
fn controlled_activity_and_selected_state_changes_are_host_owned() {
    let mut demo = ButtonDemo::new(PreviewLang::ZhCn);
    assert!(!demo.selected);
    assert!(!demo.loading);
    assert_eq!(demo.progress, 0.25);

    demo.toggle_selected();
    demo.toggle_loading();
    demo.advance_progress();
    assert!(demo.selected);
    assert!(demo.loading);
    assert_eq!(demo.progress, 0.5);

    demo.progress = 1.;
    demo.advance_progress();
    assert_eq!(demo.progress, 0.);
}
