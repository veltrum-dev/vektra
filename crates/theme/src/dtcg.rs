//! DTCG 2025.10 的第一阶段子集解析器。
//!
//! 本模块支持 Vektra Button 当前需要的 token/group 结构、`$type` 继承、
//! `$value`、`$description`、`$extensions` 保留以及 `{path.to.token}` 别名。

use crate::error::ThemeError;
use gpui::{Hsla, Pixels, hsla, px};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};

/// DTCG token 类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// DTCG `color`。
    Color,
    /// DTCG `dimension`。
    Dimension,
    /// DTCG `number`。
    Number,
    /// DTCG `fontWeight`。
    FontWeight,
    /// DTCG `shadow`。
    Shadow,
}

impl TokenType {
    /// 从 DTCG `$type` 字符串解析第一阶段支持的类型。
    pub fn parse(path: &str, raw: &str) -> Result<Self, ThemeError> {
        match raw {
            "color" => Ok(Self::Color),
            "dimension" => Ok(Self::Dimension),
            "number" => Ok(Self::Number),
            "fontWeight" => Ok(Self::FontWeight),
            "shadow" => Ok(Self::Shadow),
            other => Err(ThemeError::InvalidValue {
                path: path.to_owned(),
                message: format!("暂不支持 DTCG 类型 `{other}`"),
            }),
        }
    }

    /// 返回 DTCG 类型名。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Dimension => "dimension",
            Self::Number => "number",
            Self::FontWeight => "fontWeight",
            Self::Shadow => "shadow",
        }
    }
}

/// 解析后的 DTCG token 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// 结构化颜色值。
    Color(ColorValue),
    /// 像素尺寸。
    Dimension(DimensionValue),
    /// 普通数值。
    Number(f64),
    /// 字体粗细。
    FontWeight(FontWeightValue),
    /// 阴影值。
    Shadow(Vec<ShadowValue>),
}

impl TokenValue {
    /// 返回值的 DTCG 类型。
    pub const fn token_type(&self) -> TokenType {
        match self {
            Self::Color(_) => TokenType::Color,
            Self::Dimension(_) => TokenType::Dimension,
            Self::Number(_) => TokenType::Number,
            Self::FontWeight(_) => TokenType::FontWeight,
            Self::Shadow(_) => TokenType::Shadow,
        }
    }
}

/// DTCG 结构化颜色值。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorValue {
    /// sRGB 红色通道，范围 0..=1。
    pub r: f32,
    /// sRGB 绿色通道，范围 0..=1。
    pub g: f32,
    /// sRGB 蓝色通道，范围 0..=1。
    pub b: f32,
    /// Alpha 通道，范围 0..=1。
    pub alpha: f32,
}

impl ColorValue {
    /// 转为锁定 GPUI 版本使用的 `Hsla`。
    pub fn to_hsla(self) -> Hsla {
        let max = self.r.max(self.g).max(self.b);
        let min = self.r.min(self.g).min(self.b);
        let lightness = (max + min) / 2.0;

        if (max - min).abs() <= f32::EPSILON {
            return hsla(0.0, 0.0, lightness, self.alpha);
        }

        let delta = max - min;
        let saturation = if lightness > 0.5 {
            delta / (2.0 - max - min)
        } else {
            delta / (max + min)
        };
        let hue = if (max - self.r).abs() <= f32::EPSILON {
            ((self.g - self.b) / delta + if self.g < self.b { 6.0 } else { 0.0 }) / 6.0
        } else if (max - self.g).abs() <= f32::EPSILON {
            ((self.b - self.r) / delta + 2.0) / 6.0
        } else {
            ((self.r - self.g) / delta + 4.0) / 6.0
        };

        hsla(hue, saturation, lightness, self.alpha)
    }
}

/// DTCG `dimension` 值。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DimensionValue {
    /// 数值。
    pub value: f32,
    /// 单位；第一阶段只转换 `px`。
    pub unit: DimensionUnit,
}

impl DimensionValue {
    /// 转为 GPUI 逻辑像素。
    pub fn to_pixels(self) -> Pixels {
        match self.unit {
            DimensionUnit::Px => px(self.value),
        }
    }
}

/// 第一阶段支持的尺寸单位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionUnit {
    /// CSS/设计 token 中的像素单位。
    Px,
}

