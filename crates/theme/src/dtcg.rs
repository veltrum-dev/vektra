//! DTCG 2025.10 的第一阶段子集解析器。
//!
//! 本模块支持 Vektra Button 当前需要的 token/group 结构、`$type` 继承、
//! `$value`、`$description`、`$extensions` 保留以及 `{path.to.token}` 别名。

use crate::error::ThemeError;
use gpui::{Hsla, Pixels, hsla, px};
use hashbrown::{DefaultHashBuilder, HashTable};
use serde::{
    Deserialize,
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
};
use serde_json::{Value, value::RawValue};
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    cmp::Reverse,
    collections::HashMap,
    fmt,
    hash::BuildHasher,
};

const SMALL_THEME_SOURCE_BYTES: usize = 256 * 1024;

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

#[derive(Debug, Clone, PartialEq)]
struct StoredToken {
    sequence: u64,
    token: ResolvedToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AliasState {
    Unresolved,
    Visiting,
    Resolved,
}

#[derive(Debug)]
struct PendingAlias {
    sequence: u64,
    path: String,
    token_type: TokenType,
    reference: String,
    description: Option<String>,
    extensions: Option<Value>,
    state: Cell<AliasState>,
    resolved: RefCell<Option<TokenValue>>,
    active: Cell<bool>,
}

struct ParsedTokens {
    direct: Vec<StoredToken>,
    aliases: Vec<PendingAlias>,
    next_sequence: u64,
}

impl ParsedTokens {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            direct: Vec::with_capacity(capacity),
            aliases: Vec::new(),
            next_sequence: 0,
        }
    }

    fn next_sequence(&mut self) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.wrapping_add(1);
        sequence
    }
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
#[derive(Debug, Clone, Default)]
pub struct ResolvedTokens {
    tokens: Vec<StoredToken>,
    index: HashTable<usize>,
    hash_builder: DefaultHashBuilder,
}

impl PartialEq for ResolvedTokens {
    fn eq(&self, other: &Self) -> bool {
        self.tokens == other.tokens
    }
}

impl ResolvedTokens {
    /// 按完整路径读取 token。
    pub fn get(&self, path: &str) -> Option<&ResolvedToken> {
        let hash = self.hash_builder.hash_one(path);
        self.index
            .find(hash, |index| self.tokens[*index].token.path == path)
            .map(|index| &self.tokens[*index].token)
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
        self.get(path)
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
    let capacity = sources
        .iter()
        .map(|source| {
            source
                .matches("\"$value\"")
                .count()
                .min(source.len() / 24 + 1)
        })
        .sum();
    let mut parsed = ParsedTokens::with_capacity(capacity);
    let mut path = String::with_capacity(128);
    let source_bytes = sources
        .iter()
        .fold(0usize, |total, source| total.saturating_add(source.len()));
    if source_bytes <= SMALL_THEME_SOURCE_BYTES {
        for source in sources {
            let value =
                serde_json::from_str(source).map_err(|err| ThemeError::Json(err.to_string()))?;
            flatten_value_group(value, None, &mut path, &mut parsed)?;
        }
    } else {
        for source in sources {
            let value: &RawValue =
                serde_json::from_str(source).map_err(|err| ThemeError::Json(err.to_string()))?;
            flatten_group(value, None, &mut path, &mut parsed)?;
        }
    }
    finish_tokens(parsed)
}

fn flatten_value_group(
    value: Value,
    inherited_type: Option<TokenType>,
    path: &mut String,
    parsed: &mut ParsedTokens,
) -> Result<(), ThemeError> {
    let Value::Object(mut object) = value else {
        return Err(ThemeError::InvalidValue {
            path: path.clone(),
            message: "group 必须是 JSON object".to_owned(),
        });
    };

    let local_type = object
        .get("$type")
        .and_then(Value::as_str)
        .map(|raw_type| TokenType::parse(path, raw_type))
        .transpose()?
        .or(inherited_type);

    if let Some(token_value) = object.remove("$value") {
        let full_path = path.clone();
        let token_type = local_type.ok_or_else(|| ThemeError::MissingType {
            path: full_path.clone(),
        })?;
        let description = object
            .remove("$description")
            .as_ref()
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let extensions = object.remove("$extensions");
        push_token(
            full_path,
            token_type,
            token_value,
            description,
            extensions,
            parsed,
        )?;
        return Ok(());
    }

    for (key, child) in object {
        if key.starts_with('$') {
            continue;
        }
        let previous_len = path.len();
        if previous_len > 0 {
            path.push('.');
        }
        path.push_str(&key);
        flatten_value_group(child, local_type, path, parsed)?;
        path.truncate(previous_len);
    }

    Ok(())
}

#[derive(Default)]
struct RawGroupMetadata<'a> {
    token_type: Option<&'a RawValue>,
    value: Option<&'a RawValue>,
    description: Option<&'a RawValue>,
    extensions: Option<&'a RawValue>,
}

