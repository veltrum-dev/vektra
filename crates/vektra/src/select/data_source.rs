//! Select 的统一惰性数据源与 owned adapter。

use super::{SelectChild, SelectGroupHeader, SelectOption};
use crate::LazyDataSource;
use gpui::ElementId;
use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

/// Select 惰性数据源中的单行。
#[derive(Clone)]
pub enum SelectEntry<T> {
    /// 可导航、可选择的 option 行。
    Option(SelectOption<T>),
    /// 不参与选择和键盘导航的分组标题行。
    Group(SelectGroupHeader),
}

impl<T> SelectEntry<T> {
    /// 返回该行的稳定标识。
    pub fn id(&self) -> &ElementId {
        match self {
            Self::Option(option) => option.id(),
            Self::Group(group) => group.id(),
        }
    }
}

/// Select 使用的统一惰性数据源协议。
///
/// 大型、分页或远程实现必须自行维护 value/key、enabled navigation 与 typeahead 索引；
/// Vektra 只读取当前可见行，不会为外部数据源建立百万级 catalog 或校验 `HashSet`。
/// 所有方法都在 UI 线程调用，不得阻塞。
pub trait SelectDataSource<T>: LazyDataSource<Item = SelectEntry<T>, Key = ElementId>
where
    T: Clone + Eq + Hash + 'static,
{
    /// 按稳定 key 定位当前行。
    fn index_of_key(&self, key: &ElementId) -> Option<usize>;

    /// 按业务值定位 canonical option。
    fn index_of_value(&self, value: &T) -> Option<usize>;

    /// 返回第一项可用 option 的行索引。
    fn first_enabled(&self) -> Option<usize>;

    /// 返回指定行是否为当前已加载且可选择的 canonical option。
    fn is_enabled(&self, index: usize) -> bool;

    /// 返回最后一项可用 option 的行索引。
    fn last_enabled(&self) -> Option<usize>;

    /// 返回指定行之前或之后的可用 option。
    ///
    /// `wrap` 为真时允许从末尾回到开头，供 typeahead 使用；普通方向键应传 `false`。
    fn next_enabled(&self, index: usize, forward: bool, wrap: bool) -> Option<usize>;

    /// 从 `after` 之后循环查找名称匹配已 case-fold 前缀的可用 option。
    fn search_prefix(&self, query: &str, after: Option<usize>) -> Option<usize>;

    /// 返回数据源中 option 的总数，不包含 group 行。
    fn option_count(&self) -> usize;

    /// 返回指定 option 行在可访问集合中的零基位置；group 行返回 `None`。
    fn option_position(&self, index: usize) -> Option<usize>;
}

/// 将 `SelectOption`、数组、逐项 builder 与 group 适配到统一 Select 数据源的 owned
/// adapter。
///
/// adapter 消费并持有调用方交付的一份业务行；构造时使用临时 `HashSet` 以预期 O(n)
/// 保留 first-canonical 语义，临时集合随后释放。持久状态只包含定位和 enabled navigation
/// 索引，不复制标签、描述或搜索文本。
pub struct OwnedSelectDataSource<T>
where
    T: Clone + Eq + Hash + 'static,
{
    entries: Vec<SelectEntry<T>>,
    keys: HashMap<ElementId, usize>,
    values: HashMap<T, usize>,
    enabled: Vec<usize>,
    option_positions: Vec<Option<usize>>,
    option_count: usize,
    revision: u64,
}

