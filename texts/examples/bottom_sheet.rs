//! bottom_sheet: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::{El, Paint, col, layers, text};
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Sheet(SheetMsg) }

/// 停在较低的一档 · Resting on the lower detent
pub fn ex_0() -> El<Msg> {
    let sheet = Sheet::resting(260.0, 0);
    let body = col().gap(8.0).child(text("A line of content", 14.0, Paint::TEXT)).child(text("Another line", 14.0, Paint::TEXT));
    layers([col().fill_width().fill_height(), bottom_sheet("bs1", &sheet, "Share to", body, Msg::Sheet)])
        .width(260.0).height(260.0).radius(16.0).border(1.0, Paint::DIM).clip()
}
/// 拉到最高一档 · At the top detent
pub fn ex_1() -> El<Msg> {
    let mut sheet = Sheet::default();
    sheet.detents = vec![0.3, 0.8];
    let sheet = sheet.rest_on(260.0, 1);
    layers([col().fill_width().fill_height(), bottom_sheet("bs2", &sheet, "Filters", text("Drag the handle or the body", 14.0, Paint::DIM), Msg::Sheet)])
        .width(260.0).height(260.0).radius(16.0).border(1.0, Paint::DIM).clip()
}
