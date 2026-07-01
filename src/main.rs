// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::iter::repeat;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::PhysicalSize;
use slint::SharedString;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    set_screen_size(&ui)?;

     ui.global::<Callbacks>().on_piece_from_fen(|fen, index| {
        get_piece_from_fen(fen, index)
    });

    ui.run()?;

    Ok(())
}

fn get_piece_from_fen(fen: SharedString, cell_index: i32) -> SharedString {

    let piece_placement = fen.split(' ').next().unwrap();
    let mut ranks = piece_placement.split('/');

    let rank_index = (cell_index / 8) as usize;
    let file_index = (cell_index % 8) as usize;

    let cell_rank = ranks.nth(rank_index).unwrap_or("");
    let row = pad_empty_squares(cell_rank.to_string());

    let value = row
        .chars()
        .nth(file_index)
        .unwrap_or('.');

    value.to_string().into()
}

fn pad_empty_squares(fen_row: String) -> SharedString {
    let mut output = String::new();

    for c in fen_row.chars() {
        match c.to_digit(10) {
            Some(n) => output.extend(repeat('.').take(n as usize)),
            None => output.push(c),
        }
    }

    output.into()
}

fn set_screen_size(ui: &AppWindow) -> Result<(), Box<dyn Error>> {
    let ui_weak = ui.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let window = ui.window();
            window.with_winit_window(|winit_window| {

                if let Some(monitor) = winit_window.current_monitor() {
                    let size = monitor.size();
                    window.set_size(PhysicalSize::new(size.width, size.height));
                    window.set_maximized(true);
                    //window.set_fullscreen(true);
                }
            });
        }
    })?;

    return Ok(())
}
