# 命令行

组件像 shadcn/ui 那样以源码分发：把组件的文件拷进你的项目，在那里改。`amenui add` 负责拷。

```sh
cargo run --release --bin amenui -- add dialog --to ../my-app
cargo run --release --bin amenui -- add button number input --to ../my-app --force
```

- `add <名字>…` 接一个或几个 `registry.json` 里的名字（`button`、`scroll-area`、`context_menu`……）。
- 它按清单解析每个组件依赖的东西（每个组件都要 `base`，即 `src/widgets/mod.rs` 里共用的尺寸；对话框要按钮），把每个文件拷到目标下同样的路径。
- 已经有的文件不动；`--force` 覆盖。
- `--to <目录>` 是项目（不写时是当前目录）。
- 名字写错是一个错误，并说出最接近的是哪个。

逻辑在 `amenui::add`（`src/add.rs`）；`tests/add.rs` 把它跑进一个临时目录。每个组件页都有它自己的 `add` 那一行和它拷贝的文件。