impl<'de> Deserialize<'de> for RawGroupMetadata<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RawGroupMetadataVisitor)
    }
}

struct RawGroupMetadataVisitor;

impl<'de> Visitor<'de> for RawGroupMetadataVisitor {
    type Value = RawGroupMetadata<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DTCG token 或 group object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut metadata = RawGroupMetadata::default();
        while let Some(key) = map.next_key_seed(CowStrSeed)? {
            match key.as_ref() {
                "$type" => metadata.token_type = Some(map.next_value()?),
                "$value" => metadata.value = Some(map.next_value()?),
                "$description" => metadata.description = Some(map.next_value()?),
                "$extensions" => metadata.extensions = Some(map.next_value()?),
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(metadata)
    }
}

struct RawGroupChildren<'a>(Vec<(Cow<'a, str>, &'a RawValue)>);

impl<'de> Deserialize<'de> for RawGroupChildren<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(RawGroupChildrenVisitor)
    }
}

struct RawGroupChildrenVisitor;

impl<'de> Visitor<'de> for RawGroupChildrenVisitor {
    type Value = RawGroupChildren<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DTCG group object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut children = Vec::with_capacity(map.size_hint().unwrap_or(0));
        while let Some(key) = map.next_key_seed(CowStrSeed)? {
            if key.starts_with('$') {
                map.next_value::<IgnoredAny>()?;
            } else {
                children.push((key, map.next_value()?));
            }
        }
        Ok(RawGroupChildren(children))
    }
}

struct CowStrSeed;

impl<'de> DeserializeSeed<'de> for CowStrSeed {
    type Value = Cow<'de, str>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CowStrVisitor)
    }
}

struct CowStrVisitor;

impl<'de> Visitor<'de> for CowStrVisitor {
    type Value = Cow<'de, str>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON object key")
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Borrowed(value))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Owned(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Cow::Owned(value))
    }
}

fn flatten_group(
    value: &RawValue,
    inherited_type: Option<TokenType>,
    path: &mut String,
    parsed: &mut ParsedTokens,
) -> Result<(), ThemeError> {
    if !value.get().trim_start().starts_with('{') {
        return Err(ThemeError::InvalidValue {
            path: path.clone(),
            message: "group 必须是 JSON object".to_owned(),
        });
    }

    let metadata = serde_json::from_str::<RawGroupMetadata<'_>>(value.get())
        .map_err(|err| ThemeError::Json(err.to_string()))?;

    let local_type = metadata
        .token_type
        .map(|value| serde_json::from_str::<Value>(value.get()))
        .transpose()
        .map_err(|err| ThemeError::Json(err.to_string()))?
        .as_ref()
        .and_then(Value::as_str)
        .map(|raw_type| TokenType::parse(path, raw_type))
        .transpose()?
        .or(inherited_type);

    if let Some(token_value) = metadata.value {
        let full_path = path.clone();
        let token_type = local_type.ok_or_else(|| ThemeError::MissingType {
            path: full_path.clone(),
        })?;
        let token_value = serde_json::from_str(token_value.get())
            .map_err(|err| ThemeError::Json(err.to_string()))?;
        let description = metadata
            .description
            .map(|value| serde_json::from_str::<Value>(value.get()))
            .transpose()
            .map_err(|err| ThemeError::Json(err.to_string()))?
            .as_ref()
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let extensions = metadata
            .extensions
            .map(|value| serde_json::from_str(value.get()))
            .transpose()
            .map_err(|err| ThemeError::Json(err.to_string()))?;
        push_token(
            full_path,
            token_type,
            token_value,
            description,
            extensions,
            parsed,
        )?;
        return Ok(());
    }

    let children = serde_json::from_str::<RawGroupChildren<'_>>(value.get())
        .map_err(|err| ThemeError::Json(err.to_string()))?;
    for (key, child) in children.0 {
        let previous_len = path.len();
        if previous_len > 0 {
            path.push('.');
        }
        path.push_str(&key);
        flatten_group(child, local_type, path, parsed)?;
        path.truncate(previous_len);
    }

    Ok(())
}

