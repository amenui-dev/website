//! nested_menu
//!
//! Purpose: A menu whose rows can open submenus, steered by where the pointer is heading: moving toward
//!   an open submenu across its siblings does not switch rows.
//! Props: `key`, `menu` (`Menu`: `items`, `submenu_delay_ms`, `close_delay_ms`, `row_h`, `width`,
//!   `show_safe_zone`), `msg`.
//! States: closed, open (a row hot, submenus open along a path), a switch pending (after
//!   `submenu_delay_ms`, or `close_delay_ms` while the pointer is inside the safe zone); role menu,
//!   rows menuitem.
//! Messages: `msg(MenuMsg)` for pointer moves over the menu, presses on rows and keys; the
//!   application calls `Menu::update` (which returns the path of a chosen item) and `Menu::step` on
//!   every tick while `Menu::waiting()`.
//! Usage: `nested_menu("file", &s.menu, Msg::Menu)`
//!
//! The safe zone is the triangle from where the pointer was at its previous move to the two near
//! corners of the open submenu. While the pointer moves inside it, the row under it does not take over:
//! the switch waits `close_delay_ms` (re-armed by every move inside), so a diagonal path across sibling
//! rows reaches the submenu. Outside it, the row under the pointer takes over after `submenu_delay_ms`.
//! Keyboard: Up / Down move in the deepest open menu, Right or Enter opens a submenu, Left or Esc
//! closes one, Enter on a leaf chooses it. The geometry is the state's own (rows `row_h` tall, menus
//! `width` wide, each submenu beside its row), so the state can test the triangle without a layout;
//! `show_safe_zone` draws the triangle through the annotation layer (`El::guide`).
use super::{LABEL, LINE, MENU_SURFACE, RADIUS, selected_on};
use crate::el::{El, Key, Paint, Role, col, layers, row, space, text};
use std::rc::Rc;
use taffy::AlignItems;

