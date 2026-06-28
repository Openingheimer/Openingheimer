// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use slint::LogicalPosition;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = init_ui()?;

    init_callbacks(&ui)?;

    ui.run()?;

    Ok(())
}

fn init_ui() -> Result<AppWindow, Box<dyn Error>> {
    let ui = AppWindow::new()?;

    ui.window().set_position(LogicalPosition::new(100 as f32, 100 as f32));

    Ok(ui)
}

fn init_callbacks(ui: &AppWindow) -> Result<(), Box<dyn Error>> {
    let ui_handle = ui.as_weak();

    ui.on_request_increase_value(move || {
        let ui = ui_handle.unwrap();
        ui.set_counter(ui.get_counter() + 1);
    });

    Ok(())
}
