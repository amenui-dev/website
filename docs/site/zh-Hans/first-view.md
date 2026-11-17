# 第一个视图

视图是从状态到元素树的函数，更新是从状态和消息到新状态的函数。消息是普通的值。

```rust
use amenui::el::{El, Paint, col, text};
use amenui::widgets::button;
#[derive(Clone)] enum Msg { Inc }
struct S { n: u32 }
fn view(s: &S) -> El<Msg> {
    col().pad(24.0, 24.0).gap(12.0)
        .child(text(format!("{}", s.n), 32.0, Paint::INK).key("count"))
        .child(button("inc", "加一", Msg::Inc))
}
fn update(s: &mut S, m: Msg) { match m { Msg::Inc => s.n += 1 } }
```

给要再找到、要动、带状态的元素一个稳定的 key。带 key 的文字变了会重组：变了的字的像素飞成新的字。用 `render` 画一帧、读它的树来验证（见[测试](#docs-testing)）。
