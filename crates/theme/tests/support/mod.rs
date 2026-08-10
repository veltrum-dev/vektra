use gpui::Hsla;

pub fn assert_contrast_at_least(label: &str, foreground: Hsla, background: Hsla, minimum: f32) {
    let ratio = contrast_ratio(foreground, background);
    assert!(
        ratio + f32::EPSILON >= minimum,
        "{label} 对比度应至少为 {minimum:.2}:1，实际为 {ratio:.2}:1"
    );
}

pub fn assert_neutral(label: &str, color: Hsla) {
    let color = color.to_rgb();
    let minimum = color.r.min(color.g).min(color.b);
    let maximum = color.r.max(color.g).max(color.b);
    assert!(
        maximum - minimum <= 0.04,
        "{label} 应保持中性灰阶，实际 RGB 为 ({:.3}, {:.3}, {:.3})",
        color.r,
        color.g,
        color.b
    );
}

pub fn contrast_ratio(first: Hsla, second: Hsla) -> f32 {
    assert!(
        (first.a - 1.).abs() <= f32::EPSILON && (second.a - 1.).abs() <= f32::EPSILON,
        "对比度测试只接受不透明颜色"
    );
    let first = relative_luminance(first);
    let second = relative_luminance(second);
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

fn relative_luminance(color: Hsla) -> f32 {
    let color = color.to_rgb();
    0.2126 * linear_channel(color.r)
        + 0.7152 * linear_channel(color.g)
        + 0.0722 * linear_channel(color.b)
}

fn linear_channel(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}
