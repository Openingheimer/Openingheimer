// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod grand_master;
mod fen_master;
mod model;

use std::error::Error;
use std::rc::Rc;
use slint::PhysicalSize;
use slint::Model;
use crate::fen_master::*;
use crate::grand_master::*;
use i_slint_backend_winit::WinitWindowAccessor;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
	let ui_handle = ui.as_weak();
    set_screen_size(&ui)?;

    let fen = ui.get_fen();

    let player_turn = fen
        .split(' ')
        .nth(1)
        .unwrap();

    ui.set_player_color(player_turn.into());

    ui.global::<Callbacks>().on_piece_from_fen(|fen, index| {
        get_piece_code_from_fen(fen.to_string(), index).into()
    });

    ui.global::<Callbacks>().on_color_on_square(|fen, square| {
        get_piece_color_from_square(fen, square)
    });

	ui.global::<Callbacks>().on_make_move(move |fen, origin, destination| -> bool {

        let (success, new_position, player_color, san_move) = try_make_move(fen, origin, destination);

		if success {
            let handle = ui_handle.unwrap();

            handle.set_fen(new_position);
            handle.set_player_color(player_color.clone());


            let moves = handle.get_moves();

            let mut move_list: Vec<Moves> = moves.iter().collect();

            match player_color.as_str() {
                "b" => move_list.push(Moves {
                    white: san_move.into(),
                    black: "".into()
                }),
                _ =>  if let Some(last_move) = move_list.last_mut() {
                        last_move.black = san_move.into();
                    }
            }

            handle.set_moves(Rc::new(slint::VecModel::from(move_list)).into());
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
