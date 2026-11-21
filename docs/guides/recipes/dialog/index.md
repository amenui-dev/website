# A dialog



Declare the dialog next to the button that opens it; when `open` is true the runtime lifts it over the window, moves focus into it, keeps Tab inside, holds the page beneath at 0.35 looseness and sends the close message on Esc.

The code is `examples/dialog.rs`, compiled into the library as `amenui::recipes::dialog`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a dialog (docs/recipes/dialog.md). Declared next to the button that opens it; the runtime lays
//! it over the window, keeps focus inside, and Esc or Close send the close message.
use amenui::el::{El, Paint, col, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{TextEdit, Variant, button_as, dialog, input};

pub struct Rename { pub open: bool, pub name: TextEdit, pub saved: String }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Open, Close, Name(TextEdit), Save }

pub fn view(s: &Rename) -> El<Msg> {
    let body = col().gap(14.0)
        .child(input("rename/name", &s.name, "Name", 296.0, 40, Msg::Name))
        .child(button_as("rename/save", "Save", Variant::Primary, Msg::Save));
    col().key("page").pad(32.0, 28.0).gap(14.0)
        .child(text(format!("Current name: {}", s.saved), 16.0, Paint::TEXT).key("page/name"))
        .child(button_as("page/open", "Rename…", Variant::Default, Msg::Open))
        .child(dialog("rename", s.open, "Rename", 340.0, 0.35, Msg::Close, body))
}

pub fn update(s: &mut Rename, m: Msg) {
    match m {
        Msg::Open => s.open = true,
        Msg::Close => s.open = false,
        Msg::Name(t) => s.name = t,
        Msg::Save => { s.saved = s.name.text.clone(); s.open = false; }
    }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Rename, Msg> {
    Ui::new(Rename { open: true, name: TextEdit::new("Studio 4"), saved: "Studio 4".into() }, view, update, faces, w, h, scale)
}
```

See it:

```sh
cargo run --release --bin render --page recipe-dialog
```