impl<T> OwnedSelectDataSource<T>
where
    T: Clone + Eq + Hash + 'static,
{
    /// 从 option `Vec` 创建 owned adapter。
    pub fn from_options(options: Vec<SelectOption<T>>) -> Self {
        Self::from_entries(options.into_iter().map(SelectEntry::Option).collect())
    }

    /// 从 option 数组创建 owned adapter。
    pub fn from_array<const N: usize>(options: [SelectOption<T>; N]) -> Self {
        Self::from_options(Vec::from(options))
    }

    /// 从 option 与 group 标题行创建 owned adapter。
    pub fn from_entries(mut entries: Vec<SelectEntry<T>>) -> Self {
        let mut seen_ids = HashSet::with_capacity(entries.len());
        let mut seen_values = HashSet::with_capacity(entries.len());
        let mut keys = HashMap::with_capacity(entries.len());
        let mut values = HashMap::with_capacity(entries.len());
        let mut enabled = Vec::with_capacity(entries.len());
        let mut option_positions = Vec::with_capacity(entries.len());
        let mut option_count = 0;
        let mut revision = std::collections::hash_map::DefaultHasher::new();

        for (index, entry) in entries.iter_mut().enumerate() {
            index.hash(&mut revision);
            entry.id().hash(&mut revision);
            keys.entry(entry.id().clone()).or_insert(index);
            match entry {
                SelectEntry::Option(option) => {
                    let unique_id = seen_ids.insert(option.id.clone());
                    let unique_value = seen_values.insert(option.value.clone());
                    option.canonical = unique_id && unique_value;
                    option.disabled |= !option.canonical;
                    option.value.hash(&mut revision);
                    option.disabled.hash(&mut revision);
                    option_positions.push(Some(option_count));
                    option_count += 1;
                    if option.canonical {
                        values.insert(option.value.clone(), index);
                        if !option.disabled {
                            enabled.push(index);
                        }
                    }
                }
                SelectEntry::Group(_) => option_positions.push(None),
            }
        }

        Self {
            entries,
            keys,
            values,
            enabled,
            option_positions,
            option_count,
            revision: revision.finish(),
        }
    }

    /// 设置数据 revision。
    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    pub(super) fn from_children(children: Vec<SelectChild<T>>) -> Self {
        let row_count = children
            .iter()
            .map(|child| match child {
                SelectChild::Option(_) => 1,
                SelectChild::Group(group) => 1 + group.options.len(),
            })
            .sum();
        let mut entries = Vec::with_capacity(row_count);
        for child in children {
            match child {
                SelectChild::Option(option) => entries.push(SelectEntry::Option(option)),
                SelectChild::Group(group) => {
                    entries.push(SelectEntry::Group(SelectGroupHeader {
                        id: group.id,
                        label: group.label,
                        aria_label: group.aria_label,
                    }));
                    entries.extend(group.options.into_iter().map(SelectEntry::Option));
                }
            }
        }
        Self::from_entries(entries)
    }

    fn option_at(&self, index: usize) -> Option<&SelectOption<T>> {
        match self.entries.get(index) {
            Some(SelectEntry::Option(option)) => Some(option),
            _ => None,
        }
    }
}

impl<T> LazyDataSource for OwnedSelectDataSource<T>
where
    T: Clone + Eq + Hash + 'static,
{
    type Item = SelectEntry<T>;
    type Key = ElementId;

    fn item_count(&self) -> usize {
        self.entries.len()
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn key(&self, index: usize) -> Self::Key {
        self.entries
            .get(index)
            .expect("OwnedSelectDataSource key 索引必须位于 item_count 内")
            .id()
            .clone()
    }

    fn item(&self, index: usize) -> Option<Self::Item> {
        self.entries.get(index).cloned()
    }
}

impl<T> SelectDataSource<T> for OwnedSelectDataSource<T>
where
    T: Clone + Eq + Hash + 'static,
{
    fn index_of_key(&self, key: &ElementId) -> Option<usize> {
        self.keys.get(key).copied()
    }

    fn index_of_value(&self, value: &T) -> Option<usize> {
        self.values.get(value).copied()
    }

    fn first_enabled(&self) -> Option<usize> {
        self.enabled.first().copied()
    }

    fn is_enabled(&self, index: usize) -> bool {
        self.enabled.binary_search(&index).is_ok()
    }

    fn last_enabled(&self) -> Option<usize> {
        self.enabled.last().copied()
    }

    fn next_enabled(&self, index: usize, forward: bool, wrap: bool) -> Option<usize> {
        if self.enabled.is_empty() {
            return None;
        }
        if forward {
            let position = self
                .enabled
                .partition_point(|candidate| *candidate <= index);
            self.enabled
                .get(position)
                .copied()
                .or_else(|| wrap.then(|| self.enabled[0]))
        } else {
            let position = self.enabled.partition_point(|candidate| *candidate < index);
            position
                .checked_sub(1)
                .and_then(|position| self.enabled.get(position).copied())
                .or_else(|| wrap.then(|| *self.enabled.last().expect("enabled 非空")))
        }
    }

    fn search_prefix(&self, query: &str, after: Option<usize>) -> Option<usize> {
        if query.is_empty() || self.enabled.is_empty() {
            return None;
        }
        let start = after
            .map(|index| {
                self.enabled
                    .partition_point(|candidate| *candidate <= index)
            })
            .unwrap_or(0);
        (0..self.enabled.len()).find_map(|offset| {
            let index = self.enabled[(start + offset) % self.enabled.len()];
            let option = self.option_at(index)?;
            option
                .accessible_label()
                .to_lowercase()
                .starts_with(query)
                .then_some(index)
        })
    }

    fn option_count(&self) -> usize {
        self.option_count
    }

    fn option_position(&self, index: usize) -> Option<usize> {
        self.option_positions.get(index).copied().flatten()
    }
}