/// One row: its label and the rows of its submenu (empty: a leaf).
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem { pub label: String, pub items: Vec<MenuItem> }
impl MenuItem {
    pub fn leaf(label: impl Into<String>) -> Self { Self { label: label.into(), items: vec![] } }
    pub fn sub(label: impl Into<String>, items: Vec<MenuItem>) -> Self { Self { label: label.into(), items } }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuMsg { Move(Option<(f32, f32)>), Press(usize, usize), Key(Key) }

/// A menu's state and its numbers. Coordinates are logical px in the menu's own box.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu {
    pub items: Vec<MenuItem>,
    /// How long the pointer rests on a row before its submenu opens or it takes over from an open one
    /// (default 120 ms), and how long a move inside the safe zone holds the open submenu (default 200 ms).
    pub submenu_delay_ms: f32,
    pub close_delay_ms: f32,
    /// Row height, menu width, padding inside a menu, and how far a submenu overlaps its parent (px).
    pub row_h: f32,
    pub width: f32,
    pub pad: f32,
    pub overlap: f32,
    /// Draw the safe-zone triangle (a debug overlay, through the annotation layer).
    pub show_safe_zone: bool,
    /// The open path: `path[k]` is the hot row of menu `k`; menu `k + 1` is its submenu when it has one
    /// and `open` is longer than `k + 1`.
    path: Vec<usize>,
    open: usize,
    /// A switch waiting: to row (level, row) at time `due` (s).
    pending: Option<(usize, usize, f32)>,
    last: Option<(f32, f32)>,
    zone: Option<[(f32, f32); 3]>,
    now: f32,
}
impl Default for Menu {
    fn default() -> Self {
        Self { items: vec![], submenu_delay_ms: 120.0, close_delay_ms: 200.0, row_h: 30.0, width: 200.0, pad: 4.0, overlap: 4.0, show_safe_zone: false,
               path: vec![], open: 1, pending: None, last: None, zone: None, now: 0.0 }
    }
}
impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self { Self { items, ..Self::default() } }
    /// The rows of menu `level` along the open path.
    pub fn rows(&self, level: usize) -> &[MenuItem] {
        let mut it: &[MenuItem] = &self.items;
        for k in 0..level { match self.path.get(k).and_then(|&r| it.get(r)) { Some(m) => it = &m.items, None => return &[] } }
        it
    }
    /// How many menus are open (the root is 1).
    pub fn depth(&self) -> usize { self.open }
    /// The hot row of menu `level`.
    pub fn hot(&self, level: usize) -> Option<usize> { self.path.get(level).copied() }
    /// Menu `level`'s box (x, y, w, h).
    pub fn panel(&self, level: usize) -> (f32, f32, f32, f32) {
        let (mut x, mut y) = (0.0, 0.0);
        for k in 0..level {
            let r = self.path.get(k).copied().unwrap_or(0) as f32;
            x += self.width - self.overlap;
            y += r * self.row_h;
        }
        (x, y, self.width, self.rows(level).len() as f32 * self.row_h + 2.0 * self.pad)
    }
    /// The whole area the open menus cover (w, h), for the box they are drawn in.
    pub fn extent(&self) -> (f32, f32) {
        (0..self.open).map(|l| self.panel(l)).fold((0.0f32, 0.0f32), |(w, h), (x, y, pw, ph)| (w.max(x + pw), h.max(y + ph)))
    }
    /// Which (menu, row) is at (x, y), the deepest first.
    pub fn at(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        (0..self.open).rev().find_map(|l| {
            let (px, py, w, h) = self.panel(l);
            if x < px || x >= px + w || y < py || y >= py + h { return None; }
            let r = ((y - py - self.pad) / self.row_h).floor();
            (r >= 0.0 && (r as usize) < self.rows(l).len()).then_some((l, r as usize))
        })
    }
    /// The safe zone toward the deepest open submenu from `from`: the pointer's previous position and
    /// the submenu's two near corners.
    pub fn safe_zone(&self, from: (f32, f32)) -> Option<[(f32, f32); 3]> {
        if self.open < 2 { return None; }
        let (x, y, w, h) = self.panel(self.open - 1);
        let near = if from.0 <= x { x } else { x + w };
        Some([from, (near, y), (near, y + h)])
    }
    /// The triangle as it stands (for the overlay), if any.
    pub fn zone(&self) -> Option<[(f32, f32); 3]> { self.zone }
    /// A switch is waiting (call `step` on every tick, and ask for frames, while it is).
    pub fn waiting(&self) -> bool { self.pending.is_some() }
    /// When the waiting switch is due (s).
    pub fn due(&self) -> Option<f32> { self.pending.map(|p| p.2) }
    fn hot_at(&mut self, l: usize, r: usize) {
        self.path.truncate(l);
        self.path.push(r);
        self.open = l + 1;
    }
    fn open_sub(&mut self, l: usize) {
        let has = self.path.get(l).and_then(|&r| self.rows(l).get(r)).is_some_and(|m| !m.items.is_empty());
        if has { self.open = l + 2; }
    }
    /// Make row (l, r) the hot one now, opening its submenu.
    fn switch(&mut self, l: usize, r: usize) { self.hot_at(l, r); self.open_sub(l); self.pending = None; }
    /// The path of the leaf at (l, r).
    fn chosen(&self, l: usize, r: usize) -> Vec<usize> { let mut p = self.path[..l.min(self.path.len())].to_vec(); p.push(r); p }
    /// Returns the path of a chosen leaf.
    pub fn update(&mut self, m: MenuMsg, now: f32) -> Option<Vec<usize>> {
        self.now = now;
        match m {
            MenuMsg::Move(None) => { self.last = None; self.zone = None; if let Some((l, _, _)) = self.pending && l + 1 >= self.open { self.pending = None; } }
            MenuMsg::Move(Some(p)) => {
                let prev = self.last.replace(p);
                self.zone = prev.and_then(|q| self.safe_zone(q));
                let (l, r) = self.at(p.0, p.1)?;
                // On the open submenu (or deeper): nothing waits.
                if l + 1 >= self.open && self.hot(l) == Some(r) { self.pending = None; return None; }
                if l + 1 < self.open {
                    // A row of a menu whose submenu is open.
                    if self.hot(l) == Some(r) { self.pending = None; self.open = self.open.max(l + 2); return None; }
                    let inside = self.zone.is_some_and(|z| in_triangle(p, z)) && l + 2 == self.open;
                    let wait = if inside { self.close_delay_ms } else { self.submenu_delay_ms };
                    // Inside the zone every move pushes the switch back; outside a waiting switch keeps its time.
                    let due = match self.pending { Some((pl, pr, d)) if (pl, pr) == (l, r) && !inside => d, _ => now + wait / 1000.0 };
                    self.pending = Some((l, r, due));
                } else {
                    // The deepest menu: the row is hot at once; its submenu opens after the delay.
                    self.hot_at(l, r);
                    let has = !self.rows(l)[r].items.is_empty();
                    self.pending = has.then(|| (l, r, now + self.submenu_delay_ms / 1000.0));
                }
            }
            MenuMsg::Press(l, r) => {
                if l >= self.open || r >= self.rows(l).len() { return None; }
                if self.rows(l)[r].items.is_empty() { let p = self.chosen(l, r); return Some(p); }
                self.switch(l, r);
            }
            MenuMsg::Key(k) => {
                let l = self.open - 1;
                let n = self.rows(l).len();
                match k {
                    Key::Down | Key::Up if n > 0 => {
                        let cur = if self.path.len() > l { self.path[l] } else if k == Key::Down { n - 1 } else { 0 };
                        let r = if k == Key::Down { (cur + 1) % n } else { (cur + n - 1) % n };
                        self.hot_at(l, r);
                    }
                    Key::Right | Key::Enter if self.path.len() > l => {
                        let r = self.path[l];
                        if self.rows(l)[r].items.is_empty() { return (k == Key::Enter).then(|| self.chosen(l, r)); }
                        self.open_sub(l);
                        // The submenu opens with its first row hot.
                        if self.open > l + 1 { self.path.truncate(l + 1); self.path.push(0); }
                    }
                    Key::Left | Key::Escape if l > 0 => { self.open = l; self.path.truncate(l); }
                    _ => {}
                }
                self.pending = None;
            }
        }
        None
    }
    /// Carry out a switch that is due; returns whether anything changed.
    pub fn step(&mut self, now: f32) -> bool {
        self.now = now;
        match self.pending {
            Some((l, r, due)) if now >= due => {
                if l + 1 < self.open && self.hot(l) != Some(r) || self.hot(l) == Some(r) { self.switch(l, r); } else { self.pending = None; }
                true
            }
            _ => false,
        }
    }
}

