// Menu bar for the browser
use tao::{
    menu::{MenuBar as TaoMenuBar, MenuItem as TaoMenuItem, MenuItemAttributes},
    window::WindowId,
};

/// Build the browser menu bar
pub fn build_menu() -> TaoMenuBar {
    let mut menu = TaoMenuBar::new();

    // File menu
    let mut file_menu = TaoMenuBar::new();
    file_menu.add_item(MenuItemAttributes::new("New Tab").with_accelerators(&["Ctrl+T"]));
    file_menu.add_item(MenuItemAttributes::new("New Window").with_accelerators(&["Ctrl+N"]));
    file_menu.add_native_item(tao::menu::MenuItem::Separator);
    file_menu.add_item(MenuItemAttributes::new("Close Tab").with_accelerators(&["Ctrl+W"]));
    file_menu.add_item(MenuItemAttributes::new("Close Window").with_accelerators(&["Ctrl+Shift+W"]));
    file_menu.add_native_item(tao::menu::MenuItem::Separator);
    file_menu.add_item(MenuItemAttributes::new("Exit").with_accelerators(&["Ctrl+Q"]));
    
    menu.add_submenu("File", true, file_menu);

    // Edit menu
    let mut edit_menu = TaoMenuBar::new();
    edit_menu.add_native_item(tao::menu::MenuItem::Undo);
    edit_menu.add_native_item(tao::menu::MenuItem::Redo);
    edit_menu.add_native_item(tao::menu::MenuItem::Separator);
    edit_menu.add_native_item(tao::menu::MenuItem::Cut);
    edit_menu.add_native_item(tao::menu::MenuItem::Copy);
    edit_menu.add_native_item(tao::menu::MenuItem::Paste);
    edit_menu.add_native_item(tao::menu::MenuItem::SelectAll);
    
    menu.add_submenu("Edit", true, edit_menu);

    // View menu
    let mut view_menu = TaoMenuBar::new();
    view_menu.add_item(MenuItemAttributes::new("Reload").with_accelerators(&["Ctrl+R"]));
    view_menu.add_item(MenuItemAttributes::new("Force Reload").with_accelerators(&["Ctrl+Shift+R"]));
    view_menu.add_native_item(tao::menu::MenuItem::Separator);
    view_menu.add_item(MenuItemAttributes::new("Zoom In").with_accelerators(&["Ctrl+Plus"]));
    view_menu.add_item(MenuItemAttributes::new("Zoom Out").with_accelerators(&["Ctrl+Minus"]));
    view_menu.add_item(MenuItemAttributes::new("Reset Zoom").with_accelerators(&["Ctrl+0"]));
    view_menu.add_native_item(tao::menu::MenuItem::Separator);
    view_menu.add_item(MenuItemAttributes::new("Toggle Developer Tools").with_accelerators(&["F12"]));
    
    menu.add_submenu("View", true, view_menu);

    // History menu
    let mut history_menu = TaoMenuBar::new();
    history_menu.add_item(MenuItemAttributes::new("Back").with_accelerators(&["Alt+Left"]));
    history_menu.add_item(MenuItemAttributes::new("Forward").with_accelerators(&["Alt+Right"]));
    history_menu.add_item(MenuItemAttributes::new("Home").with_accelerators(&["Alt+Home"]));
    history_menu.add_native_item(tao::menu::MenuItem::Separator);
    history_menu.add_item(MenuItemAttributes::new("Show History").with_accelerators(&["Ctrl+H"]));
    history_menu.add_item(MenuItemAttributes::new("Clear History"));
    
    menu.add_submenu("History", true, history_menu);

    // Bookmarks menu
    let mut bookmarks_menu = TaoMenuBar::new();
    bookmarks_menu.add_item(MenuItemAttributes::new("Bookmark This Page").with_accelerators(&["Ctrl+D"]));
    bookmarks_menu.add_item(MenuItemAttributes::new("Show Bookmarks").with_accelerators(&["Ctrl+Shift+B"]));
    bookmarks_menu.add_native_item(tao::menu::MenuItem::Separator);
    bookmarks_menu.add_item(MenuItemAttributes::new("Bookmark Manager"));
    
    menu.add_submenu("Bookmarks", true, bookmarks_menu);

    // Help menu
    let mut help_menu = TaoMenuBar::new();
    help_menu.add_item(MenuItemAttributes::new("About WebX"));
    help_menu.add_item(MenuItemAttributes::new("Check for Updates"));
    help_menu.add_native_item(tao::menu::MenuItem::Separator);
    help_menu.add_item(MenuItemAttributes::new("Report Issue"));
    help_menu.add_item(MenuItemAttributes::new("Documentation"));
    
    menu.add_submenu("Help", true, help_menu);

    menu
}
