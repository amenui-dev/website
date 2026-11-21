//! dot_chart
//!
//! Purpose: A series drawn as a field of dots: each column is filled from the axis up to the curve, a
//!   dot's size and brightness saying how much of it lies under the curve; a range picked out in ink,
//!   the rest of the field dim; a marker that follows the pointer with the value under it, the dot under
//!   the pointer in the accent; control points on the axis that can be dragged (in the accent while
//!   dragged), the curve recomputed as they move.
//! Props: `key`, `chart` (`DotChart`: `knots`, `cols`, `rows`, `pitch`, `range`, `max`, `unit`),
//!   `msg`.
//! States: rest, scrubbing (a marker and a label at the column under the pointer), dragging a
//!   control point (captured until release); focused, Left / Right move the marker, Up / Down change
//!   the control point nearest to it.
//! Messages: `msg(ChartMsg)` for pointer moves, drags of control points and keys; the application
//!   calls `DotChart::update` with it.
//! Usage: `dot_chart("load", &s.chart, Msg::Chart)`
//!
//! The curve is a Catmull-Rom spline through the control points (`knots`, 0..1 of `max`), sampled once
//! a column. In column c, dot r (from the axis) covers the band [r, r + 1) / rows; how much of the band
//! lies under the curve is its coverage k: the dot's diameter is `pitch × (0.25 + 0.65 k)` and its
//! tone `0.25 + 0.75 k`, so the field's area reads as the area under the curve. Labels are in the mono
//! (pixel) face.
use super::LINE;
use crate::el::{Drag, El, Font, Key, Paint, Phase, Role, col, layers, space, text};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChartMsg { Move(Option<(f32, f32)>), Knot(usize, Drag), Key(Key) }