/// DTCG `fontWeight` 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontWeightValue {
    /// 数字字体粗细。
    Number(f64),
    /// 命名字体粗细。
    Name(String),
}

/// DTCG `shadow` 值。
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowValue {
    /// 阴影颜色。
    pub color: ColorValue,
    /// X 偏移。
    pub offset_x: DimensionValue,
    /// Y 偏移。
    pub offset_y: DimensionValue,
    /// 模糊半径。
    pub blur: DimensionValue,
    /// 扩散半径。
    pub spread: DimensionValue,
    /// 是否为内阴影。
    pub inset: bool,
}

#[derive(Debug, Clone)]
struct RawToken {
    token_type: TokenType,
    value: Value,
    description: Option<String>,
    extensions: Option<Value>,
}

/// 已解析的 token。
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedToken {
    /// 完整 token 路径。
    pub path: String,
    /// DTCG 值。
    pub value: TokenValue,
    /// DTCG `$description`。
    pub description: Option<String>,
    /// DTCG `$extensions`。
    pub extensions: Option<Value>,
}

/// 已解析 token 集。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResolvedTokens {
    tokens: BTreeMap<String, ResolvedToken>,
}

impl ResolvedTokens {
    /// 按完整路径读取 token。
    pub fn get(&self, path: &str) -> Option<&ResolvedToken> {
        self.tokens.get(path)
    }

    /// 按完整路径读取颜色 token。
    pub fn color(&self, path: &str) -> Result<ColorValue, ThemeError> {
        match &self.required(path)?.value {
            TokenValue::Color(value) => Ok(*value),
            value => Err(type_error(path, TokenType::Color, value.token_type())),
        }
    }

    /// 按完整路径读取尺寸 token。
    pub fn dimension(&self, path: &str) -> Result<DimensionValue, ThemeError> {
        match &self.required(path)?.value {
            TokenValue::Dimension(value) => Ok(*value),
            value => Err(type_error(path, TokenType::Dimension, value.token_type())),
        }
    }

    /// 读取必须存在的 token。
    pub fn required(&self, path: &str) -> Result<&ResolvedToken, ThemeError> {
        self.tokens
            .get(path)
            .ok_or_else(|| ThemeError::MissingProfileToken {
                path: path.to_owned(),
            })
    }

