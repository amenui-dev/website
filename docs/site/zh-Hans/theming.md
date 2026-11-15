# 主题与深色模式

主题是一组颜色，加上显示、正文、等宽三种用途的字体（`style::THEMES`）。颜色在视图里写成名字而不是数值：`Paint::INK`、`TEXT`、`DIM`、`FAINT`、`GROUND`、`ACCENT`，以及色相 `Paint::hue(i)`，所以同一个视图在每个主题里都对。深色和浅色是同一个视图的两个主题：用顶栏的“主题”菜单、`Msg::Theme`、渲染工具的 `theme`，或者 `<lab-view theme="dark">` 切换。

## 主题里有什么

| 字段 | 意思 |
|---|---|
| `ground`、`ink`、`accent` | 页面的底色、最深的墨色、唯一的强调色。 |
| `hues` | 用于色块和图表的其他颜色。 |
| `display`、`body`、`mono` | 每种用途的字体（`text::SANS`、`PIXEL`、`MONO`、`GEIST_PIXEL`）。 |
| `sizes` | 字号阶：显示、标题、正文、小字。 |

## 克制

一页只用一个强调色。深色主题最好是接近黑的底色加一个饱和的强调色；读数和影片的场记板就是这个样子。
