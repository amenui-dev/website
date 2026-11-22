# A custom material on one element



`El::material(id)` draws a subtree with another material whatever the global one is. The card is marked as decoration (`respond(Loosen)`, `legible(false)`) so a front shows the material, and the button sends its front only over the card (`Scope::Container(Some("card"))`).

The code is `examples/material.rs`, compiled into the library as `amenui::recipes::material`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: another material on one element (docs/recipes/material.md). The page draws with the look's
//! material; the card says `.material(..)` and draws with halftone. Its button sends fronts over the
//! card only, and the card is decoration (`Loosen`, not legible), so the front shows the material.
use amenui::el::{El, Paint, Response, Scope, col, text};
use amenui::style::Material;
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Variant, button_as, card};

pub struct Page { pub pressed: u32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Ripple }

pub fn view(s: &Page) -> El<Msg> {
    let body = text("This card is drawn in halftone; the rest of the page uses the global material.", 15.0, Paint::TEXT);
    col().key("page").pad(32.0, 28.0).gap(16.0)
        .child(text("Text on the page: the global material", 15.0, Paint::DIM))
        .child(card("card", Some("Halftone"), body).width(360.0).material(Material::Halftonr.id()).respond(Response::Loosen).legible(false))
        .child(button_as("ripple", format!("Ripple · {}", s.pressed).as_str(), Variant::Default, Msg::Ripple).ripplr(Scope::Container(Some("card".into()))))
}

pub fn update(s: &mut Page, m: Msg) {
    match m { Msg::Ripple => s.pressed += 1 }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Page, Msg> {
    Ui::new(Page { pressed: 0 }, view, update, faces, w, h, scale)
}
```

See it:

```sh
cargo run --release --bin render --page recipe-material --material plain
```
