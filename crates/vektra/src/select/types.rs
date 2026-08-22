//! Select 的公开 option、group 与状态类型。

use crate::traits::Disableable;
use gpui::{ElementId, SharedString};

/// Select Popup 当前由宿主控制的互斥内容状态。
///
/// 状态内容不是 option，不参与键盘导航，也不会产生受控变化回调。Select
/// 不发起异步请求；宿主完成加载、重试或数据更新后，应传入新的状态与 option。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SelectStatus {
    /// 显示结构化 option 与 group。
    #[default]
    Ready,
    /// 显示宿主提供的加载文案。
    Loading(SharedString),
    /// 显示宿主提供的空状态文案。
    Empty(SharedString),
    /// 显示宿主提供的错误文案。
    Error(SharedString),
}

impl SelectStatus {
    /// 创建 loading 状态。
    pub fn loading(message: impl Into<SharedString>) -> Self {
        Self::Loading(message.into())
    }

    /// 创建 empty 状态。
    pub fn empty(message: impl Into<SharedString>) -> Self {
        Self::Empty(message.into())
    }

    /// 创建 error 状态。
    pub fn error(message: impl Into<SharedString>) -> Self {
        Self::Error(message.into())
    }

    pub(super) fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Select 中一个结构化、强类型的可选项。
///
/// `id` 与 `value` 在同一个 Select 中都应唯一。若出现重复，Select 按输入顺序只把
/// 第一个同时拥有未重复 ID 与未重复值的项作为 canonical option；后续冲突项仍可见，
/// 但按禁用项处理，不会形成第二个选中视觉或重复变化回调。
#[derive(Clone)]
pub struct SelectOption<T> {
    pub(super) id: ElementId,
    pub(super) value: T,
    pub(super) label: SharedString,
    pub(super) icon: Option<super::IconSource>,
    pub(super) description: Option<SharedString>,
    pub(super) aria_label: Option<SharedString>,
    pub(super) aria_description: Option<SharedString>,
    pub(super) disabled: bool,
    pub(super) canonical: bool,
}

impl<T> SelectOption<T> {
    /// 创建带稳定 `ElementId`、业务值和可见标签的 option。
    pub fn new(id: impl Into<ElementId>, value: T, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            value,
            label: label.into(),
            icon: None,
            description: None,
            aria_label: None,
            aria_description: None,
            disabled: false,
            canonical: true,
        }
    }

    /// 设置 option 的可选前置图标。
    pub fn icon(mut self, icon: super::IconSource) -> Self {
        self.icon = Some(icon);
        self
    }

    /// 设置显示在主标签下方的补充描述。
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 覆盖辅助技术使用的 option 名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 覆盖辅助技术使用的 option 描述。
    pub fn aria_description(mut self, description: impl Into<SharedString>) -> Self {
        self.aria_description = Some(description.into());
        self
    }

    /// 设置单项禁用状态。
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// 返回 option 的稳定标识。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub(super) fn accessible_label(&self) -> SharedString {
        self.aria_label
            .clone()
            .unwrap_or_else(|| self.label.clone())
    }

    pub(super) fn accessible_description(&self) -> Option<SharedString> {
        self.aria_description
            .clone()
            .or_else(|| self.description.clone())
    }
}

/// 惰性 Select 数据源中的分组标题行。
#[derive(Clone)]
pub struct SelectGroupHeader {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) aria_label: Option<SharedString>,
}

impl SelectGroupHeader {
    /// 创建稳定分组标题行。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aria_label: None,
        }
    }

    /// 覆盖辅助技术使用的分组名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 返回分组标题的稳定标识。
    pub fn id(&self) -> &ElementId {
        &self.id
    }

    pub(super) fn accessible_label(&self) -> SharedString {
        self.aria_label
            .clone()
            .unwrap_or_else(|| self.label.clone())
    }
}

impl<T> Disableable for SelectOption<T> {
    fn disabled(self, disabled: bool) -> Self {
        SelectOption::disabled(self, disabled)
    }
}

/// Select 中带可见标题的一组结构化 option。
pub struct SelectGroup<T> {
    pub(super) id: ElementId,
    pub(super) label: SharedString,
    pub(super) aria_label: Option<SharedString>,
    pub(super) options: Vec<SelectOption<T>>,
}

impl<T> SelectGroup<T> {
    /// 创建带稳定 `ElementId` 和可见标题的 group。
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            aria_label: None,
            options: Vec::new(),
        }
    }

    /// 覆盖辅助技术使用的 group 名称。
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    /// 向 group 添加一个类型一致的 option。
    pub fn option(mut self, option: SelectOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// 返回 group 的稳定标识。
    pub fn id(&self) -> &ElementId {
        &self.id
    }
}

pub(super) enum SelectChild<T> {
    Option(SelectOption<T>),
    Group(SelectGroup<T>),
}
