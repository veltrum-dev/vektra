//! 集合型组件共享的惰性数据源协议与内存内适配器。

use gpui::{App, Window};
use std::{hash::Hash, ops::Range, rc::Rc};

/// 集合型组件共享的非阻塞惰性数据源协议。
///
/// 数据源必须为每个逻辑索引提供稳定 key。`item` 与 `request_range` 会在 UI
/// 线程调用，因此不得执行阻塞 I/O；分页或远程实现应只返回当前已加载数据，并通过
/// `request_range` 通知宿主在后台加载缺失范围。
pub trait LazyDataSource: 'static {
    /// 单个逻辑项目的值类型。
    type Item: Clone + 'static;

    /// 跨 revision 仍可用于识别同一逻辑项目的 key 类型。
    type Key: Clone + Eq + Hash + 'static;

    /// 返回当前逻辑项目总数。
    fn item_count(&self) -> usize;

    /// 返回数据 revision；项目、顺序或加载状态变化后必须改变。
    fn revision(&self) -> u64;

    /// 返回指定索引的稳定 key。
    ///
    /// 即使该索引对应的分页数据尚未加载，也必须能返回稳定 key。
    fn key(&self, index: usize) -> Self::Key;

    /// 惰性读取当前已加载项目；缺页时返回 `None`。
    fn item(&self, index: usize) -> Option<Self::Item>;

    /// 返回指定索引当前是否已加载。
    fn is_loaded(&self, index: usize) -> bool {
        self.item(index).is_some()
    }

    /// 通知宿主即将需要指定可见范围。
    ///
    /// 实现只能安排非阻塞加载，不得在调用期间等待网络、文件或其他外部 I/O。
    fn request_range(&self, _range: Range<usize>, _window: &mut Window, _cx: &mut App) {}
}

type KeyFactory<T, K> = dyn Fn(usize, &T) -> K + 'static;

/// 将 `Vec`、数组或其他已拥有集合适配到 [`LazyDataSource`] 的内存内数据源。
///
/// 适配器只持有调用方交付的一份项目数据；key 在读取时生成，不建立第二份全量 key
/// catalog。适合中小型已拥有数据，大型生成式、分页或远程数据应直接实现
/// [`LazyDataSource`]。
#[derive(Clone)]
pub struct OwnedDataSource<T, K> {
    items: Rc<[T]>,
    key: Rc<KeyFactory<T, K>>,
    revision: u64,
}

impl<T, K> OwnedDataSource<T, K>
where
    T: Clone + 'static,
    K: Clone + Eq + Hash + 'static,
{
    /// 从 `Vec` 创建适配器，并为每项惰性生成稳定 key。
    pub fn from_vec(items: Vec<T>, key: impl Fn(usize, &T) -> K + 'static) -> Self {
        Self {
            items: items.into(),
            key: Rc::new(key),
            revision: 0,
        }
    }

    /// 从定长数组创建适配器，并为每项惰性生成稳定 key。
    pub fn from_array<const N: usize>(
        items: [T; N],
        key: impl Fn(usize, &T) -> K + 'static,
    ) -> Self {
        Self::from_vec(Vec::from(items), key)
    }

    /// 设置数据 revision。
    pub fn with_revision(mut self, revision: u64) -> Self {
        self.revision = revision;
        self
    }

    /// 返回适配器持有的项目切片。
    pub fn items(&self) -> &[T] {
        &self.items
    }
}

impl<T, K> LazyDataSource for OwnedDataSource<T, K>
where
    T: Clone + 'static,
    K: Clone + Eq + Hash + 'static,
{
    type Item = T;
    type Key = K;

    fn item_count(&self) -> usize {
        self.items.len()
    }

    fn revision(&self) -> u64 {
        self.revision
    }

    fn key(&self, index: usize) -> Self::Key {
        let item = self
            .items
            .get(index)
            .expect("OwnedDataSource key 索引必须位于 item_count 内");
        (self.key)(index, item)
    }

    fn item(&self, index: usize) -> Option<Self::Item> {
        self.items.get(index).cloned()
    }
}
