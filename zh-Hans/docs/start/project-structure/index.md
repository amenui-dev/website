# 项目结构


amenui 仓库里各部分在哪。这个 crate 是一个库加几个命令行工具；这个网站、组件展示和实验室在另一个仓库 `amenui-dev/website` 里，它像普通应用一样依赖 amenui。

## 运行时

| 位置 | 是什么 |
|---|---|
| `src/el.rs` | 元素树和它的构造函数 |
| `src/ui.rs` | 运行时：布局、输入、焦点、滚动、动画 |
| `src/app/` | 打开窗口：`native.rs`（winit，无障碍在 `access.rs`，用 AccessKit）和 `web.rs`（画布） |
| `src/style.rs`、`src/preset.rs` | 外观、主题和预设，都是数据 |
| `src/i18n.rs`、`assets/i18n/` | 多语言和消息目录 |
| `src/text.rs`、`assets/` | 文字排版和内置字体 |
| `src/tree.rs`、`src/drive.rs`、`src/audit.rs` | 排好的树作为数据、从外部驱动页面，以及页面检查 |

## 绘制

| 位置 | 是什么 |
|---|---|
| `src/scene.rs`、`src/render.rs` | 要画什么，以及用 wgpu 画它的渲染器 |
| `src/shader.wgsl` | 几遍绘制：实例、材质、标注 |
| `src/materials/*.wgsl` | 每种材质一个文件（`plain`、`dottr`、`halftonr`……） |
| `src/mesh.wgsl`、`src/particles.wgsl`、`src/sim_cs.wgsl` | 三维物体、粒子，以及移动粒子的计算着色器 |
| `src/offscreen.rs` | 不开窗口的渲染，给工具和测试用 |

## 组件和工具

| 位置 | 是什么 |
|---|---|
| `src/widgets/` | 组件，一个文件一个 |
| `registry.json` | 组件清单，`amenui add` 读它 |
| `examples/` | 示例，每个是一个完整的小应用 |
| `src/bin/` | `amenui`（把组件加进项目）、`render` 和 `sheet` |
| `web/` | 网页版的 JavaScript 部分 |
| `docs/DESIGN.md` | 设计契约 |
| `tests/` | 测试 |
