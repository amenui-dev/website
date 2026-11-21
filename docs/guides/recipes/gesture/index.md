# Gestures: a bottom sheet and a floating box



Both widgets keep their physics in the state (`amenui::gesture`): a drag is reported as `SheetMsg::Drag` / `FloatMsg::Drag`, the update hands it to `Sheet::update` / `Floating::update` with the time, and the tick steps the springs while `moving()` — so a still page draws nothing. Letting go hands the finger's velocity to the spring; where it comes to rest is decided by where that velocity would carry it (`gesture::project`), not by where the finger let go. Past a limit the drag is rubber-banded (`gesture::RubberBand`, 0.55).

The code is `examples/gesture.rs`, compiled into the library as `amenui::recipes::gesture`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a bottom sheet and a floating box driven by springs (docs/recipes/gesture.md). The physics is
//! state: the widgets report drags, the update hands them to `Sheet` / `Floating` with the time, and the
//! tick steps their springs only while they move, so a still page asks for no frames.
use amenui::el::{El, Paint, col, layers, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{FloatMsg, Floating, Sheet, SheetMsg, Variant, bottom_sheet, button_as, floating};

pub struct App { pub sheet: Sheet, pub pip: Floating, pub now: f32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Open, Sheet(SheetMsg), Pip(FloatMsg) }

pub fn view(s: &App) -> El<Msg> {
    let body = (0..30).fold(col().gap(10.0), |c, i| c.child(text(format!("Item {}", i + 1), 15.0, Paint::TEXT).key(format!("item{i}"))));
    // The page, the floating box and the sheet over one another in the same box.
    let page = col().key("page").fill_width().fill_height().pad(20.0, 20.0).child(button_as("open", "Open the sheet", Variant::Primary, Msg::Open));
    layers([page, floating("pip", &s.pip, text("Picture in picture", 13.0, Paint::TEXT).key("pip/label"), Msg::Pip), bottom_sheet("sheet", &s.sheet, "Sheet", body, Msg::Sheet)])
        .key("screen").width(360.0).height(600.0).clip()
}

pub fn update(s: &mut App, m: Msg) {
    match m {
        Msg::Open => s.sheet.present(s.now),
        Msg::Sheet(m) => s.sheet.update(m, s.now),
        Msg::Pip(m) => s.pip.update(m, s.now),
    }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<App, Msg> {
    let mut sheet = Sheet::default();
    sheet.height = 600.0;
    let mut pip = Floating::default();
    (pip.area, pip.size) = ((360.0, 600.0), (120.0, 72.0));
    pip.place(1);
    Ui::new(App { sheet, pip, now: 0.0 }, view, update, faces, w, h, scale)
        // Step the springs on every tick; ask for frames only while one moves.
        .on_tick(|s, t| { s.now = t; let a = s.sheet.step(t); let b = s.pip.step(t); a || b })
        .on_wake(|s, t| (s.sheet.moving() || s.pip.moving()).then_some(t))
}
```

See it:

```sh
cargo run --release --bin render -- --page recipe-gesture --events open.json --at 900 --out sheet.png --tree sheet.json
```

`open.json`: `[{"at_ms": 100, "kind": "press", "target": "open"}, {"at_ms": 120, "kind": "release", "target": "open"}]`
