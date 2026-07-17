// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]
slint::include_modules!();

mod fen_master;
mod grand_master;
mod model;
mod movetext;

use crate::fen_master::*;
use crate::grand_master::*;
use crate::model::*;
use crate::movetext::*;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::Model;
use slint::PhysicalSize;
use slint::SharedString;
use slint::ToSharedString;
use slint::Weak;
use std::error::Error;
use std::rc::Rc;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();
    //set_full_screen(&ui)?;

    let fen = ui_handle.clone().unwrap().get_fen();

    let player_turn = fen.split(' ').nth(1).unwrap();

    ui.set_player_color(player_turn.into());

    ui.global::<Callbacks>().on_piece_from_fen(|fen, index|
        get_piece_code_from_fen(fen, index).into());

    ui.global::<Callbacks>().on_color_on_square(|fen, square|
        get_piece_color_from_square(fen, square));

    let make_move_weak = ui_handle.clone();
    ui.global::<Callbacks>().on_make_move(move |fen, origin, destination| -> bool {
        do_make_move(&make_move_weak, fen, origin, destination)
    });

    let is_legal_weak = ui_handle.clone();
    ui.global::<Callbacks>().on_check_legal_move(move |square| -> bool {
        do_check_legal_square(&is_legal_weak, square)
    });

    let refresh_legal_moves = ui_handle.clone();
    ui.global::<Callbacks>().on_refresh_legal_moves(move |fen, square, clear| {
        do_refresh_legal_moves(&refresh_legal_moves, fen, square, clear)
    });

    let go_to_position = ui_handle.clone();
    ui.global::<Callbacks>().on_go_to_position(move |san_move| {
        do_go_to_position(&go_to_position, san_move);
    });

    let import_pgn = ui_handle.clone();
    ui.global::<Callbacks>().on_import_pgn(move |pgn| {

        let handle = import_pgn.clone().unwrap();
        let fen = handle.get_fen();
        println!("PGN {}", pgn);

        do_make_move(&import_pgn, fen, "e2".into(), "e4".into());
        ()
    });

    ui.run()?;

    Ok(())
}

fn do_make_move(ui: &Weak<AppWindow>, fen: SharedString, origin: SharedString, destination: SharedString) -> bool {
    if origin == "" || destination == "" {
        return false;
    }

    let move_result = try_make_move(fen, origin, destination);

    if move_result.success {

        let handle = ui.clone().unwrap();
        let current_move = handle.get_current_move();
        let moves: Vec<SanMoveRow> = handle.get_move_rows().iter().collect();
        let last_move_in_variation = handle.get_last_move_in_variation();

        let child_moves: Vec<SanMoveRow> = handle.get_move_rows()
            .iter()
            .filter(|x| x.white.previous_id == current_move.id || x.black.previous_id == current_move.id)
            .collect();

        let move_played_already = match move_result.fenboard.active_color.clone() {
            Color::White => child_moves.iter().find(|x| x.black.san_text == move_result.fenboard.san_move),
            Color::Black => child_moves.iter().find(|x| x.white.san_text == move_result.fenboard.san_move),
        };

        if let Some(m) = move_played_already {
            let san_move = match move_result.fenboard.active_color.clone() {
                Color::White => &m.black,
                Color::Black => &m.white,
            };

            do_go_to_position(ui, san_move.clone());

            let (x, y) = get_scroll_point(moves, current_move.id, m.depth);
            handle.invoke_scroll_to_y(y);
            handle.invoke_scroll_to_x(x);

            return true;
        }

        let move_request = MoveRequest {
            fenboard: move_result.fenboard.clone(),
            moves: moves,
            current_move: current_move,
            last_move_in_variation: last_move_in_variation
        };

        let response = update_move_list(move_request.clone());

        handle.set_fen(move_result.fenboard.to_fen());
        handle.set_player_color(move_result.fenboard.active_color.as_str().into());
        handle.set_move_rows(Rc::new(slint::VecModel::from(response.moves)).into());
        handle.set_current_move(response.current_move.clone());
        handle.set_last_move_in_variation(response.current_move.id);
        handle.invoke_scroll_to_y(response.scroll_y);
        handle.invoke_scroll_to_x(response.scroll_x);
    }

    move_result.success
}

fn do_go_to_position(ui: &Weak<AppWindow>, san_move: SanMove) {
    let handle = ui.unwrap();

    if san_move.san_text.is_empty() || san_move.san_text == ".." {
        return;
    }

    let player_color: Vec<&str> = san_move.fen.split(' ').collect();

    handle.set_fen(san_move.fen.clone());
    handle.set_current_move(san_move.clone());
    handle.set_player_color(player_color[1].to_shared_string());
}

fn do_check_legal_square(ui: &Weak<AppWindow>, square: SharedString) -> bool {
    if square == "" {
        return false;
    }

    let handle = ui.unwrap();
    let legal_moves = handle.get_legal_moves();

    legal_moves.iter().any(|m| m.as_str() == square.as_str())
}

fn do_refresh_legal_moves(ui: &Weak<AppWindow>, fen: SharedString, square: SharedString, clear: bool) {
    if square != "" {
        let handle = ui.unwrap();
        let piece = get_piece_from_fen(fen.clone(), square.clone());

        let legal_moves = match piece {
            _ if clear => [].to_vec(),
            Some(p) => p.get_moves(fen, square_to_index64(square)),
            _ => [].to_vec(),
        };

        handle.set_legal_moves(Rc::new(slint::VecModel::from(legal_moves)).into());
    }
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

    return Ok(());
}
