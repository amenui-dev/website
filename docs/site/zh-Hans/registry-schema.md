# 注册表结构

`registry.json` 把每个组件列成可以拷贝的源码，和 shadcn/ui 的注册表一样。`amenui add` 读它；文档里的组件页也由它生成。

| 字段 | 类型 | 意思 |
|---|---|---|
| `name` | 字符串 | 组件名，`amenui add` 用的就是它（`dialog`、`scroll_area`）。 |
| `title` | 字符串 | 中文名。 |
| `group` | 字符串 | 文档把它列在哪一组：`inputs`、`display`、`feedback`、`navigation`、`layout`、`data`。 |
| `description`、`description_en` | 字符串 | 两种语言各一句话。 |
| `functions` | 字符串数组 | 它导出的函数（`dialog`、`dialog_sized`）。 |
| `files` | 字符串数组 | 源码文件，相对仓库根目录，拷到同样的路径。 |
| `dependencies` | 字符串数组 | 它需要的其他条目（一起拷）。 |
| `response` | 字符串 | 它怎么回应波前（`nudge`、`loosen`、`ignore`）。 |
| `recompose` | 字符串 | 文字变化时哪些部分重组。 |
| `recipes` | 字符串数组 | 用到它的配方。 |

名叫 `base` 的条目是每个组件都需要的文件（元素树、运行时）。

```sh
cargo run --release --bin amenui -- add dialog --to ../my-app
```
