use vektra_theme::{
    ThemeError,
    dtcg::{TokenValue, parse_token_sets},
};

#[test]
fn group_type_is_order_independent_and_escaped_keys_are_borrowed_safely() {
    let tokens =
        parse_token_sets(&[r#"{"group":{"token\u002done":{"$value":1},"$type":"number"}}"#])
            .expect("位于 child 之后的 group type 仍应生效");

    assert_eq!(
        tokens.get("group.token-one").unwrap().value,
        TokenValue::Number(1.0)
    );
}

#[test]
fn later_direct_and_alias_tokens_preserve_overlay_semantics() {
    let tokens = parse_token_sets(&[
        r#"{"value":{"$type":"number","base":{"$value":1},"alias":{"$value":"{value.base}"}}}"#,
        r#"{"value":{"$type":"number","base":{"$value":2}}}"#,
    ])
    .expect("后置 overlay 应覆盖 alias 目标");

    assert_eq!(
        tokens.get("value.base").unwrap().value,
        TokenValue::Number(2.0)
    );
    assert_eq!(
        tokens.get("value.alias").unwrap().value,
        TokenValue::Number(2.0)
    );
}

#[test]
fn duplicate_token_paths_keep_the_last_json_value() {
    let tokens = parse_token_sets(&[
        r#"{"value":{"$type":"number","token":{"$value":1},"token":{"$value":4}}}"#,
    ])
    .expect("重复 token path 应保持 serde_json 的后值覆盖语义");

    assert_eq!(
        tokens.get("value.token").unwrap().value,
        TokenValue::Number(4.0)
    );
}

#[test]
fn overridden_invalid_alias_is_not_resolved_but_active_cycle_still_fails() {
    let tokens = parse_token_sets(&[
        r#"{"value":{"$type":"number","token":{"$value":"{missing}"}}}"#,
        r#"{"value":{"$type":"number","token":{"$value":3}}}"#,
    ])
    .expect("已被覆盖的 alias 不应进入解析图");
    assert_eq!(
        tokens.get("value.token").unwrap().value,
        TokenValue::Number(3.0)
    );

    let error = parse_token_sets(&[
        r#"{"value":{"$type":"number","a":{"$value":"{value.b}"},"b":{"$value":"{value.a}"}}}"#,
    ])
    .unwrap_err();
    assert!(matches!(error, ThemeError::CircularReference { .. }));
}

#[test]
fn compact_and_large_source_paths_have_identical_semantics() {
    let compact =
        r#"{"value":{"$type":"number","base":{"$value":2},"alias":{"$value":"{value.base}"}}}"#;
    let padding = "x".repeat(300 * 1024);
    let large = format!(
        r#"{{"$padding":"{padding}","value":{{"$type":"number","base":{{"$value":2}},"alias":{{"$value":"{{value.base}}"}}}}}}"#
    );

    assert_eq!(
        parse_token_sets(&[compact]).unwrap(),
        parse_token_sets(&[&large]).unwrap()
    );
}