/// Whether `p` is inside triangle `t` (edges included).
pub fn in_triangle(p: (f32, f32), t: [(f32, f32); 3]) -> bool {
    let s = |a: (f32, f32), b: (f32, f32)| (b.0 - a.0) * (p.1 - a.1) - (b.1 - a.1) * (p.0 - a.0);
    let (d1, d2, d3) = (s(t[0], t[1]), s(t[1], t[2]), s(t[2], t[0]));
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

/// The open menus, each beside the row it came from, in a box `Menu::extent` big that hears the pointer.
pub fn nested_menu<M: Clone + 'static>(key: &str, m: &Menu, msg: impl Fn(MenuMsg) -> M + 'static) -> El<M> {
    let msg = Rc::new(msg);
    let mut panels: Vec<El<M>> = vec![];
    for l in 0..m.depth() {
        let (x, y, w, h) = m.panel(l);
        let mut p = col().key(format!("{key}/m{l}")).width(w).height(h).pad(m.pad, m.pad).radius(RADIUS + 2.0).fill(MENU_SURFACE).border(LINE, Paint::HAIRLINE)
            .role(Role::Menu).shift(x, y);
        for (r, it) in m.rows(l).iter().enumerate() {
            let hot = m.hot(l) == Some(r);
            let a = msg.clone();
            let mut item = row().key(format!("{key}/m{l}/r{r}")).fill_width().height(m.row_h).pad(10.0, 0.0).radius(RADIUS).align(AlignItems::CENTER).gap(8.0)
                .role(Role::MenuItem).label(it.label.clone()).focusable(false).on_press(a(MenuMsg::Press(l, r)))
                .child(text(it.label.clone(), LABEL - 1.0, if hot { Paint::INK } else { Paint::TEXT }).grow(1.0));
            // The chevron of a row with a submenu: dim does not read 3:1 on the raised panel in the light scheme.
            if !it.items.is_empty() { item = item.child(text("›", LABEL, Paint::TEXT)); }
            if hot { item = item.fill(selected_on(MENU_SURFACE)); }
            p = p.child(item);
        }
        panels.push(p);
    }
    let (w, h) = m.extent();
    let (a, b) = (msg.clone(), msg);
    let mut area = layers(panels).key(key.to_owned()).width(w.max(1.0)).height(h.max(1.0)).focusable(true).role(Role::Menu)
        .on_move(move |p| Some(a(MenuMsg::Move(p))))
        .on_key(move |k, _| matches!(k, Key::Up | Key::Down | Key::Left | Key::Right | Key::Enter | Key::Escape).then(|| b(MenuMsg::Key(k))));
    if m.show_safe_zone && let Some(z) = m.zone() { area = area.guide(z.to_vec()); }
    let _ = space::<M>(0.0, 0.0);
    area
}

