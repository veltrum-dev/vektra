# Action 与快捷键

**目录：** [概述](#概述) · [快速开始](#快速开始) · [按键格式](#按键格式) · [跨平台修饰键](#跨平台修饰键) · [Action 命名](#action-命名) · [上下文感知绑定](#上下文感知绑定) · [最佳实践](#最佳实践)

## 概述

Action 为 GPUI 提供声明式的键盘驱动界面交互。

**关键概念：**

- 使用 `actions!` 宏或 `#[derive(Action)]` 定义 Action
- 使用 `cx.bind_keys()` 绑定按键
- 在元素上使用 `.on_action()` 处理 Action
- 通过 `key_context()` 实现上下文感知

## 快速开始

### 简单 Action

```rust
use gpui::actions;

actions!(editor, [MoveUp, MoveDown, Save, Quit]);

const CONTEXT: &str = "Editor";

#[cfg(target_os = "macos")]
const SAVE_KEY: &str = "cmd-s";
#[cfg(not(target_os = "macos"))]
const SAVE_KEY: &str = "ctrl-s";

#[cfg(target_os = "macos")]
const QUIT_KEY: &str = "cmd-q";
#[cfg(not(target_os = "macos"))]
const QUIT_KEY: &str = "ctrl-q";

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", MoveUp, Some(CONTEXT)),
        KeyBinding::new("down", MoveDown, Some(CONTEXT)),
        KeyBinding::new(SAVE_KEY, Save, Some(CONTEXT)),
        KeyBinding::new(QUIT_KEY, Quit, Some(CONTEXT)),
    ]);
}

impl Render for Editor {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(Self::move_up))
            .on_action(cx.listener(Self::move_down))
            .on_action(cx.listener(Self::save))
    }
}

impl Editor {
    fn move_up(&mut self, _: &MoveUp, cx: &mut Context<Self>) {
        // 处理向上移动
        cx.notify();
    }

    fn move_down(&mut self, _: &MoveDown, cx: &mut Context<Self>) {
        cx.notify();
    }

    fn save(&mut self, _: &Save, cx: &mut Context<Self>) {
        // 保存逻辑
        cx.notify();
    }
}
```

### 带参数的 Action

```rust
#[derive(Clone, PartialEq, Action, Deserialize)]
#[action(namespace = editor)]
pub struct InsertText {
    pub text: String,
}

#[derive(Action, Clone, PartialEq, Eq, Deserialize)]
#[action(namespace = editor, no_json)]
pub struct Digit(pub u8);

cx.bind_keys([
    KeyBinding::new("0", Digit(0), Some(CONTEXT)),
    KeyBinding::new("1", Digit(1), Some(CONTEXT)),
    // ...
]);

impl Editor {
    fn on_digit(&mut self, action: &Digit, cx: &mut Context<Self>) {
        self.insert_digit(action.0, cx);
    }
}
```

## 按键格式

```rust
// Modifiers
"cmd-s"         // macOS Command；Windows/Linux 分别是 Win/Super，不是 Ctrl
"ctrl-c"        // Control
"alt-f"         // Alt
"shift-tab"     // Shift
"cmd-ctrl-f"    // 多个修饰键

// Keys
"a-z", "0-9"    // 字母和数字
"f1-f12"        // 功能键
"up", "down", "left", "right"
"enter", "escape", "space", "tab"
"backspace", "delete"
"-", "=", "[", "]" 等     // 特殊字符
```

## 跨平台修饰键

锁定 GPUI 中，`Modifiers::platform` 表示 macOS Command、Windows 键或 Linux Super；它不是跨平台的“主快捷键”。需要实现 macOS Cmd / Windows/Linux Ctrl 语义时：

- 原始 `KeyDownEvent` 使用 `modifiers.secondary()`；不要把 `modifiers.platform` 当作通用 Command。
- `KeyBinding` 字符串不会把 `cmd-` 自动翻译为非 macOS 的 `ctrl-`。按目标平台显式绑定 `cmd-*` 或 `ctrl-*`，或复用仓库已有平台绑定模式。
- 匹配完整且明确支持的修饰键集合。只有操作确实支持 Shift 时才允许 Shift；Alt、Fn、Control、Win/Super 等额外修饰键不能被顺带接受。
- 未识别组合必须继续冒泡。只有实际执行了组件操作后，才调用 `window.prevent_default()` 与 `cx.stop_propagation()`。
- Windows 键和 Linux Super 不得被当作单词移动、单词删除、复制粘贴或撤销重做修饰键；macOS 的 Fn + 方向键等平台行为需要单独、显式建模。
- 为修饰键判定函数构造显式 `Modifiers` 单元测试，至少覆盖 secondary、secondary + Shift、额外修饰键，以及非 macOS 的 `platform`；能使用对应 target/CI 时再补目标平台编译或交互验证。

原始按键处理的精确匹配可以采用：

```rust
fn secondary_shortcut(modifiers: Modifiers, shift: bool) -> bool {
    modifiers.secondary()
        && modifiers.shift == shift
        && modifiers.number_of_modifiers() == if shift { 2 } else { 1 }
}

if event.keystroke.key == "s" && secondary_shortcut(event.keystroke.modifiers, false) {
    save();
    window.prevent_default();
    cx.stop_propagation();
}
```

## Action 命名

优先采用“动词-名词”模式：

```rust
actions!([
    OpenFile,      // ✅ Good
    CloseWindow,   // ✅ Good
    ToggleSidebar, // ✅ Good
    Save,          // ✅ 合适（常见例外）
]);
```

## 上下文感知绑定

```rust
const EDITOR_CONTEXT: &str = "Editor";
const MODAL_CONTEXT: &str = "Modal";

// 同一按键，不同上下文
cx.bind_keys([
    KeyBinding::new("escape", CloseModal, Some(MODAL_CONTEXT)),
    KeyBinding::new("escape", ClearSelection, Some(EDITOR_CONTEXT)),
]);

// 在元素上设置上下文
div()
    .key_context(EDITOR_CONTEXT)
    .child(editor_content)
```

## 最佳实践

### ✅ 使用上下文

```rust
// ✅ 合适：感知上下文
div()
    .key_context("MyComponent")
    .on_action(cx.listener(Self::handle))
```

### ✅ 清晰命名 Action

```rust
// ✅ 合适：意图清晰
actions!([
    SaveDocument,
    CloseTab,
    TogglePreview,
]);
```

### ✅ 使用监听器处理

```rust
// ✅ 合适：处理器命名恰当
impl MyComponent {
    fn on_action_save(&mut self, _: &Save, cx: &mut Context<Self>) {
        // 处理保存
        cx.notify();
    }
}

div().on_action(cx.listener(Self::on_action_save))
```
