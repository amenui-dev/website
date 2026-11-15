# 表单

表单和别的一样，是状态和消息：状态里存着每一个值，每个控件每改一次发一条消息，`update` 是唯一改值的地方。文字字段在状态里存一个 `TextEdit`；数字控件键入时发 `Edit::Draft`，回车或步进时发 `Edit::Set`，Esc 时发 `Edit::Cancel`，所以状态把草稿和值分开存；开关和复选框发新的值。

校验是状态的函数：在 `view` 里算出哪里不对，显示在字段旁边；有不对的地方时用 `.disabled(true)` 让提交按钮不可用。没有在背后偷偷检查的东西。

标签和控件在一起（`number`、`input` 都带一个），所以无障碍镜像给每个控件一个名字。Tab 按树的顺序走过每个字段；在按钮上按回车就是提交。

这是表单配方 `examples/form.rs`，编进库里是 `lab::recipes::form`：

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

画出来、读它的树：

```sh
cargo run --release --bin render -- --page recipe-form --tree form.json
```