#[cfg(test)]
mod tests {
    use super::*;
    fn menu() -> Menu {
        let sub = |n: &str| MenuItem::sub(n, (0..6).map(|i| MenuItem::leaf(format!("{n} {i}"))).collect());
        Menu::new(vec![sub("A"), sub("B"), sub("C"), sub("D"), MenuItem::leaf("E")])
    }
    /// Move along `pts` at 60 Hz from `t`, stepping the menu every frame.
    fn walk(m: &mut Menu, t: &mut f32, pts: &[(f32, f32)]) {
        for &p in pts { *t += 1.0 / 60.0; m.step(*t); m.update(MenuMsg::Move(Some(p)), *t); }
    }
    fn rest(m: &mut Menu, t: &mut f32, secs: f32) { let end = *t + secs; while *t < end { *t += 1.0 / 60.0; m.step(*t); } }

    #[test]
    fn a_diagonal_path_to_the_submenu_keeps_it_open() {
        let mut m = menu();
        let mut t = 0.0;
        // Rest on row A (y 4..34): its submenu opens after the delay.
        walk(&mut m, &mut t, &[(100.0, 18.0)]);
        rest(&mut m, &mut t, 0.15);
        assert_eq!((m.depth(), m.hot(0)), (2, Some(0)));
        // Head down and right toward the submenu, crossing rows B and C quickly: A stays open.
        let path: Vec<(f32, f32)> = (1..=10).map(|i| (100.0 + 9.5 * i as f32, 18.0 + 7.0 * i as f32)).collect();
        walk(&mut m, &mut t, &path);
        assert_eq!((m.depth(), m.hot(0)), (2, Some(0)), "crossing B inside the safe zone did not switch");
        // Into the submenu: its rows are hot, A still open.
        walk(&mut m, &mut t, &[(205.0, 92.0), (215.0, 95.0)]);
        assert_eq!((m.depth(), m.hot(0), m.hot(1)), (2, Some(0), Some(3)));
        rest(&mut m, &mut t, 1.0);
        assert_eq!(m.hot(0), Some(0));
    }
    #[test]
    fn a_path_outside_switches_after_the_delay() {
        let mut m = menu();
        let mut t = 0.0;
        walk(&mut m, &mut t, &[(100.0, 18.0)]);
        rest(&mut m, &mut t, 0.15);
        // Straight down onto C: outside the zone.
        walk(&mut m, &mut t, &[(100.0, 40.0), (100.0, 60.0), (100.0, 78.0)]);
        assert_eq!(m.hot(0), Some(0), "not at once");
        let d = m.submenu_delay_ms / 1000.0 + 0.02;
        rest(&mut m, &mut t, d);
        assert_eq!((m.hot(0), m.depth()), (Some(2), 2), "C took over after submenu_delay_ms");
        // Stopping inside the zone also switches, after close_delay_ms.
        walk(&mut m, &mut t, &[(120.0, 70.0), (140.0, 100.0)]);
        assert_eq!(m.hot(0), Some(2));
        let d = m.close_delay_ms / 1000.0 + 0.02;
        rest(&mut m, &mut t, d);
        assert_eq!(m.hot(0), Some(3), "resting in the zone over D: D after close_delay_ms");
    }
    #[test]
    fn keys_and_presses() {
        let mut m = menu();
        m.update(MenuMsg::Key(Key::Down), 0.0);
        assert_eq!(m.hot(0), Some(0));
        m.update(MenuMsg::Key(Key::Right), 0.0);
        assert_eq!((m.depth(), m.hot(1)), (2, Some(0)));
        m.update(MenuMsg::Key(Key::Down), 0.0);
        assert_eq!(m.update(MenuMsg::Key(Key::Enter), 0.0), Some(vec![0, 1]));
        m.update(MenuMsg::Key(Key::Left), 0.0);
        assert_eq!(m.depth(), 1);
        assert_eq!(m.update(MenuMsg::Press(0, 4), 0.0), Some(vec![4]));
        assert!(in_triangle((1.0, 1.0), [(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)]) && !in_triangle((3.0, 3.0), [(0.0, 0.0), (4.0, 0.0), (0.0, 4.0)]));
    }
}
