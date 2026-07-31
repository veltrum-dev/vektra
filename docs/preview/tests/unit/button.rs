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
