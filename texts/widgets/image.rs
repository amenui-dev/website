//! image
//!
//! Purpose: A picture at its aspect ratio, with an optional caption.
//! Props: `key`, `id` (picture id), `ratio` (height / width), `slot` (`Markr` set for its marks),
//!   `caption`.
//! States: rest; loosens into its marks under fronts and the lens.
//! Messages: none.
//! Usage: `image("sky", SKY, 0.6, 0, Some("Sky"))`
//!
//! A picture at its own aspect ratio, filling the width it is given, with an optional caption. `slot`
//! picks which of the eight `Markr` sets it turns into marks with when it loosens.
use super::LABEL;
use crate::el::{El, Paint, Role, col, picture, text};

pub fn image<M>(key: &str, id: u32, ratio: f32, slot: u8, caption: Option<&str>) -> El<M> {
    let mut c = col().key(key).gap(8.0).child(picture(id, ratio).key(format!("{key}/picture")).markr(slot).role(Role::Img).label(caption.map_or_else(|| crate::i18n::tr("widget.image.alt"), str::to_owned)));
    if let Some(t) = caption { c = c.child(text(t, LABEL - 1.0, Paint::DIM)); }
    c
}
