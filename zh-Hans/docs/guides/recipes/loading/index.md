# 逐步载入的列表



The rules are numbers (`widgets::Loading`): what does not depend on the data (the title, the button) draws at once; a skeleton appears only if the data takes longer than `skeleton_after_ms` (300), so a quick answer never flashes one; when the data comes, row `i` takes its place `i × swap_stagger_ms` (40) later. The skeleton is the real row with `El::skeleton()`: laid out exactly the same, its text drawn as bars as wide as each line measures and its picture as a block, so the content replaces it without anything moving (`tests/loading.rs` checks every box).

The code is `examples/loading.rs`, compiled into the library as `amenui::recipes::loading`; `tests/recipes.rs` renders it, plays its interaction, and fails if this block differs from the file.

```rust
//! Recipe: a list that loads (docs/recipes/loading.md). The title and the button draw at once; the rows
//! are the real rows all along — hidden for the first `skeleton_after_ms`, then their own skeleton
//! (`El::skeleton`), then, when the data comes, the content row by row `swap_stagger_ms` apart, in the
//! same boxes. The tick runs only at the times something changes (`Loading::next_change`).
use amenui::el::{El, Font, Paint, col, row, space, text};
use amenui::text::Face;
use amenui::ui::Ui;
use amenui::widgets::{Loading, Show, Variant, button_as};

pub struct App { pub loading: Loading, pub latency_ms: f32, pub now: f32 }

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Reload }

pub const ROWS: [(&str, &str); 5] = [
    ("Harbour in the morning", "Three photos · last week"), ("A longer set of reading notes", "2,000 words"), ("Meeting notes", "Yesterday"), ("Packing list", "Twelve items"), ("Sketch", "One picture"),
];

/// Row `i` as it is when loaded; the loading state decides how it is drawn.
pub fn row_el(i: usize) -> El<Msg> {
    let (title, meta) = ROWS[i];
    row().key(format!("row{i}")).gap(12.0).height(52.0)
        .child(space(40.0, 40.0).radius(6.0).fill([Paint::INK, Paint::DIM, Paint::TEXT][i % 3]).key(format!("row{i}/pic")))
        .child(col().gap(4.0).child(text(title, 15.0, Paint::TEXT).key(format!("row{i}/title"))).child(text(meta, 12.0, Paint::DIM).key(format!("row{i}/meta"))))
}

pub fn view(s: &App) -> El<Msg> {
    let mut list = col().key("rows").gap(8.0);
    for i in 0..ROWS.len() {
        list = list.child(match s.loading.show(i, s.now) {
            Show::Content => row_el(i),
            Show::Skeleton => row_el(i).skeleton(),
            Show::Nothing => row_el(i).hidden_until_morphed(true),
        });
    }
    col().key("page").pad(24.0, 24.0).gap(16.0)
        .child(row().gap(16.0).child(text("Recent", 22.0, Paint::INK).font(Font::Display)).child(button_as("reload", "Reload", Variant::Default, Msg::Reload)))
        .child(list)
}

pub fn update(s: &mut App, m: Msg) {
    match m { Msg::Reload => s.loading.start(s.now) }
}

pub fn ui(faces: Vec<Face>, w: u32, h: u32, scale: f32) -> Ui<App, Msg> {
    Ui::new(App { loading: Loading::default(), latency_ms: 900.0, now: 0.0 }, view, update, faces, w, h, scale)
        .on_tick(|s, t| {
            let before: Vec<Show> = (0..ROWS.len()).map(|i| s.loading.show(i, s.now)).collect();
            s.now = t;
            // The data comes `latency_ms` after the request (a stand-in for the network).
            if let Some(st) = s.loading.started() && s.loading.arrived().is_none() && t >= st + s.latency_ms / 1000.0 { s.loading.arrive(t); }
            // A new view only when a row is shown another way.
            (0..ROWS.len()).any(|i| s.loading.show(i, t) != before[i])
        })
        .on_wake(|s, t| {
            let data = s.loading.started().filter(|_| s.loading.arrived().is_none()).map(|st| st + s.latency_ms / 1000.0);
            match (s.loading.next_change(ROWS.len(), t), data) { (Some(a), Some(b)) => Some(a.min(b)), (a, b) => a.or(b) }
        })
}
```

See it:

```sh
cargo run --release --bin render -- --page recipe-loading --events reload.json --at 700 --out skeleton.png --tree skeleton.json
```

`reload.json`: `[{"at_ms": 100, "kind": "press", "target": "reload"}, {"at_ms": 120, "kind": "release", "target": "reload"}]`
