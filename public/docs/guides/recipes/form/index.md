# A form



A text field, a number control, a switch and a submit button. The state holds every value; the widgets send a message for each edit and `update` is the only place values change. The number control sends `Edit::Draft` while you type, `Edit::Set` on Enter or a step, `Edit::Cancel` on Esc.

The code is `examples/form.rs`, compiled into the library as `amenui::recipes::form`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a form (docs/recipes/form.md). A text field, a number, a switch and a submit button; the
//! state holds every value, `update` is the only place they change.
use amenui::el::{El, Paint, col, row, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Edit, Spec, TextEdit, Variant, button_as, input, number, switch};

pub struct Form { pub name: TextEdit, pub copies: f32, pub draft: Option<String>, pub notify: bool, pub sent: u32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Name(TextEdit), Copies(Edit), Notify, Send }

const COPIES: Spec = Spec { min: 1.0, max: 99.0, step: 1.0, decimals: 0, unit: "" };

pub fn view(s: &Form) -> El<Msg> {
    col().key("form").pad(32.0, 28.0).gap(16.0)
        .child(text("Order", 22.0, Paint::INK))
        .child(input("form/name", &s.name, "Recipient", 320.0, 40, Msg::Name))
        .child(number("form/copies", "Copies", s.copies, COPIES, s.draft.as_deref(), Msg::Copies).width(320.0))
        .child(switch("form/notify", "Notify me when it ships", s.notify, None, Msg::Notify).width(320.0))
        .child(row().gap(12.0)
            .child(button_as("form/send", "Send", Variant::Primary, Msg::Send))
            .child(text(format!("Sent: {}", s.sent), 14.0, Paint::DIM).key("form/sent")))
}

pub fn update(s: &mut Form, m: Msg) {
    match m {
        Msg::Name(t) => s.name = t,
        Msg::Copies(Edit::Draft(d)) => s.draft = Some(d),
        Msg::Copies(Edit::Set(v)) => { s.copies = v; s.draft = None; }
        Msg::Copies(Edit::Cancel) => s.draft = None,
        Msg::Notify => s.notify = !s.notify,
        Msg::Send => s.sent += 1,
    }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<Form, Msg> {
    let state = Form { name: TextEdit::new("Lin"), copies: 2.0, draft: None, notify: true, sent: 0 };
    Ui::new(state, view, update, faces, w, h, scale)
}
```

See it:

```sh
cargo run --release --bin render --page recipe-form --tree form.json
```