fn push_token(
    path: String,
    token_type: TokenType,
    token_value: Value,
    description: Option<String>,
    extensions: Option<Value>,
    parsed: &mut ParsedTokens,
) -> Result<(), ThemeError> {
    let sequence = parsed.next_sequence();
    if let Some(reference) = alias_reference(&token_value) {
        parsed.aliases.push(PendingAlias {
            sequence,
            path,
            token_type,
            reference: reference.to_owned(),
            description,
            extensions,
            state: Cell::new(AliasState::Unresolved),
            resolved: RefCell::new(None),
            active: Cell::new(false),
        });
    } else {
        let value = parse_value(&path, token_type, &token_value)?;
        parsed.direct.push(StoredToken {
            sequence,
            token: ResolvedToken {
                path,
                value,
                description,
                extensions,
            },
        });
    }
    Ok(())
}

fn finish_tokens(mut parsed: ParsedTokens) -> Result<ResolvedTokens, ThemeError> {
    sort_and_dedup_direct(&mut parsed.direct);
    sort_and_dedup_aliases(&mut parsed.aliases);

    if parsed.aliases.is_empty() {
        for token in &mut parsed.direct {
            token.sequence = 0;
        }
        return Ok(resolved_tokens(parsed.direct));
    }

    for index in 0..parsed.aliases.len() {
        let path = parsed.aliases[index].path.as_str();
        let direct_sequence = direct_index(&parsed.direct, path)
            .map(|index| parsed.direct[index].sequence)
            .unwrap_or(0);
        parsed.aliases[index].active.set(
            direct_index(&parsed.direct, path).is_none()
                || parsed.aliases[index].sequence > direct_sequence,
        );
    }

    let winners = token_winners(&parsed.direct, &parsed.aliases);
    let mut stack = Vec::new();
    for index in 0..parsed.aliases.len() {
        if parsed.aliases[index].active.get() {
            resolve_alias(index, &parsed.direct, &parsed.aliases, &winners, &mut stack)?;
        }
    }
    drop(winners);

    parsed.direct.retain(|token| {
        alias_index(&parsed.aliases, &token.token.path).is_none_or(|alias_index| {
            !parsed.aliases[alias_index].active.get()
                || token.sequence > parsed.aliases[alias_index].sequence
        })
    });
    parsed.direct.reserve(
        parsed
            .aliases
            .iter()
            .filter(|alias| alias.active.get())
            .count(),
    );
    for alias in parsed.aliases {
        if !alias.active.get() {
            continue;
        }
        parsed.direct.push(StoredToken {
            sequence: 0,
            token: ResolvedToken {
                path: alias.path,
                value: alias
                    .resolved
                    .into_inner()
                    .expect("所有 active alias 必须在生成最终 token 前完成解析"),
                description: alias.description,
                extensions: alias.extensions,
            },
        });
    }
    for token in &mut parsed.direct {
        token.sequence = 0;
    }
    parsed
        .direct
        .sort_unstable_by(|left, right| left.token.path.cmp(&right.token.path));
    Ok(resolved_tokens(parsed.direct))
}

fn resolved_tokens(tokens: Vec<StoredToken>) -> ResolvedTokens {
    let mut index = HashTable::with_capacity(tokens.len());
    let hash_builder = DefaultHashBuilder::default();
    for token_index in 0..tokens.len() {
        let hash = hash_builder.hash_one(&tokens[token_index].token.path);
        index.insert_unique(hash, token_index, |token_index| {
            hash_builder.hash_one(&tokens[*token_index].token.path)
        });
    }
    ResolvedTokens {
        tokens,
        index,
        hash_builder,
    }
}

