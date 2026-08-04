use super::*;

#[test]
fn checkbox_demo_preserves_each_examples_initial_state() {
    let demo = CheckboxDemo::new();

    assert_eq!(demo.terms, CheckboxState::unchecked());
    assert_eq!(demo.mixed, CheckboxState::indeterminate());
    assert_eq!(demo.no_label, CheckboxState::unchecked());
    assert_eq!(demo.xs, CheckboxState::unchecked());
    assert_eq!(demo.sm, CheckboxState::checked());
    assert_eq!(demo.md, CheckboxState::checked());
    assert_eq!(demo.lg, CheckboxState::indeterminate());
    assert_eq!(demo.custom_unchecked, CheckboxState::unchecked());
    assert_eq!(demo.custom_checked, CheckboxState::checked());
    assert_eq!(demo.custom_mixed, CheckboxState::indeterminate());
    assert_eq!(demo.favorite, CheckboxState::unchecked());
    assert_eq!(demo.batch_product, CheckboxState::checked());
    assert_eq!(demo.batch_billing, CheckboxState::unchecked());
    assert_eq!(demo.batch_security, CheckboxState::checked());
    assert_eq!(demo.batch_selected_count(), 2);
    assert!(!demo.batch_all_selected());
    assert!(demo.batch_indeterminate());
    assert_eq!(demo.global_size, CheckboxState::unchecked());
    assert_eq!(demo.explicit_size, CheckboxState::unchecked());
}

#[test]
fn changing_one_checkbox_does_not_change_unrelated_examples() {
    let mut demo = CheckboxDemo::new();
    let mixed = demo.mixed;
    let no_label = demo.no_label;
    let sizes = [demo.xs, demo.sm, demo.md, demo.lg];
    let custom_icons = [
        demo.custom_unchecked,
        demo.custom_checked,
        demo.custom_mixed,
    ];
    let size_overrides = [demo.global_size, demo.explicit_size];
    let favorite = demo.favorite;
    let batch_items = [demo.batch_product, demo.batch_billing, demo.batch_security];

    demo.terms.apply_change(true);

    assert_eq!(demo.terms, CheckboxState::checked());
    assert_eq!(demo.mixed, mixed);
    assert_eq!(demo.no_label, no_label);
    assert_eq!([demo.xs, demo.sm, demo.md, demo.lg], sizes);
    assert_eq!(
        [
            demo.custom_unchecked,
            demo.custom_checked,
            demo.custom_mixed,
        ],
        custom_icons
    );
    assert_eq!([demo.global_size, demo.explicit_size], size_overrides);
    assert_eq!(demo.favorite, favorite);
    assert_eq!(
        [demo.batch_product, demo.batch_billing, demo.batch_security,],
        batch_items
    );
}

#[test]
fn mixed_checkbox_clears_indeterminate_then_toggles_normally() {
    let mut demo = CheckboxDemo::new();
    let terms = demo.terms;

    demo.mixed.apply_change(true);
    assert_eq!(demo.mixed, CheckboxState::checked());
    assert_eq!(demo.terms, terms);

    demo.mixed.apply_change(false);
    assert_eq!(demo.mixed, CheckboxState::unchecked());
}

#[test]
fn batch_selection_tracks_all_mixed_and_inverted_states() {
    let mut demo = CheckboxDemo::new();

    assert_eq!(demo.batch_selected_count(), 2);
    assert!(!demo.batch_all_selected());
    assert!(demo.batch_indeterminate());

    demo.set_batch_checked(true);
    assert_eq!(demo.batch_selected_count(), 3);
    assert!(demo.batch_all_selected());
    assert!(!demo.batch_indeterminate());

    demo.invert_batch_selection();
    assert_eq!(demo.batch_selected_count(), 0);
    assert!(!demo.batch_all_selected());
    assert!(!demo.batch_indeterminate());

    demo.batch_billing.apply_change(true);
    assert_eq!(demo.batch_selected_count(), 1);
    assert!(!demo.batch_all_selected());
    assert!(demo.batch_indeterminate());
}
