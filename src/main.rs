// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod grand_master;
mod fen_parser;

use std::error::Error;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::PhysicalSize;
use crate::fen_parser::*;
use crate::grand_master::*;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
	let ui_handle = ui.as_weak();

    set_screen_size(&ui)?;

    ui.global::<Callbacks>().on_piece_from_fen(|fen, index| {
        get_piece_code_from_fen(fen.to_string(), index).into()
    });

	ui.global::<Callbacks>().on_make_move(move |fen, origin, destination| -> bool {

        let (success, new_position, player_turn) = try_making_move(fen, origin, destination);

		if success {
			let handle = ui_handle.unwrap();

			 handle.set_fen(new_position);
			 handle.set_player_turn(player_turn);
		}

        return success;
    });

    ui.run()?;

    Ok(())
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
