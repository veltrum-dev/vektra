use super::*;

#[test]
fn checkbox_example_embeds_both_heart_states() {
    for path in ["icons/heart.svg", "icons/heart-filled.svg"] {
        let bytes = CheckboxExampleAssets.load(path).unwrap().unwrap();
        let svg = std::str::from_utf8(bytes.as_ref()).unwrap();

        assert!(svg.contains("viewBox=\"0 0 16 16\""));
        assert!(svg.contains("currentColor"));
    }
}
