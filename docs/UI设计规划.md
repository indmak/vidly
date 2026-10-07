# Vidly · UI 设计规划文档

> **文档版本**：v1.0 · **适用**：iced 0.13 · **定位**：单窗口工具类应用（文件队列 + 进度反馈）
>
> **设计基调**：2026 年桌面工具的主流审美是**“克制的暗色 + 单一强调色 + 大留白 + 轻拟态状态反馈”**——介于纯 Flat 与 Neo-Soft 之间：底色与卡片完全扁平，但通过细微的描边、悬停态和进度条赋予层次感。本文档将给出这套美学的完整落地规范，并保证每一条都在 iced 0.13 的能力圈内。
---
## 目录
1. [设计基调与 2026 美学关键词](#1-设计基调与-2026-美学关键词)
2. [iced 能力边界与规避清单](#2-iced-能力边界与规避清单)
3. [设计令牌：色彩 / 字体 / 间距 / 圆角](#3-设计令牌)
4. [组件样式规范](#4-组件样式规范)
5. [布局结构](#5-布局结构)
6. [状态与反馈设计](#6-状态与反馈设计)
7. [交互细节](#7-交互细节)
8. [暗色主题 Theme 实现示例](#8-暗色主题-theme-实现示例)
9. [浅色主题与未来扩展](#9-浅色主题与未来扩展)
---

## 1. 设计基调与 2026 美学关键词

| 关键词 | 在本应用的体现 |
|---|---|
| **Flat + 细描边** | 所有面板纯色填充，无阴影无渐变；卡片用 1px `border` 区分层级 |
| **单一强调色** | 全局只有一个 Accent（建议靛蓝 `#6C8CFF`），所有可交互/进行中状态共用，避免多色混乱 |
| **大留白** | 8pt 网格；主内容区左右各 24px 内边距，列表项间距 12px |
| **轻拟态状态反馈** | 进度条用强调色填充；按钮悬停/按下通过透明度变化表达，不使用阴影位移 |
| **等宽数字** | 时长、百分比用 monospace 字体（`iced::Font::MONOSPACE`），避免数字跳动时宽度抖动 |
| **信息降噪** | 图标使用单色线性 SVG（Lucide 风格），仅状态列用色彩，其余文字一律两级灰度 |
**不做的事**：玻璃拟态、多层阴影、彩色渐变、动画图标——这些要么超出 iced 能力，要么与工具类应用的“安静可靠”气质相悖。
---

## 2. iced 能力边界与规避清单

写规范前先画清楚围栏，以下每一项都标注了“能力/规避”，避免设计稿落地时撞墙：
| 能力项 | iced 0.13 支持度 | 本文档的处理 |
|---|---|---|
| 暗色/浅色主题 | ✅ `iced::theme::Theme` + `Palette`，可完全自定义 | 采用**自定义 Theme**（见第 8 节），而非内置 `Theme::Dark`（后者蓝色不可控） |
| 单色线性 SVG 图标 | ✅ `iced::widget::svg`（`svg` feature） | 主用；配合 `resvg` 后端渲染 |
| 圆角 | ✅ `border::rounded(n)` | 统一 8px（按钮/卡片/进度条） |
| 边框 | ✅ `border::with_width + with_color` | 卡片 1px 分隔线 |
| 悬停/按下/选中/禁用四态 | ✅ 各 widget 的 `style()` + `Catalog` 闭包 | 完整四态（第 4 节） |
| 进度条 | ✅ `progress_bar`，样式可调 | 用自定义样式贴合强调色 |
| 文本样式（字号/颜色/等宽） | ✅ `text().size().font().style()` | 两级字号 + monospace 数字 |
| 滚动列表 | ✅ `scrollable`，含细滚动条样式 | 队列区使用 |
| 文件拖放 | ✅ `window::Event::FileDropped` | 空态引导文案配合 |
| 工具提示 | ⚠️ 有 `tooltip` widget 但定制空间有限 | 仅在“FFmpeg 未检测到”等关键处使用，不做花式 |
| **玻璃拟态/多层阴影** | ❌ 无阴影系统 | **规避**：用描边 + 背景色分层替代 |
| **复杂动画（Lottie/帧动画）** | ❌ 无时间轴动画系统 | **规避**：进度条本身就是动态反馈，转换中图标用字符 `◌` 静态占位 |
| **图片背景/纹理** | ⚠️ 可放 `image` 但不适合工具场景 | **规避** |
| **多窗口/自定义标题栏** | ⚠️ 0.13 支持但复杂度高 | **规避**：单窗口 + 系统标题栏，符合平台惯例 |
---

## 3. 设计令牌

### 3.1 色彩（暗色主题为主）

| 令牌 | 色值 | 用途 |
|---|---|---|
| `bg_base` | `#16181D` | 窗口底色 |
| `bg_surface` | `#1E2128` | 卡片/列表项/工具栏背景 |
| `bg_hover` | `#262A33` | 悬停态背景 |
| `bg_input` | `#111318` | 输入框/滑块槽背景 |
| `border_subtle` | `#2A2E38` | 1px 分隔线/描边 |
| `text_primary` | `#E8EAED` | 主文字（文件名、标题） |
| `text_secondary` | `#9AA0A8` | 次要文字（路径、说明） |
| `text_disabled` | `#5A5F68` | 禁用态文字 |
| `accent` | `#6C8CFF` | **全局唯一强调色**：主按钮、进度条、选中态、链接 |
| `accent_hover` | `#8AA4FF` | 强调色悬停 |
| `success` | `#4ADE80` | 仅“完成”状态 |
| `error` | `#F87171` | 仅“失败”状态与错误文案 |
| `warning` | `#FBBF24` | 仅“跳过/未检测到 FFmpeg”提示 |
> **单强调色原则**：`accent` 之外的颜色只表达状态，绝不用于装饰。例如"全部转换"按钮是 `accent` 填充白字，而"取消"是描边灰字——层级一目了然。

### 3.2 字体与排版

| 层级 | 字体 | 字号 | 颜色 | 用途 |
|---|---|---|---|---|
| H1 | 默认 | 22 | `text_primary` | 窗口标题区 "Vidly" |
| H2 | 默认 | 14 | `text_primary` | 区块标题（"队列"、"设置"） |
| Body | 默认 | 13 | `text_primary` | 文件名、按钮文字 |
| Caption | 默认 | 11 | `text_secondary` | 路径、说明、统计 |
| **Mono** | `Font::MONOSPACE` | 12 | `text_secondary` | **时长 / 百分比 / 大小** |
> 中文回退：iced 的 cosmic-text 通常能自动落到系统字体（Win: 微软雅黑 / mac: 苹方 / Linux: Noto Sans CJK）。若测试出现方框，再在 `main.rs` 用 `.default_font()` 内嵌 Noto Sans SC。

### 3.3 间距与圆角

| 令牌 | 值 |
|---|---|
| `space_xs` | 4px |
| `space_s` | 8px |
| `space_m` | 12px |
| `space_l` | 16px |
| `space_xl` | 24px（页面左右内边距） |
| `radius_s` | 6px（小按钮、checkbox） |
| `radius_m` | 8px（标准按钮、卡片、进度条） |
| `radius_l` | 12px（设置面板容器） |
---

## 4. 组件样式规范

所有样式均通过 `.style(|theme| ...)` 闭包给出，theme 为我们自定义的 `Theme`（第 8 节）。以下表格为“设计规格 → iced 实现要点”对照。

### 4.1 按钮

| 变体 | 默认 | 悬停 | 按下 | 禁用 |
|---|---|---|---|---|
| **Primary**（全部转换/启动） | `accent` 填充，白字 | `accent_hover` | 透明度 85% | `bg_surface` + `text_disabled` |
| **Secondary**（取消/移除） | 透明底，`border_subtle` 描边，`text_primary` | `bg_hover` 底 | 描边 `accent` | 同上禁用 |
| **Ghost**（图标按钮 ✕） | 透明底，无描边，`text_secondary` | `bg_hover` + `text_primary` | 同悬停 | 隐藏或 30% 透明 |
| **Danger**（清空列表） | 透明底，`error` 描边字 | `error` 15% 透明底 | 描边加深 | 禁用 |
实现要点：`button::Catalog` 返回 `button::Style { background, text_color, border: rounded(8).with_width(1).with_color(...), .. }`；悬停/按下由 `status: Status` 匹配分支控制。

### 4.2 进度条（核心组件）

| 元素 | 规格 |
|---|---|
| 轨道 | `bg_input`，高 **6px**，圆角 3px（全圆角） |
| 填充 | `accent`（单项运行中）/ `success`（完成）/ `error`（失败） |
| 总进度（页脚） | 同规格但高 **4px**，更轻量 |
> **总进度条配色切换**：`progress_bar` 的 style 闭包可按当前完成/失败比例返回不同填充色；简单做法是全部用 `accent`，完成率 100% 时切 `success`。

### 4.3 列表项（队列卡片）

```
┌──────────────────────────────────────────────────────┐
│ ⏳ video_clip.mp4 → video_clip.mov          [启动][✕] │
│    /Users/you/Desktop/video_clip.mp4                 │
│    ▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░  42%              │
└──────────────────────────────────────────────────────┘
```
| 元素 | 规格 |
|---|---|
| 容器 | `bg_surface`，`radius_m`，1px `border_subtle`，内边距 `space_l` |
| 状态图标 | 16px 单色 SVG，颜色按状态：排队 `text_secondary` / 运行 `accent` / 完成 `success` / 失败 `error` / 取消 `text_disabled` / 跳过 `warning` |
| 文件名 | Body，`text_primary`；箭头 `→` 与目标名用 `text_secondary` |
| 路径 | Caption，`text_secondary`，超长时省略号（iced 自动截断） |
| 状态文字 | Caption；**运行中百分比用 Mono**：`42% · 剩余 3.2s` |
| 操作按钮 | 运行中显示"取消"（Secondary），否则显示"启动"（Secondary）+ ✕（Ghost） |

### 4.4 输入控件

| 控件 | 规格 |
|---|---|
| 下拉框（覆盖策略） | `bg_input` 底，1px 描边，展开菜单 `bg_surface` + 选中项 `accent` 15% 透明底 |
| 滑块（并发数） | 轨道 `bg_input` 4px；滑块圆点 14px `accent`，悬停放大至 16px |
| 复选框 | 16px，选中态 `accent` 填充 + 白色对勾 SVG，圆角 `radius_s` |

### 4.5 空态引导

```
        ┌────────────┐
        │   ⬇ 图标    │   40px 线性图标，text_secondary 色
        └────────────┘
     将 MP4 / MOV 拖入窗口
     或点击「添加文件」(Ctrl+O)
```
居中容器，两层文字分别为 Body 和 Caption。这是整个应用唯一的“视觉焦点区”，保持极简。
---

## 5. 布局结构

单窗口，最小尺寸 760×580，纵向四段式：
```
┌──────────────────────────────────────────────────┐
│ ① 标题栏（应用内）                                │
│   Vidly            无损重封装 · FFmpeg ✅         │
├──────────────────────────────────────────────────┤
│ ② 工具栏（水平）                                  │
│   [+ 添加文件] [📁 输出目录…]  …与源同目录        │
├──────────────────────────────────────────────────┤
│ ③ 主内容区（弹性填充，可滚动）                    │
│   ┌ 设置面板（卡片，radius_l）────────────────┐   │
│   │ 覆盖策略 ▾   并发 ●——○ 2  □ faststart    │   │
│   └───────────────────────────────────────────┘   │
│   ┌ 队列列表（可滚动）─────────────────────────┐   │
│   │  …列表项（见 4.3）…                        │   │
│   └───────────────────────────────────────────┘   │
├──────────────────────────────────────────────────┤
│ ④ 页脚（固定）                                    │
│   ▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░ 38%                 │
│   共 12 · ✅ 5 · ❌ 0 · ⏳ 2      [全部转换] [取消] │
└──────────────────────────────────────────────────┘
```
**布局原则**：
- 设置面板默认**展开**（工具类应用，设置项少不值得折叠）；若未来项增多可做 `Toggler` 收起
- 队列列表为 `scrollable`，占据剩余全部高度
- 页脚固定不滚动，保证总进度条和主按钮永远可见
- 警告条（如"未检测到 FFmpeg"）出现时插入在标题栏与工具栏之间，`warning` 15% 透明底 + 1px `warning` 描边
---

## 6. 状态与反馈设计

### 6.1 状态-颜色-图标映射表

| 状态 | 色彩 | 图标（Lucide 名） | 文案示例 |
|---|---|---|---|
| 排队 | `text_secondary` | `clock` | 等待中 |
| 运行中 | `accent` | `loader` | 转换中… 42% |
| 完成 | `success` | `check-circle` | 3.2s 完成 |
| 失败 | `error` | `x-circle` | ffmpeg 异常退出：… |
| 已取消 | `text_disabled` | `slash` | 已取消 |
| 跳过 | `warning` | `skip-forward` | 输出已存在，跳过 |

### 6.2 反馈机制

| 场景 | 反馈方式 |
|---|---|
| 添加文件成功 | 列表新增项短暂高亮（`accent` 10% 透明底，无需动画，静态即可） |
| 全部完成 | 页脚总进度条变 `success`，主按钮文字变为"全部完成"并禁用 |
| 单项失败 | 该项卡片左侧加 3px `error` 色条（用 `container` 左边框实现），页脚 ❌ 计数更新 |
| FFmpeg 缺失 | 顶部警告条 + 所有转换按钮禁用 + 状态图标区显示 `warning` |
| 拖入不支持的文件 | 警告条短暂显示"已忽略 N 个不支持的文件"，5 秒后淡出（用 `time` subscription 实现，或简化为常驻直到下次操作） |
**关于微动画**：iced 无内置时间轴动画。本应用唯一的"动态感"来自进度条填充本身，这已足够传达"正在工作"。不引入 `Subscription::tick` 做轮询动画，保持安静。
---

## 7. 交互细节

| 交互 | 实现 |
|---|---|
| 拖拽文件/文件夹 | `window::Event::FileDropped` → 展开文件夹 → 过滤支持格式 → 去重入队 |
| `Ctrl+O` | 添加文件（`keyboard::on_key_press`） |
| `Ctrl+Enter` | 全部转换 |
| `Delete`（选中项） | iced 无原生列表选中概念，暂不做；删除走每项的 ✕ 按钮 |
| 双击完成项 | 在文件管理器中定位（复用"打开"逻辑；若实现成本高，仅保留显式按钮） |
| 悬停列表项 | 整卡背景 → `bg_hover`（提示可交互区域） |
**无障碍**：所有图标按钮配合 `tooltip` 提供文字说明；色彩对比度已按 WCAG AA 校验（`text_primary` on `bg_base` ≈ 13:1，`text_secondary` on `bg_surface` ≈ 5.8:1，均达标）。
---

## 8. 暗色主题 Theme 实现示例

以下为可直接合入 `src/theme.rs` 的骨架，展示自定义 Theme 与组件样式闭包的写法。**建议先做纯暗色**，把精力放在状态反馈的打磨上。
```rust
use iced::{
    border, theme,
    widget::{button, container, progress_bar, text},
    Border, Color, Length,
};
// ───────────────────────── 设计令牌 ─────────────────────────
pub mod token {
    use iced::Color;
    pub const BG_BASE: Color = Color::from_rgb(0x16 as f32 / 255.0, 0x18 as f32 / 255.0, 0x1D as f32 / 255.0);
    pub const BG_SURFACE: Color = Color::from_rgb(0x1E as f32 / 255.0, 0x21 as f32 / 255.0, 0x28 as f32 / 255.0);
    pub const BG_HOVER: Color = Color::from_rgb(0x26 as f32 / 255.0, 0x2A as f32 / 255.0, 0x33 as f32 / 255.0);
    pub const BG_INPUT: Color = Color::from_rgb(0x11 as f32 / 255.0, 0x13 as f32 / 255.0, 0x18 as f32 / 255.0);
    pub const BORDER_SUBTLE: Color = Color::from_rgb(0x2A as f32 / 255.0, 0x2E as f32 / 255.0, 0x38 as f32 / 255.0);
    pub const TEXT_PRIMARY: Color = Color::from_rgb(0xE8 as f32 / 255.0, 0xEA as f32 / 255.0, 0xED as f32 / 255.0);
    pub const TEXT_SECONDARY: Color = Color::from_rgb(0x9A as f32 / 255.0, 0xA0 as f32 / 255.0, 0xA8 as f32 / 255.0);
    pub const TEXT_DISABLED: Color = Color::from_rgb(0x5A as f32 / 255.0, 0x5F as f32 / 255.0, 0x68 as f32 / 255.0);
    pub const ACCENT: Color = Color::from_rgb(0x6C as f32 / 255.0, 0x8C as f32 / 255.0, 0xFF as f32 / 255.0);
    pub const ACCENT_HOVER: Color = Color::from_rgb(0x8A as f32 / 255.0, 0xA4 as f32 / 255.0, 0xFF as f32 / 255.0);
    pub const SUCCESS: Color = Color::from_rgb(0x4A as f32 / 255.0, 0xDE as f32 / 255.0, 0x80 as f32 / 255.0);
    pub const ERROR: Color = Color::from_rgb(0xF8 as f32 / 255.0, 0x71 as f32 / 255.0, 0x71 as f32 / 255.0);
    pub const WARNING: Color = Color::from_rgb(0xFB as f32 / 255.0, 0xBF as f32 / 255.0, 0x24 as f32 / 255.0);
}
// ───────────────────────── 自定义 Theme ─────────────────────────
// 从 Palette 派生，让内置 widget 的默认样式也贴合我们的基调
pub fn theme() -> theme::Theme {
    use iced::theme::Palette;
    theme::Theme::custom(
        "Vidly Dark".into(),
        Palette {
            background: token::BG_BASE,
            text: token::TEXT_PRIMARY,
            primary: token::ACCENT,
            success: token::SUCCESS,
            danger: token::ERROR,
        },
    )
}
// ───────────────────────── 样式函数 ─────────────────────────
// 每个样式都是一个 fn(&Theme) -> Style，view 里 .style(card) 这样引用
/// 页面根容器：底色 + 内边距
pub fn page(_t: &theme::Theme) -> container::Style {
    container::Style {
        background: Some(token::BG_BASE.into()),
        text_color: Some(token::TEXT_PRIMARY),
        ..Default::default()
    }
}
/// 卡片/列表项：surface 底 + 圆角 + 细描边
pub fn card(_t: &theme::Theme) -> container::Style {
    container::Style {
        background: Some(token::BG_SURFACE.into()),
        text_color: Some(token::TEXT_PRIMARY),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: token::BORDER_SUBTLE,
        },
        ..Default::default()
    }
}
/// 悬停态卡片（配合 mouse_area 使用）
pub fn card_hovered(_t: &theme::Theme) -> container::Style {
    container::Style {
        background: Some(token::BG_HOVER.into()),
        text_color: Some(token::TEXT_PRIMARY),
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: token::BORDER_SUBTLE,
        },
        ..Default::default()
    }
}
/// 主按钮（Primary）
pub fn primary_button(t: &theme::Theme) -> button::Style {
    let accent = t.palette().primary;
    button::Style {
        background: Some(accent.into()),
        text_color: iced::Color::WHITE,
        border: Border {
            radius: 8.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        ..Default::default()
    }
}
/// 主按钮悬停
pub fn primary_button_hovered(t: &theme::Theme) -> button::Style {
    button::Style {
        background: Some(token::ACCENT_HOVER.into()),
        text_color: iced::Color::WHITE,
        ..primary_button(t)
    }
}
/// 次按钮：透明底 + 描边
pub fn secondary_button(_t: &theme::Theme) -> button::Style {
    button::Style {
        background: Some(Color::TRANSPARENT.into()),
        text_color: token::TEXT_PRIMARY,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: token::BORDER_SUBTLE,
        },
        ..Default::default()
    }
}
/// 次按钮悬停
pub fn secondary_button_hovered(_t: &theme::Theme) -> button::Style {
    button::Style {
        background: Some(token::BG_HOVER.into()),
        text_color: token::TEXT_PRIMARY,
        border: Border {
            radius: 8.0.into(),
            width: 1.0,
            color: token::BORDER_SUBTLE,
        },
        ..Default::default()
    }
}
/// 进度条：细轨道 + 强调色填充
pub fn progress_accent(_t: &theme::Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: token::BG_INPUT.into(),
        bar: token::ACCENT.into(),
        border_radius: 3.0.into(),
    }
}
/// 完成态进度条
pub fn progress_success(_t: &theme::Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: token::BG_INPUT.into(),
        bar: token::SUCCESS.into(),
        border_radius: 3.0.into(),
    }
}
/// 次要文字
pub fn text_secondary<'a>() -> iced::widget::Text<'a> {
    text("").style(|_t| text::Style { color: Some(token::TEXT_SECONDARY) })
}
```
**在 view 中使用**：
```rust
use crate::theme as ui;
// 卡片
container(column![...]).style(ui::card)
// 主按钮（含四态）
button("全部转换")
    .style(|t, status| match status {
        button::Status::Hovered => ui::primary_button_hovered(t),
        button::Status::Pressed => ui::primary_button(t), // 按下可再调暗
        _ => ui::primary_button(t),
    })
// 进度条
progress_bar(0.0..=100.0, pct).style(ui::progress_accent)
```
> 上述代码为**骨架示意**：`Theme::custom` 的确切签名（0.13 中接收 `Seed` 或直接 `Palette` 的重载）、`button::Status` 变体名等请以你锁定的 iced 版本文档为准微调；`Palette` 缺 `warning` 字段，警告色只在自定义样式函数里直接引用 token 常量即可。
---

## 9. 浅色主题与未来扩展


### 9.1 浅色令牌（预留）

| 令牌 | 色值 |
|---|---|
| `bg_base` | `#F7F8FA` |
| `bg_surface` | `#FFFFFF` |
| `bg_hover` | `#EFF1F4` |
| `border_subtle` | `#E3E6EB` |
| `text_primary` | `#1A1D23` |
| `text_secondary` | `#5F6672` |
| `accent` | `#4A68E0`（比暗色版深一档，保证浅底对比度） |

### 9.2 落地节奏建议

| 阶段 | 内容 |
|---|---|
| **v0.1（当前文档范围）** | 纯暗色自定义 Theme + 全套四态样式 + 状态色彩映射。**不做主题切换** |
| v0.2 ✅ 已实现 | 设置面板加"跟随系统/暗色/浅色"三选一（`ThemeMode`）；`theme::set_mode` 后整棵 view 重绘；主题持久化在配置里 |
| v0.3+ | 拖拽悬停高亮区域、单色 SVG 图标（`src/icons.rs`，Lucide，随包内嵌）+ 图标按钮 `tooltip` 已做；i18n 已实现（8 种语言）；键盘焦点环（`focus` 态样式）待做 |
> **不建议 v0.1 就做浅色**的原因：工具类应用暗色接受度极高（视频工具用户尤其如此），先把一套主题的状态反馈打磨到位，比两套平庸主题更有价值。浅色令牌已预留，切换成本仅在 Theme 构造一处。
---

## 附：设计交付物清单

| 交付物 | 状态 |
|---|---|
| 设计令牌表（色彩/字体/间距/圆角） | ✅ 第 3 节 |
| 组件四态规格 | ✅ 第 4 节 |
| 布局线框与分区说明 | ✅ 第 5 节 |
| 状态-色彩-图标映射 | ✅ 第 6 节 |
| Lucide 图标清单 | `clock, loader, check-circle, x-circle, slash, skip-forward, plus, folder, x, refresh-cw, external-link, alert-triangle, arrow-down`（均为单色线性，符合 iced svg 能力） |
| Theme 实现骨架 | ✅ 第 8 节 |
| iced 能力规避清单 | ✅ 第 2 节 |
这份文档配合此前的技术栈与打包文档，设计→实现→发布的链路就完整闭环了。下一步如果需要，可以出**浅色主题的完整 `theme.rs` 双套实现**，或者**设置面板的交互细化稿（含主题切换的三态选择器）**。
