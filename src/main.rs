// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod grand_master;
mod fen_master;
mod model;

use std::error::Error;
use std::rc::Rc;
use slint::PhysicalSize;
use slint::Model;
use slint::ToSharedString;
use std::sync::atomic::{AtomicU64, Ordering};
use crate::fen_master::*;
use crate::grand_master::*;
use crate::model::MoveResult;
use crate::model::PieceBrain;
use i_slint_backend_winit::WinitWindowAccessor;

slint::include_modules!();

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
	let ui_handle = ui.as_weak();
    //set_full_screen(&ui)?;

    let fen = ui_handle.clone().unwrap().get_fen();

    let player_turn = fen
        .split(' ')
        .nth(1)
        .unwrap();

    ui.set_player_color(player_turn.into());

    ui.global::<Callbacks>().on_piece_from_fen(|fen, index| {
        get_piece_code_from_fen(fen, index).into()
    });

    ui.global::<Callbacks>().on_color_on_square(|fen, square| {
        get_piece_color_from_square(fen, square)
    });

    let make_move_weak = ui_handle.clone();

	ui.global::<Callbacks>().on_make_move(move |fen, origin, destination| -> bool {

        println!("Making Move {} {}", origin, destination);

        if origin == "" || destination == "" {
            return false;
        }

        let move_result = try_make_move(fen, origin, destination);

		if move_result.success {
            let handle = make_move_weak.unwrap();

            let moves = update_move_list(&handle, move_result.clone());

            handle.set_fen(move_result.fenboard.to_fen());
            handle.set_player_color(move_result.fenboard.active_color.as_str().into());
            handle.set_move_rows(Rc::new(slint::VecModel::from(moves)).into());
        }

        move_result.success
    });

    let is_legal_weak = ui_handle.clone();
    ui.global::<Callbacks>().on_check_legal_move(move |square| -> bool {

        if square == "" {
            return false;
        }

        let handle = is_legal_weak.unwrap();
        let legal_moves = handle.get_legal_moves();

        legal_moves.iter().any(|m| m.as_str() == square.as_str())
    });

    let refresh_legal_moves = ui_handle.clone();
    ui.global::<Callbacks>().on_refresh_legal_moves(move |fen, square, clear| {

        if square != "" {
            let handle = refresh_legal_moves.unwrap();
            let piece = get_piece_from_fen(fen.clone(), square.clone());

            let legal_moves = match piece {
                _ if clear => [].to_vec(),
                Some(p) => p.get_moves(fen, square_to_index64(square)),
                _ => [].to_vec(),
            };

            handle.set_legal_moves(Rc::new(slint::VecModel::from(legal_moves)).into());
        }

    });

    let go_to_position = ui_handle.clone();
    ui.global::<Callbacks>().on_go_to_position(move |moves| {

        let handle = go_to_position.unwrap();

        let player_color: Vec<&str> = moves.fen.split(' ').collect();

        handle.set_fen(moves.fen.clone());
        handle.set_current_move(moves.id);
        handle.set_player_color(player_color[1].to_shared_string());
    });

    ui.run()?;

    Ok(())
}

fn update_move_list(ui: &AppWindow, move_result: MoveResult) -> Vec<MoveRowItem> {

    let ui_handle = ui.as_weak();
    let handle = ui_handle.unwrap();
    let mut moves: Vec<MoveRowItem> = handle.get_move_rows().iter().collect();
    let id = next_id().to_shared_string();

    match move_result.fenboard.active_color.as_str() {
        "b" => {
                moves.push(MoveRowItem {
                    black: SanMove { fen: move_result.fenboard.to_fen(), id: "0".into(), san_text: "".into(), variation: "0".into() },
                    white: SanMove {
                        id: id.clone(),
                        fen: move_result.fenboard.to_fen(),
                        san_text: move_result.fenboard.san_move.into(),
                        variation: "0".into()
                    },
                })
            },

        _ =>  if let Some(last_move) = moves.last_mut() {
                last_move.black.id = id.clone();
                last_move.black.fen = move_result.fenboard.to_fen();
                last_move.black.san_text = move_result.fenboard.san_move;
            }
    }

    handle.set_current_move(id);

    moves
}

fn set_full_screen(ui: &AppWindow) -> Result<(), Box<dyn Error>> {

    let ui_weak = ui.as_weak();
    let handle = ui_weak.unwrap();
    handle.set_is_full_screen(true);
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

fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}