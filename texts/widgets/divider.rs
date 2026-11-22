//! divider
//!
//! Purpose: A thin line between groups (the separator).
//! Props: `thickness` (logical px).
//! States: none.
//! Messages: none.
//! Usage: `divider(1.0)` or `vdivider(1.0)`
//!
//! A thin line between groups: `divider` across, `vdivider` down. `thickness` is logical px.
use crate::el::{El, Paint, Role, space};

/// The separator role goes to the accessibility mirror (a separator, WAI-ARIA: not focusable).
pub fn divider<M>(thickness: f32) -> El<M> { space(0.0, thickness).fill_width().fill(Paint::FAINT).role(Role::Separator) }
pub fn vdivider<M>(thickness: f32) -> El<M> { space(thickness, 0.0).fill_height().fill(Paint::FAINT).role(Role::Separator) }
