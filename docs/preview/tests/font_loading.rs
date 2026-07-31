use std::{fs, path::Path};
use ttf_parser::{Face, Tag, name_id};

const FONT_PATH: &str = "assets/fonts/noto-sans-sc/NotoSansSC-VF.ttf";
const FONT_FAMILY: &str = "Noto Sans SC";
const REPRESENTATIVE_CHARS: &[char] = &[
    '中', '文', '按', '钮', '点', '击', '次', '数', '增', '加', '禁', '用', '预', '览',
];

#[test]
fn noto_sans_sc_font_file_exists_and_is_not_empty() {
    let metadata = fs::metadata(font_path()).expect("Noto Sans SC 字体文件必须存在");
    assert!(metadata.len() > 0, "Noto Sans SC 字体文件不能为空");
}

#[test]
fn noto_sans_sc_font_metadata_matches_preview_configuration() {
    let bytes = fs::read(font_path()).expect("Noto Sans SC 字体文件必须可读取");
    let face = Face::parse(&bytes, 0).expect("Noto Sans SC 字体文件必须可被 ttf-parser 解析");

    let families = face
        .names()
        .into_iter()
        .filter(|name| name.name_id == name_id::FAMILY)
        .filter_map(|name| name.to_string())
        .collect::<Vec<_>>();
    assert!(
        families.iter().any(|family| family == FONT_FAMILY),
        "字体真实 family 必须包含 {FONT_FAMILY:?}，实际为 {families:?}"
    );

    assert!(face.is_regular(), "字体必须提供 Regular 能力");
    assert!(face.is_variable(), "预览使用的 Noto Sans SC 应为可变 TTF");

    let weight_axis = face
        .tables()
        .fvar
        .and_then(|fvar| {
            fvar.axes
                .into_iter()
                .find(|axis| axis.tag == Tag::from_bytes(b"wght"))
        })
        .expect("可变字体必须包含 wght 轴");
    assert!(
        weight_axis.min_value <= 400.0 && weight_axis.max_value >= 600.0,
        "wght 轴必须覆盖 Regular(400) 和 SemiBold(600)，实际范围为 {}..={}",
        weight_axis.min_value,
        weight_axis.max_value
    );
}

#[test]
fn noto_sans_sc_font_covers_representative_chinese_characters() {
    let bytes = fs::read(font_path()).expect("Noto Sans SC 字体文件必须可读取");
    let face = Face::parse(&bytes, 0).expect("Noto Sans SC 字体文件必须可被 ttf-parser 解析");

    for ch in REPRESENTATIVE_CHARS {
        assert!(
            face.glyph_index(*ch).is_some(),
            "Noto Sans SC 必须覆盖代表性中文字符 {ch:?}"
        );
    }
}

fn font_path() -> &'static Path {
    Path::new(FONT_PATH)
}