fn sort_and_dedup_direct(tokens: &mut Vec<StoredToken>) {
    tokens.sort_unstable_by(|left, right| {
        left.token
            .path
            .cmp(&right.token.path)
            .then_with(|| Reverse(left.sequence).cmp(&Reverse(right.sequence)))
    });
    let mut write = 0;
    for read in 0..tokens.len() {
        let keep = write == 0 || tokens[read].token.path != tokens[write - 1].token.path;
        if keep {
            tokens.swap(write, read);
            write += 1;
        }
    }
    tokens.truncate(write);
}

fn sort_and_dedup_aliases(aliases: &mut Vec<PendingAlias>) {
    aliases.sort_unstable_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then_with(|| Reverse(left.sequence).cmp(&Reverse(right.sequence)))
    });
    let mut write = 0;
    for read in 0..aliases.len() {
        let keep = write == 0 || aliases[read].path != aliases[write - 1].path;
        if keep {
            aliases.swap(write, read);
            write += 1;
        }
    }
    aliases.truncate(write);
}

fn direct_index(tokens: &[StoredToken], path: &str) -> Option<usize> {
    tokens
        .binary_search_by(|token| token.token.path.as_str().cmp(path))
        .ok()
}

fn alias_index(aliases: &[PendingAlias], path: &str) -> Option<usize> {
    aliases
        .binary_search_by(|alias| alias.path.as_str().cmp(path))
        .ok()
}

#[derive(Clone, Copy)]
enum TokenWinner {
    Direct(usize),
    Alias(usize),
}

fn token_winners<'a>(
    direct: &'a [StoredToken],
    aliases: &'a [PendingAlias],
) -> HashMap<&'a str, TokenWinner> {
    let mut winners = HashMap::with_capacity(direct.len() + aliases.len());
    for (index, token) in direct.iter().enumerate() {
        winners.insert(token.token.path.as_str(), TokenWinner::Direct(index));
    }
    for (index, alias) in aliases.iter().enumerate() {
        match winners.get(alias.path.as_str()) {
            Some(TokenWinner::Direct(direct_index))
                if direct[*direct_index].sequence > alias.sequence => {}
            _ => {
                winners.insert(alias.path.as_str(), TokenWinner::Alias(index));
            }
        }
    }
    winners
}

fn resolve_alias(
    index: usize,
    direct: &[StoredToken],
    aliases: &[PendingAlias],
    winners: &HashMap<&str, TokenWinner>,
    stack: &mut Vec<usize>,
) -> Result<TokenValue, ThemeError> {
    match aliases[index].state.get() {
        AliasState::Resolved => {
            return Ok(aliases[index]
                .resolved
                .borrow()
                .as_ref()
                .expect("Resolved alias 必须持有值")
                .clone());
        }
        AliasState::Visiting => {
            let mut cycle = stack
                .iter()
                .map(|index| aliases[*index].path.as_str())
                .collect::<Vec<_>>();
            cycle.push(&aliases[index].path);
            return Err(ThemeError::CircularReference {
                path: aliases[index].path.clone(),
                cycle: cycle.join(" -> "),
            });
        }
        AliasState::Unresolved => {}
    }

    aliases[index].state.set(AliasState::Visiting);
    stack.push(index);
    let path = aliases[index].path.clone();
    let reference = aliases[index].reference.clone();
    let expected = aliases[index].token_type;
    let winner =
        winners
            .get(reference.as_str())
            .copied()
            .ok_or_else(|| ThemeError::MissingReference {
                path: path.clone(),
                reference: reference.clone(),
            })?;
    let (found, value) = match winner {
        TokenWinner::Direct(target) => (
            direct[target].token.value.token_type(),
            direct[target].token.value.clone(),
        ),
        TokenWinner::Alias(target) => {
            let found = aliases[target].token_type;
            let value = resolve_alias(target, direct, aliases, winners, stack)?;
            (found, value)
        }
    };
    if expected != found {
        return Err(ThemeError::TypeMismatch {
            path,
            expected: expected.as_str().to_owned(),
            found: found.as_str().to_owned(),
        });
    }
    stack.pop();
    aliases[index].state.set(AliasState::Resolved);
    *aliases[index].resolved.borrow_mut() = Some(value.clone());
    Ok(value)
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
