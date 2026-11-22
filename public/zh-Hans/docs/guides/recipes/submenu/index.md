# 带子菜单和安全区的菜单



`widgets::nested_menu` keeps its open path in a `Menu` in the state. The pointer's moves over it arrive as `MenuMsg::Move` (`El::on_move`); moving toward the open submenu inside the triangle from the pointer's previous position to the submenu's two near corners does not switch rows — each such move waits `close_delay_ms` (200) again — while outside it the row under the pointer takes over after `submenu_delay_ms` (120). `show_safe_zone` draws the triangle through the annotation layer (`El::guide`). Up / Down / Right / Left / Enter / Esc work when the menu has focus.

The code is `examples/submenu.rs`, compiled into the library as `amenui::recipes::submenu`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a menu with submenus and a safe zone (docs/recipes/submenu.md). The menu's open path and its
//! pending switch are state; the pointer's moves over it are messages (`El::on_move`), and the tick only
//! runs while a switch waits for its time (`Menu::due`).
use amenui::el::{El, Paint, col, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Menu, MenuItem, MenuMsg, nested_menu};

pub struct App { pub menu: Menu, pub chosen: String, pub now: f32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Menu(MenuMsg) }

pub fn view(s: &App) -> El<Msg> {
    col().key("page").pad(24.0, 24.0).gap(16.0)
        .child(nested_menu("menu", &s.menu, Msg::Menu))
        .child(text(format!("Chosen: {}", s.chosen), 14.0, Paint::DIM).key("chosen"))
}

pub fn update(s: &mut App, m: Msg) {
    let Msg::Menu(m) = m;
    if let Some(path) = s.menu.update(m, s.now) {
        let mut items: &[MenuItem] = &s.menu.items;
        let mut names = vec![];
        for i in path { if let Some(it) = items.get(i) { names.push(it.label.clone()); items = &it.items; } }
        s.chosen = names.join(" › ");
    }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<App, Msg> {
    let sub = |name: &str, n: usize| MenuItem::sub(name, (1..=n).map(|i| MenuItem::leaf(format!("{name} {i}"))).collect());
    let mut menu = Menu::new(vec![sub("New", 4), sub("Open recent", 5), sub("Arrange", 3), MenuItem::leaf("Close")]);
    menu.show_safe_zone = true;
    Ui::new(App { menu, chosen: "—".into(), now: 0.0 }, view, update, faces, w, h, scale)
        .on_tick(|s, t| { s.now = t; s.menu.step(t) })
        // No frames while nothing waits; a waiting switch wakes the page at its time.
        .on_wake(|s, _| s.menu.due())
}
```

See it:

```sh
cargo run --release --bin render -- --page recipe-submenu --events open.json --at 600 --out menu.png --tree menu.json
```

`open.json`: `[{"at_ms": 100, "kind": "press", "target": "menu/m0/r0"}, {"at_ms": 120, "kind": "release", "target": "menu/m0/r0"}]`
