# 文字

文字由 parley 排版、swash 栅格化，图集里每个像素存覆盖率和到笔画的带符号距离，所以每种材质都按字的真实轮廓画它。段落可以绕开任意形状排（`.around`）。滚动时字按整设备像素移动，不会重新取样。

挨着障碍物的一段两端对齐，所以列的边和离轮廓的间隔都是直的：剩下的宽度分到字群之间（中日韩字每个字后面，西文在空格处），每个间隙最多 `Around::justify_max` 个字号（默认 0.25）。一段西文里没有空格、并且 `Around::justify_latin_letters` 开着（默认关）时才分到字母之间。要加得更多时按多放一个字群重新断开、稍微收紧，放得下就用它，否则这一段从起点排；段落的最后一行不对齐。

```rust
use amenui::wrap::{Around, Silhouette};
let mut around = Around::new(vec![Silhouette::circle(120.0, 60.0, 40.0)], 8.0);
around.justify_max = 0.3;            // 每个间隙最多加几个字号
around.justify_latin_letters = true; // 没有空格的一段可以分到字母之间
```

`El::tracking(em)` 设字距（−0.05–1.5），由 parley 排，宽度和选区都跟着。`El::whisper()` 是小声的字：小写、字距 0.6 em、一套小衬线体（Source Serif 4 的 8 pt 光学尺寸，只有西文）。
