# 用 render 和树测试

每个示例都能不开窗口、从外部驱动：`render` 在离屏画布上打开它，在从 0 开始的固定 60 Hz 时钟上、用固定的随机种子播放一串输入事件，把最后一帧写成 PNG、把排好的树写成 JSON。同样的参数给出逐字节相同的结果，所以图和树都能在测试或评审里直接比较。

```sh
cargo run --release --bin render -- --page button --size 1280x800 --at 1000 --out out.png --tree out.json --locale en
```

## 选项

| 选项 | 作用 |
|---|---|
| `--at <ms>` | 画哪一刻，按页面自己的时钟 |
| `--events <file.json>` | 先播放的输入：`[{"at_ms": 200, "kind": "press", "target": "button/live"}]`；种类有 press、release、move、key、wheel、text |
| `--frames <from>..<to> [--fps 60]` | 画一段帧而不是一帧，写到 `dir/%05d.png` |
| `--material`、`--theme`、`--locale`、`--scale` | 用哪种材质、主题、语言和缩放画 |
| `--audit` | 测量这一帧：文字被截断、对比度、字形重叠、字太小、页面一直不停；结果从 P0（无法阅读）到 P3（备注） |
| `--strict` | 视图里的问题（重复的键……）和 P0、P1 结果都让这次运行失败 |

树里列着每个元素的键、角色、名字、状态和矩形；测试读它就能检查布局和无障碍镜像，不用看像素。在 Rust 里同样的调用是 `amenui::drive::render` 和 `amenui::drive::audit`。

网站仓库里的 `check` 在显卡上读回像素做验收（20 条，包括慢速滚动是整像素移动的图）。
