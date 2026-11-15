# Forms

A form is state and messages, like everything else: the state holds every value, each widget sends a message for each edit, and `update` is the only place a value changes. Text fields keep a `TextEdit` in the state; number controls send `Edit::Draft` while you type, `Edit::Set` on Enter or a step and `Edit::Cancel` on Esc, so the state keeps the draft apart from the value; switches and checkboxes send the new value.

Validation is a function of the state: compute what is wrong in `view` and show it next to the field; disable the submit button with `.disabled(true)` while something is wrong. Nothing is checked behind your back.

Labels go with their controls (`number` and `input` take one), so the accessibility mirror names each control. Tab moves through the fields in tree order; Enter on the button submits.

This is the form recipe, `examples/form.rs`, compiled into the library as `lab::recipes::form`:

```rust
//! Recipe: a form (docs/recipes/form.md). A text field, a number, a switch and a submit button; the
//! state holds every value, `update` is the only place they change.
use lab::el::{El, Paint, col, row, text};
use lab::text::Face;
use lab::ui::Ui;
use lab::widgets::{Edit, Spec, TextEdit, Variant, button_as, input, number, switch};

pub struct Form { pub name: TextEdit, pub copies: f32, pub draft: Option<String>, pub notify: bool, pub sent: u32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Name(TextEdit), Copies(Edit), Notify, Send }

const COPIES: Spec = Spec { min: 1.0, max: 99.0, step: 1.0, decimals: 0, unit: "份" };

pub fn view(s: &Form) -> El<Msg> {
    col().key("form").pad(32.0, 28.0).gap(16.0)
        .child(text("订单", 22.0, Paint::INK))
        .child(input("form/name", &s.name, "收件人", 320.0, 40, Msg::Name))
        .child(number("form/copies", "份数", s.copies, COPIES, s.draft.as_deref(), Msg::Copies).width(320.0))
        .child(switch("form/notify", "发货时通知我", s.notify, None, Msg::Notify).width(320.0))
        .child(row().gap(12.0)
            .child(button_as("form/send", "提交", Variant::Accent, Msg::Send))
            .child(text(format!("已提交 {} 次", s.sent), 14.0, Paint::DIM).key("form/sent")))
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
    let state = Form { name: TextEdit::new("林"), copies: 2.0, draft: None, notify: true, sent: 0 };
    Ui::new(state, view, update, faces, w, h, scale)
}
```

Render it and read its tree:

```sh
cargo run --release --bin render -- --page recipe-form --tree form.json
```
