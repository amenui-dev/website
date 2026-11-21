//! Widgets are plain functions that return element trees. Each one is a file so it can be copied out
//! and changed (`registry.json` at the repository root lists the files and what each depends on).
mod accordion;
mod avatar;
mod badge;
mod bottom_sheet;
mod breadcrumb;
mod button;
mod checkbox;
mod command;
mod context_menu;
mod caption;
mod card;
mod card_stack;
mod choice;
mod dialog;
mod divider;
mod dot_chart;
mod floating;
mod image;
mod input;
mod list;
mod menu;
mod nested_menu;
mod number;
mod orbit;
mod pagination;
mod popover;
mod progress;
mod radio;
mod scroll;
mod select;
mod sheet;
mod skeleton;
mod skip_link;
mod slider;
mod switch;
mod table;
mod tabs;
mod toast;
mod toggle;
mod tooltip;

pub use accordion::accordion;
pub use avatar::{SHADES, avatar, initials};
pub use badge::badge;
pub use bottom_sheet::{Sheet, SheetMsg, bottom_sheet};
pub use breadcrumb::breadcrumb;
pub use floating::{FloatMsg, Floating, floating};
pub use button::{Variant, button, button_as};
pub use checkbox::checkbox;
pub use command::{command, command_matches};
pub use context_menu::context_menu;
pub use pagination::pagination;
pub use popover::popover;
pub use progress::progress;
pub use radio::radio_group;
pub use select::select;
pub use sheet::sheet;
pub use skeleton::{Loading, Show, skeleton};
pub use skip_link::skip_link;
pub use table::table;
pub use toast::toast;
pub use toggle::{toggle, toggle_group};
pub use tooltip::tooltip;
pub use caption::{CaptionParam, caption};
pub use card::card;
pub use card_stack::{Stack, StackLook, StackMsg, card_stack};
pub use choice::{choice, segmented};
pub use dialog::dialog;
pub use divider::{divider, vdivider};
pub use dot_chart::{ChartMsg, DotChart, dot_chart, spline};
pub use image::image;
pub use input::{TextEdit, input, textarea};
pub use list::{list, list_virtual};
pub use menu::menu;
pub use nested_menu::{Menu, MenuItem, MenuMsg, in_triangle, nested_menu};
pub use number::{Edit, SCRUB, Spec, number};
pub use orbit::{Orbit, OrbitMsg, orbit};
pub use scroll::scroll_area;
pub use slider::{slider, slider_value};
pub use switch::{switch, switch_sized};
pub use tabs::tabs;

/// Sizes shared by the widgets, logical px.
pub const LABEL: f32 = 14.0;
pub const CONTROL_H: f32 = 30.0;
pub const LINE: f32 = 1.0;
pub const RADIUS: f32 = 4.0;
/// The plate of what is selected or under the keys in a list or a menu on the ground: the raised grey.
pub const SELECTED: crate::el::Paint = crate::el::Paint::RAISED;
/// The plate of what is selected or under the keys on `surface`: `SELECTED`, or the hairline grey on a
/// surface that is itself raised (a menu, a sheet), where the raised grey would not show.
pub fn selected_on(surface: crate::el::Paint) -> crate::el::Paint { if surface == crate::el::Paint::RAISED { crate::el::Paint::HAIRLINE } else { SELECTED } }
/// The surface of the menus and lists that open over the page (menu, select, context and nested menus)
/// and of the sheets (sheet, bottom sheet): the raised grey, in a hairline frame; what is selected or hot
/// on it is `selected_on(MENU_SURFACE)`, the hairline grey.
pub const MENU_SURFACE: crate::el::Paint = crate::el::Paint::RAISED;
