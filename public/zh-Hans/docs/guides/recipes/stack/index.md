# 可扇形展开的卡片堆



`widgets::card_stack` draws a `Stack` as a 3D element, one card each. Hovering or pressing a card (the element's `Pick`) fans the cards beneath it about their bottom-left corners: card n below turns `angle(n) = angle0 × falloff^n` (42° × 0.7^n) and starts `delay(n) = delay_ms × n` (28 ms) later, each on its own spring. The look follows the material (`StackLook::of`): flat cards, thick cards with visible edges, or cards drawn in marks.

The code is `examples/stack.rs`, compiled into the library as `amenui::recipes::stack`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a stack of cards that fans under the pointer (docs/recipes/stack.md). The angles are state:
//! the 3D element reports which card the pointer is on (`Pick`), `Stack::update` schedules each card
//! beneath with its own delay, and the tick steps their springs only while one moves.
use amenui::el::{El, Font, Paint, col, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Stack, StackMsg, card_stack};

pub struct App { pub stack: Stack, pub now: f32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Stack(StackMsg) }

pub fn view(s: &App) -> El<Msg> {
    let st = &s.stack;
    col().key("page").pad(24.0, 24.0).gap(12.0)
        .child(text("A stack of cards", 20.0, Paint::INK).font(Font::Display).key("title"))
        .child(card_stack("deck", st, 480.0, 320.0, Msg::Stack))
        // The formula, from the same fields.
        .child(text(format!("angle(n) = {:.0}° × {:.2}^n · delay(n) = {:.0} ms × n", st.angle0, st.falloff, st.delay_ms), 11.5, Paint::DIM).font(Font::Mono).key("formula"))
}

pub fn update(s: &mut App, m: Msg) {
    let Msg::Stack(m) = m;
    s.stack.update(m, s.now);
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<App, Msg> {
    Ui::new(App { stack: Stack::default(), now: 0.0 }, view, update, faces, w, h, scale)
        .on_tick(|s, t| { s.now = t; s.stack.step(t) })
        .on_wake(|s, t| s.stack.moving().then_some(t))
}
```

See it:

```sh
cargo run --release --bin render -- --page recipe-stack --events press.json --at 900 --out stack.png
```

`press.json`: `[{"at_ms": 100, "kind": "press", "target": "deck"}, {"at_ms": 120, "kind": "release", "target": "deck"}]`