    /// 返回 token 数量。
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// 判断 token 集是否为空。
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// 解析并合并多个 DTCG token set。
pub fn parse_token_sets(sources: &[&str]) -> Result<ResolvedTokens, ThemeError> {
    let mut raw = HashMap::new();
    for source in sources {
        let value: Value =
            serde_json::from_str(source).map_err(|err| ThemeError::Json(err.to_string()))?;
        flatten_group(&value, None, &mut Vec::new(), &mut raw)?;
    }

    let mut resolved = BTreeMap::new();
    let paths = raw.keys().cloned().collect::<Vec<_>>();
    for path in paths {
        let token = resolve_token(
            &path,
            &raw,
            &mut Vec::new(),
            &mut HashSet::new(),
            &mut resolved,
        )?;
        resolved.insert(path, token);
    }

    Ok(ResolvedTokens { tokens: resolved })
}

fn flatten_group(
    value: &Value,
    inherited_type: Option<TokenType>,
    path: &mut Vec<String>,
    raw: &mut HashMap<String, RawToken>,
) -> Result<(), ThemeError> {
    let object = value.as_object().ok_or_else(|| ThemeError::InvalidValue {
        path: path.join("."),
        message: "group 必须是 JSON object".to_owned(),
    })?;

    let local_type = object
        .get("$type")
        .and_then(Value::as_str)
        .map(|raw_type| TokenType::parse(&path.join("."), raw_type))
        .transpose()?
        .or(inherited_type);

    if let Some(token_value) = object.get("$value") {
        let full_path = path.join(".");
        let token_type = local_type.ok_or_else(|| ThemeError::MissingType {
            path: full_path.clone(),
        })?;
        raw.insert(
            full_path,
            RawToken {
                token_type,
                value: token_value.clone(),
                description: object
                    .get("$description")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
                extensions: object.get("$extensions").cloned(),
            },
        );
        return Ok(());
    }

    for (key, child) in object {
        if key.starts_with('$') {
            continue;
        }
        path.push(key.clone());
        flatten_group(child, local_type, path, raw)?;
        path.pop();
    }

    Ok(())
}

fn resolve_token(
    path: &str,
    raw: &HashMap<String, RawToken>,
    stack: &mut Vec<String>,
    visiting: &mut HashSet<String>,
    resolved: &mut BTreeMap<String, ResolvedToken>,
) -> Result<ResolvedToken, ThemeError> {
    if let Some(token) = resolved.get(path) {
        return Ok(token.clone());
    }

    if !visiting.insert(path.to_owned()) {
        stack.push(path.to_owned());
        return Err(ThemeError::CircularReference {
            path: path.to_owned(),
            cycle: stack.join(" -> "),
        });
    }
    stack.push(path.to_owned());

    let token = raw.get(path).ok_or_else(|| ThemeError::MissingReference {
        path: path.to_owned(),
        reference: path.to_owned(),
    })?;

    let value = if let Some(reference) = alias_reference(&token.value) {
        let target = raw
            .get(reference)
            .ok_or_else(|| ThemeError::MissingReference {
                path: path.to_owned(),
                reference: reference.to_owned(),
            })?;
        if token.token_type != target.token_type {
            return Err(ThemeError::TypeMismatch {
                path: path.to_owned(),
                expected: token.token_type.as_str().to_owned(),
                found: target.token_type.as_str().to_owned(),
            });
        }
        resolve_token(reference, raw, stack, visiting, resolved)?
            .value
            .clone()
    } else {
        parse_value(path, token.token_type, &token.value)?
    };

    stack.pop();
    visiting.remove(path);

    let resolved_token = ResolvedToken {
        path: path.to_owned(),
        value,
        description: token.description.clone(),
        extensions: token.extensions.clone(),
    };
    resolved.insert(path.to_owned(), resolved_token.clone());
    Ok(resolved_token)
}

fn parse_value(path: &str, token_type: TokenType, value: &Value) -> Result<TokenValue, ThemeError> {
    match token_type {
        TokenType::Color => parse_color(path, value).map(TokenValue::Color),
        TokenType::Dimension => parse_dimension(path, value).map(TokenValue::Dimension),
        TokenType::Number => value
            .as_f64()
            .map(TokenValue::Number)
            .ok_or_else(|| invalid(path, "number token 必须是 JSON number")),
        TokenType::FontWeight => parse_font_weight(path, value).map(TokenValue::FontWeight),
        TokenType::Shadow => parse_shadow(path, value).map(TokenValue::Shadow),
    }
}

fn parse_color(path: &str, value: &Value) -> Result<ColorValue, ThemeError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(path, "color token 必须是结构化 object"))?;
    let color_space = object
        .get("colorSpace")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(path, "color 缺少 colorSpace"))?;
    if color_space != "srgb" {
        return Err(invalid(path, "第一阶段只支持 srgb colorSpace"));
    }
    let components = object
        .get("components")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(path, "color 缺少 components"))?;
    if components.len() != 3 {
        return Err(invalid(path, "srgb components 必须包含 3 个分量"));
    }
    let component = |index: usize| -> Result<f32, ThemeError> {
        components[index]
            .as_f64()
            .map(|value| value.clamp(0.0, 1.0) as f32)
            .ok_or_else(|| invalid(path, "srgb component 必须是 number"))
    };
    let alpha = object.get("alpha").and_then(Value::as_f64).unwrap_or(1.0);
    Ok(ColorValue {
        r: component(0)?,
        g: component(1)?,
        b: component(2)?,
        alpha: alpha.clamp(0.0, 1.0) as f32,
    })
}

fn parse_dimension(path: &str, value: &Value) -> Result<DimensionValue, ThemeError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(path, "dimension token 必须是 object"))?;
    let numeric = object
        .get("value")
        .and_then(Value::as_f64)
        .ok_or_else(|| invalid(path, "dimension 缺少 number value"))?;
    let unit = match object.get("unit").and_then(Value::as_str) {
        Some("px") => DimensionUnit::Px,
        Some(_) => return Err(invalid(path, "第一阶段只支持 px dimension")),
        None => return Err(invalid(path, "dimension 缺少 unit")),
    };
    Ok(DimensionValue {
        value: numeric as f32,
        unit,
    })
}

