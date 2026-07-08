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
use crate::model::MoveResult;
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

        let move_result = try_make_move(fen, origin, destination);

		if move_result.success {
            let handle = ui_handle.unwrap();

            let moves = update_move_list(&handle, move_result.clone());

            handle.set_fen(move_result.fenboard.to_fen());
            handle.set_player_color(move_result.fenboard.active_color.as_str().into());
            handle.set_moves(Rc::new(slint::VecModel::from(moves)).into());
        }

        move_result.success
    });

    ui.run()?;

    Ok(())
}

fn update_move_list(ui: &AppWindow, move_result: MoveResult) -> Vec<Moves> {

    let ui_handle = ui.as_weak();
    let handle = ui_handle.unwrap();
    let moves = handle.get_moves();

    let mut moves: Vec<Moves> = moves.iter().collect();

    match move_result.fenboard.active_color.as_str() {
        "b" => moves.push(Moves {
            white: move_result.fenboard.san_move.clone().into(),
            black: "".into()
        }),
        _ =>  if let Some(last_move) = moves.last_mut() {
                last_move.black = move_result.fenboard.san_move.clone().into();
            }
    }

    moves
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