/// A dot chart's state and its numbers.
#[derive(Clone, Debug, PartialEq)]
pub struct DotChart {
    /// The control points, evenly along the axis, each 0..1 of `max`.
    pub knots: Vec<f32>,
    /// Columns and rows of dots, and the distance between dot centres (logical px).
    pub cols: usize,
    pub rows: usize,
    pub pitch: f32,
    /// The highlighted part of the axis (fractions 0..1, start ≤ end).
    pub range: (f32, f32),
    /// What 1.0 is, and its unit, for the labels.
    pub max: f32,
    pub unit: &'static str,
    /// The column under the marker.
    hover: Option<usize>,
    /// The dot under the pointer: its column and row (from the axis).
    under: Option<(usize, usize)>,
    /// A control point being dragged: which, and its value at the press.
    grab: Option<(usize, f32)>,
}
impl Default for DotChart {
    fn default() -> Self {
        Self { knots: vec![0.22, 0.38, 0.7, 0.52, 0.86, 0.6, 0.3], cols: 56, rows: 16, pitch: 9.0, range: (0.42, 0.68), max: 120.0, unit: "", hover: None, under: None, grab: None }
    }
}
/// Catmull-Rom through `k` at `t` (0..1 along all of them).
pub fn spline(k: &[f32], t: f32) -> f32 {
    if k.is_empty() { return 0.0; }
    if k.len() == 1 { return k[0]; }
    let x = t.clamp(0.0, 1.0) * (k.len() - 1) as f32;
    let i = (x.floor() as usize).min(k.len() - 2);
    let u = x - i as f32;
    let p = |j: isize| k[(j.clamp(0, k.len() as isize - 1)) as usize];
    let (p0, p1, p2, p3) = (p(i as isize - 1), p(i as isize), p(i as isize + 1), p(i as isize + 2));
    0.5 * (2.0 * p1 + (p2 - p0) * u + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * u * u + (3.0 * p1 - p0 - 3.0 * p2 + p3) * u * u * u)
}
impl DotChart {
    /// The field's size (logical px).
    pub fn size(&self) -> (f32, f32) { (self.cols as f32 * self.pitch, self.rows as f32 * self.pitch) }
    /// The curve in column `c` (0..1 of `max`).
    pub fn value(&self, c: usize) -> f32 { spline(&self.knots, (c as f32 + 0.5) / self.cols.max(1) as f32).clamp(0.0, 1.0) }
    /// How much of dot (c, r)'s band (r from the axis) lies under the curve.
    pub fn coverage(&self, c: usize, r: usize) -> f32 { (self.value(c) * self.rows as f32 - r as f32).clamp(0.0, 1.0) }
    /// The dot's diameter (px) and tone for a coverage.
    pub fn dot(&self, k: f32) -> (f32, f32) { (self.pitch * (0.25 + 0.65 * k), 0.25 + 0.75 * k) }
    /// The column under the marker.
    pub fn hover(&self) -> Option<usize> { self.hover }
    /// The dot under the pointer (column, row from the axis).
    pub fn under(&self) -> Option<(usize, usize)> { self.under }
    /// The control point being dragged.
    pub fn dragged(&self) -> Option<usize> { self.grab.map(|g| g.0) }
    /// Whether column `c` is in the highlighted range.
    pub fn in_range(&self, c: usize) -> bool { let f = (c as f32 + 0.5) / self.cols.max(1) as f32; f >= self.range.0 && f <= self.range.1 }
    /// The area under the curve in the range, in `max` × axis fractions (what the ink dots add up to).
    pub fn range_area(&self) -> f32 {
        let cs: Vec<usize> = (0..self.cols).filter(|&c| self.in_range(c)).collect();
        cs.iter().map(|&c| self.value(c)).sum::<f32>() / self.cols.max(1) as f32
    }
    /// Control point `i`'s x (px from the left of the field).
    pub fn knot_x(&self, i: usize) -> f32 { let (w, _) = self.size(); w * i as f32 / (self.knots.len().max(2) - 1) as f32 }
    pub fn update(&mut self, m: ChartMsg) {
        match m {
            ChartMsg::Move(p) => {
                let (w, h) = self.size();
                self.hover = p.filter(|&(x, y)| x >= 0.0 && x < w && y >= 0.0 && y <= h + 40.0).map(|(x, _)| ((x / self.pitch) as usize).min(self.cols - 1));
                self.under = p.filter(|&(x, y)| x >= 0.0 && x < w && y >= 0.0 && y < h).map(|(x, y)| (((x / self.pitch) as usize).min(self.cols - 1), (((h - y) / self.pitch) as usize).min(self.rows - 1)));
            }
            ChartMsg::Knot(i, d) => {
                if i >= self.knots.len() { return; }
                let (_, h) = self.size();
                match d.phase {
                    Phase::Start => self.grab = Some((i, self.knots[i])),
                    Phase::Move | Phase::End => {
                        let Some((_, g)) = self.grab else { return };
                        self.knots[i] = (g - d.dy / h).clamp(0.0, 1.0);
                        self.hover = Some(((self.knot_x(i) / self.pitch) as usize).min(self.cols - 1));
                        if d.phase == Phase::End { self.grab = None; }
                    }
                }
            }
            ChartMsg::Key(k) => {
                let c = self.hover.unwrap_or(self.cols / 2);
                match k {
                    Key::Left => self.hover = Some(c.saturating_sub(1)),
                    Key::Right => self.hover = Some((c + 1).min(self.cols - 1)),
                    Key::Up | Key::Down => {
                        let n = self.knots.len().max(2) - 1;
                        let i = ((c as f32 + 0.5) / self.cols as f32 * n as f32).round() as usize;
                        let d = if k == Key::Up { 1.0 } else { -1.0 } / self.rows as f32;
                        if let Some(v) = self.knots.get_mut(i) { *v = (*v + d).clamp(0.0, 1.0); }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// The chart: the dot field, the axis with its control points, the marker and its label.
pub fn dot_chart<M: Clone + 'static>(key: &str, c: &DotChart, msg: impl Fn(ChartMsg) -> M + 'static) -> El<M> {
    let msg = Rc::new(msg);
    let (w, h) = c.size();
    let p = c.pitch;
    let mut parts: Vec<El<M>> = vec![];
    for x in 0..c.cols {
        let hot = c.in_range(x);
        for r in 0..c.rows {
            let k = c.coverage(x, r);
            let (d, tone) = c.dot(k);
            // Dots over the curve are the faintest, smallest ones: the empty part of the field. In the range
            // the dots go up to the ink, outside it up to dim; the one under the pointer is the accent.
            let paint = if c.under == Some((x, r)) { Paint::ACCENT } else if k <= 0.0 { Paint::tone(0.14) } else if hot { Paint::tone(tone) } else { Paint::tone(tone * Paint::DIM.tone) };
            let (cx, cy) = (x as f32 * p + p * 0.5, h - (r as f32 * p + p * 0.5));
            parts.push(space(d, d).radius(d * 0.5).fill(paint).shift((cx - d * 0.5).round(), (cy - d * 0.5).round()).key(format!("{key}/d{x}.{r}")));
        }
    }
    // The axis and the range under it.
    parts.push(space(w, LINE).fill(Paint::DIM).shift(0.0, h + 4.0).key(format!("{key}/axis")));
    parts.push(space((c.range.1 - c.range.0) * w, 3.0).fill(Paint::INK).shift(c.range.0 * w, h + 3.0).key(format!("{key}/range")));
    // Control points: a small handle on the axis per knot, dragged up and down.
    for i in 0..c.knots.len() {
        let x = c.knot_x(i);
        let m = msg.clone();
        // Ink at rest; the accent while it is dragged.
        let held = c.dragged() == Some(i);
        let pen = if held { Paint::ACCENT } else { Paint::INK };
        parts.push(col().key(format!("{key}/knot{i}")).width(14.0).height(14.0).radius(7.0).fill(Paint::GROUND).border(1.5, pen)
            .shift(x - 7.0, h + 10.0).cursor(crate::el::Cursor::RowResize).role(Role::Slider).label(format!("{i}"))
            .value_range(c.knots[i] * c.max, 0.0, c.max).focusable(false)
            .on_drag_with(move |d| Some(m(ChartMsg::Knot(i, d)))));
        // Its height on the curve, as a tick.
        let y = h - c.knots[i] * h;
        parts.push(space(5.0, 5.0).radius(1.0).fill(pen).shift(x - 2.5, y - 2.5).key(format!("{key}/kdot{i}")));
    }
    // The marker and its label.
    if let Some(x) = c.hover {
        let mx = x as f32 * p + p * 0.5;
        let v = c.value(x) * c.max;
        parts.push(space(LINE, h).fill(Paint::INK).shift(mx.round(), 0.0).key(format!("{key}/marker")));
        let label = format!("{v:.0}{}", c.unit);
        let lx = if mx > w - 60.0 { mx - 54.0 } else { mx + 6.0 };
        parts.push(text(label, 11.0, Paint::INK).font(Font::Mono).key(format!("{key}/value")).recompose(false).shift(lx, 2.0));
    }
    let (a, b) = (msg.clone(), msg);
    layers(parts).key(key.to_owned()).width(w).height(h + 28.0).focusable(true).role(Role::Img).label(crate::i18n::tr("widget.chart.label"))
        .on_move(move |p| Some(a(ChartMsg::Move(p))))
        .on_key(move |k, _| matches!(k, Key::Left | Key::Right | Key::Up | Key::Down).then(|| b(ChartMsg::Key(k))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn the_field_is_the_area_under_the_curve() {
        let c = DotChart::default();
        // A column's coverages add up to its value in rows.
        for x in [0, 10, 30, 55] {
            let sum: f32 = (0..c.rows).map(|r| c.coverage(x, r)).sum();
            assert!((sum - c.value(x) * c.rows as f32).abs() < 1e-3);
        }
        // Bigger and brighter with more coverage.
        assert!(c.dot(1.0).0 > c.dot(0.5).0 && c.dot(1.0).1 > c.dot(0.5).1);
        // The spline passes through the knots.
        assert!((spline(&c.knots, 0.0) - c.knots[0]).abs() < 1e-5 && (spline(&c.knots, 1.0) - c.knots[6]).abs() < 1e-5);
        assert!((spline(&c.knots, 2.0 / 6.0) - c.knots[2]).abs() < 1e-5);
    }
    #[test]
    fn dragging_a_knot_recomputes_the_curve() {
        let mut c = DotChart::default();
        let before = c.value(28);
        let d = |phase, dy| ChartMsg::Knot(3, Drag { phase, x: 0.0, y: 0.0, w: 14.0, h: 14.0, dx: 0.0, dy, t: 0.0 });
        c.update(d(Phase::Start, 0.0));
        c.update(d(Phase::Move, -60.0));
        assert!(c.value(28) > before, "dragged up, the curve rose there");
        assert!((c.knots[3] - (0.52 + 60.0 / c.size().1)).abs() < 1e-4);
        c.update(d(Phase::End, 1e5));
        assert_eq!(c.knots[3], 0.0);
        c.update(ChartMsg::Move(Some((100.0, 20.0))));
        assert_eq!(c.hover(), Some(11));
        c.update(ChartMsg::Move(None));
        assert_eq!(c.hover(), None);
    }
    #[test]
    fn the_range_is_ink_the_rest_dim_and_the_accent_only_under_the_hand() {
        let mut c = DotChart::default();
        // Each element's fill by its key, and its border's paint by its key and `#border`.
        fn walk<M>(e: &El<M>, out: &mut std::collections::HashMap<String, Paint>) {
            if let Some(f) = e.fill { out.insert(e.key.clone(), f); }
            if let Some(b) = e.border { out.insert(format!("{}#border", e.key), b.1); }
            for ch in &e.children { walk(ch, out); }
        }
        let fills = |c: &DotChart| { let mut out = std::collections::HashMap::new(); walk(&dot_chart("c", c, |m| m), &mut out); out };
        let accents = |f: &std::collections::HashMap<String, Paint>| f.iter().filter(|(_, p)| p.accent > 0.0).map(|(k, _)| k.clone()).collect::<Vec<_>>();
        let f = fills(&c);
        assert!(accents(&f).is_empty(), "no accent at rest: {:?}", accents(&f));
        assert_eq!(f["c/range"], Paint::INK);
        // The fullest dot in the range is ink; outside it no dot is past dim.
        let inside = (0..c.cols).find(|&x| c.in_range(x)).unwrap();
        assert_eq!(f[&format!("c/d{inside}.0")].tone, c.dot(c.coverage(inside, 0)).1);
        assert!((0..c.cols).filter(|&x| !c.in_range(x)).all(|x| (0..c.rows).all(|r| f[&format!("c/d{x}.{r}")].tone <= Paint::DIM.tone + 1e-6)));
        // The pointer over a dot: that dot alone is the accent.
        let (_, h) = c.size();
        c.update(ChartMsg::Move(Some((c.pitch * 3.5, h - c.pitch * 0.5))));
        assert_eq!(c.under(), Some((3, 0)));
        assert_eq!(accents(&fills(&c)), vec!["c/d3.0".to_owned()]);
        c.update(ChartMsg::Move(None));
        // A control point dragged: it and its tick are the accent until the release.
        let d = |phase| ChartMsg::Knot(2, Drag { phase, x: 0.0, y: 0.0, w: 14.0, h: 14.0, dx: 0.0, dy: -4.0, t: 0.0 });
        c.update(d(Phase::Start));
        let mut a = accents(&fills(&c));
        a.sort();
        assert_eq!(a, vec!["c/kdot2".to_owned(), "c/knot2#border".to_owned()]);
        c.update(d(Phase::End));
        assert!(accents(&fills(&c)).is_empty());
    }
}
