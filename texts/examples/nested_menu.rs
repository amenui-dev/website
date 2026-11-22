//! nested_menu: its examples, as the documentation page shows them (each `ex_*` function is a preview
//! and its code).
use amenui::el::El;
use amenui::widgets::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Msg { Nothing, Menu(MenuMsg) }

/// 一个子菜单打开着 · With a submenu open
pub fn ex_0() -> El<Msg> {
    let mut m = Menu::new(vec![
        MenuItem::sub("New", vec![MenuItem::leaf("Document"), MenuItem::leaf("Spreadsheet"), MenuItem::leaf("Folder")]),
        MenuItem::sub("Open recent", vec![MenuItem::leaf("Report"), MenuItem::leaf("Draft")]),
        MenuItem::leaf("Close"),
    ]);
    m.update(MenuMsg::Press(0, 0), 0.0);
    nested_menu("nm1", &m, Msg::Menu)
}
