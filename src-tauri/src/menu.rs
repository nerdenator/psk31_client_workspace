//! Native menu bar setup
//!
//! Creates the application menu bar with File, Configurations, View, and Help menus.
//! Menu events are emitted to the frontend for handling.

use tauri::{
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    App, Emitter,
};

/// Menu event payload sent to frontend
#[derive(Clone, serde::Serialize)]
pub struct MenuEvent {
    pub id: String,
}

/// Set up the application menu bar
pub fn setup_menu(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let handle = app.handle();

    // Build File menu
    let settings_item = MenuItemBuilder::with_id("settings", "Settings...")
        .accelerator("CmdOrCtrl+,")
        .build(handle)?;

    let file_menu = SubmenuBuilder::new(handle, "File")
        .item(&settings_item)
        .separator()
        .quit()
        .build()?;

    // Build Configurations menu
    let config_default = MenuItemBuilder::with_id("config_default", "Default ✓").build(handle)?;

    let config_save =
        MenuItemBuilder::with_id("config_save", "Save Current Configuration").build(handle)?;

    let config_delete =
        MenuItemBuilder::with_id("config_delete", "Delete Configuration...").build(handle)?;

    let configurations_menu = SubmenuBuilder::new(handle, "Configurations")
        .item(&config_default)
        .separator()
        .item(&config_save)
        .item(&config_delete)
        .build()?;

    // Build View menu
    let theme_light = MenuItemBuilder::with_id("theme_light", "Light Theme").build(handle)?;

    let theme_dark = MenuItemBuilder::with_id("theme_dark", "Dark Theme").build(handle)?;

    let waterfall_colors =
        MenuItemBuilder::with_id("waterfall_colors", "Waterfall Colors...").build(handle)?;

    let zoom_in = MenuItemBuilder::with_id("zoom_in", "Zoom In")
        .accelerator("CmdOrCtrl+=")
        .build(handle)?;

    let zoom_out = MenuItemBuilder::with_id("zoom_out", "Zoom Out")
        .accelerator("CmdOrCtrl+-")
        .build(handle)?;

    let zoom_reset = MenuItemBuilder::with_id("zoom_reset", "Reset Zoom")
        .accelerator("CmdOrCtrl+0")
        .build(handle)?;

    let view_menu = SubmenuBuilder::new(handle, "View")
        .item(&theme_light)
        .item(&theme_dark)
        .separator()
        .item(&waterfall_colors)
        .separator()
        .item(&zoom_in)
        .item(&zoom_out)
        .item(&zoom_reset)
        .build()?;

    // Build Help menu
    let documentation = MenuItemBuilder::with_id("documentation", "Documentation").build(handle)?;

    let about = MenuItemBuilder::with_id("about", "About PSK-31").build(handle)?;

    let help_menu = SubmenuBuilder::new(handle, "Help")
        .item(&documentation)
        .separator()
        .item(&about)
        .build()?;

    // Build the complete menu bar
    let menu = MenuBuilder::new(handle)
        .item(&file_menu)
        .item(&configurations_menu)
        .item(&view_menu)
        .item(&help_menu)
        .build()?;

    // Set the menu at app level (works on macOS)
    app.set_menu(menu)?;

    // Handle menu events at app level
    app.on_menu_event(move |app_handle, event| {
        let id = event.id().0.clone();
        println!("Menu event: {}", id);

        // Emit event to frontend
        let _ = app_handle.emit("menu-event", MenuEvent { id: id.clone() });

        // Handle quit specially (doesn't need frontend)
        if id == "quit" {
            std::process::exit(0);
        }
    });

    Ok(())
}