fn parse_font_weight(path: &str, value: &Value) -> Result<FontWeightValue, ThemeError> {
    if let Some(number) = value.as_f64() {
        return Ok(FontWeightValue::Number(number));
    }
    value
        .as_str()
        .map(|value| FontWeightValue::Name(value.to_owned()))
        .ok_or_else(|| invalid(path, "fontWeight 必须是 number 或 string"))
}

fn parse_shadow(path: &str, value: &Value) -> Result<Vec<ShadowValue>, ThemeError> {
    if let Some(array) = value.as_array() {
        return array
            .iter()
            .enumerate()
            .map(|(index, item)| parse_shadow_item(&format!("{path}[{index}]"), item))
            .collect();
    }
    parse_shadow_item(path, value).map(|shadow| vec![shadow])
}

fn parse_shadow_item(path: &str, value: &Value) -> Result<ShadowValue, ThemeError> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid(path, "shadow item 必须是 object"))?;
    let field = |name: &str| {
        object
            .get(name)
            .ok_or_else(|| invalid(path, &format!("shadow 缺少 {name}")))
    };
    Ok(ShadowValue {
        color: parse_color(path, field("color")?)?,
        offset_x: parse_dimension(path, field("offsetX")?)?,
        offset_y: parse_dimension(path, field("offsetY")?)?,
        blur: parse_dimension(path, field("blur")?)?,
        spread: object
            .get("spread")
            .map(|value| parse_dimension(path, value))
            .transpose()?
            .unwrap_or(DimensionValue {
                value: 0.0,
                unit: DimensionUnit::Px,
            }),
        inset: object
            .get("inset")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn alias_reference(value: &Value) -> Option<&str> {
    let value = value.as_str()?;
    value.strip_prefix('{')?.strip_suffix('}')
}

fn type_error(path: &str, expected: TokenType, found: TokenType) -> ThemeError {
    ThemeError::TypeMismatch {
        path: path.to_owned(),
        expected: expected.as_str().to_owned(),
        found: found.as_str().to_owned(),
    }
}

fn invalid(path: &str, message: &str) -> ThemeError {
    ThemeError::InvalidValue {
        path: path.to_owned(),
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_type_inherits_to_children() {
        let tokens = parse_token_sets(&[
            r#"{"a":{"$type":"dimension","b":{"$value":{"value":8,"unit":"px"}}}}"#,
        ])
        .expect("group type should parse");
        assert_eq!(tokens.dimension("a.b").unwrap().value, 8.0);
    }

    #[test]
    fn aliases_resolve_across_sets() {
        let tokens = parse_token_sets(&[
            r#"{"a":{"$type":"color","base":{"$value":{"colorSpace":"srgb","components":[1,0,0],"alpha":1}}}}"#,
            r#"{"b":{"$type":"color","alias":{"$value":"{a.base}"}}}"#,
        ])
        .expect("alias should resolve");
        assert_eq!(tokens.color("b.alias").unwrap().r, 1.0);
    }

    #[test]
    fn missing_alias_reports_path() {
        let error =
            parse_token_sets(&[r#"{"a":{"$type":"color","b":{"$value":"{nope}"}}}"#]).unwrap_err();
        assert!(error.to_string().contains("a.b"));
    }

    #[test]
    fn circular_alias_reports_cycle() {
        let error = parse_token_sets(&[
            r#"{"a":{"$type":"color","b":{"$value":"{a.c}"},"c":{"$value":"{a.b}"}}}"#,
        ])
        .unwrap_err();
        assert!(matches!(error, ThemeError::CircularReference { .. }));
    }

    #[test]
    fn type_mismatch_reports_path() {
        let error = parse_token_sets(&[
            r#"{"a":{"$type":"dimension","b":{"$value":{"value":1,"unit":"px"}}}}"#,
            r#"{"c":{"$type":"color","d":{"$value":"{a.b}"}}}"#,
        ])
        .unwrap_err();
        assert!(matches!(error, ThemeError::TypeMismatch { .. }));
        assert!(error.to_string().contains("c.d"));
    }

    #[test]
    fn srgb_converts_to_hsla() {
        let color = ColorValue {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            alpha: 1.0,
        }
        .to_hsla();
        assert!((color.h - 0.0).abs() < 0.001);
        assert!((color.s - 1.0).abs() < 0.001);
        assert!((color.l - 0.5).abs() < 0.001);
    }
}
