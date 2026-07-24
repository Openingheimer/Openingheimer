// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]
#![allow(unused_variables)]

slint::include_modules!();

mod fen_master;
mod grand_master;
mod model;
mod movetext;
mod pgn_import;
mod puzzle_master;

use crate::fen_master::*;
use crate::model::*;
use crate::pgn_import::*;
use crate::puzzle_master::*;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::Model;
use slint::PhysicalSize;
use slint::SharedString;
use slint::ToSharedString;
use slint::Weak;
use std::error::Error;
use std::rc::Rc;
use std::cell::RefCell;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();

    //set_full_screen(&ui)?;

    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let player_turn = fen.split(' ').nth(1).unwrap();
    ui.set_fen(fen.into());
    ui.set_player_color(player_turn.into());

    let puzzle_master = Rc::new(RefCell::new(seed_position(&ui_handle)));

    ui.global::<Callbacks>().on_piece_from_fen(|fen, index|
        get_piece_code_from_fen(fen, index).into());

    ui.global::<Callbacks>().on_color_on_square(|fen, square|
        get_piece_color_from_square(fen, square));

    let make_move_weak = ui_handle.clone();
    let puzzle_master_clone = puzzle_master.clone();
    ui.global::<Callbacks>().on_make_move(move |fen, origin, destination| -> bool {

        let handle = make_move_weak.unwrap();

        let mut pm = puzzle_master_clone.borrow_mut();

        pm.fen = fen;
        pm.start_square = origin;
        pm.end_square = destination;

        let (success, san_move, finished_line, finished_line_move) = do_make_move(&mut pm);

       if success {
            do_go_to_position(&make_move_weak, san_move, finished_line, finished_line_move);
            // handle.invoke_scroll_to_y(response.scroll_y);
            // handle.invoke_scroll_to_x(response.scroll_x);
       }

        success
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
    let puzzle_master_clone = puzzle_master.clone();
    ui.global::<Callbacks>().on_go_to_position(move |san_move| {

        let mut pm = puzzle_master_clone.borrow_mut();
        pm.next = Some(san_move.next_id as usize);

        do_go_to_position(&go_to_position, san_move, false, -1);
    });

    let import_pgn = ui_handle.clone();
    let puzzle_master_clone = puzzle_master.clone();
    ui.global::<Callbacks>().on_import_pgn(move |pgn| {
        let handle = import_pgn.clone().unwrap();

        let reader = parse_pgn(pgn.into());

       *puzzle_master_clone.borrow_mut() = PuzzleMaster {
            move_reader: reader.clone(),
            next: Some(0),
            ..Default::default()
        };

        handle.set_move_rows(Rc::new(slint::VecModel::from(reader.san_move_rows)).into());
    });

    ui.run()?;

    Ok(())
}

fn seed_position(ui: &Weak<AppWindow>) -> PuzzleMaster {

    let handle = ui.unwrap();
    let reader = parse_pgn("1. e4 c5 2. Nf3 Nc6 (2... d6 3. Nc3) (2... g6 3. g3) 3. Bc4 *".into());

    handle.set_current_move(SanMove{ id: -2, ..Default::default() });
    handle.set_move_rows(Rc::new(slint::VecModel::from(reader.san_move_rows.clone())).into());

    PuzzleMaster {
         move_reader: reader,
         next: Some(0),
         ..Default::default()
    }
}

fn do_make_move(puzzle_master: &mut PuzzleMaster) -> (bool, SanMove, bool, i32) {

    check_move(puzzle_master)
}

fn do_go_to_position(ui: &Weak<AppWindow>, san_move: SanMove, finished_line: bool, finished_line_move: i32) {
    let handle = ui.unwrap();

    if san_move.san_text.is_empty() || san_move.san_text == ".." {
        return;
    }

    let player_color: Vec<&str> = san_move.fen.split(' ').collect();
    let move_rows = handle.get_move_rows();

    match finished_line {
        true => {
            let mut finished_variation = move_rows
                .iter()
                .enumerate()
                .find(|(_, x)| x.black.id == finished_line_move || x.white.id == finished_line_move)
                .unwrap();

            finished_variation.1.black.hide_move = false;
            finished_variation.1.white.hide_move = false;

           move_rows.set_row_data(finished_variation.0 as usize, finished_variation.1);
        },
        false => {
            let moves: Vec<SanMoveRow> = move_rows
            .iter()
            .enumerate()
            .map(|(i, mut x)| {

                if x.white.variation_id == san_move.variation_id || x.black.variation_id == san_move.variation_id {
                    if x.white.id > san_move.id {
                        x.white.hide_move = true;
                    }
                    else{
                        x.white.hide_move = false;
                    }
                    if x.black.id > san_move.id {
                        x.black.hide_move = true;
                    }
                    else{
                        x.black.hide_move = false;
                    }
                }

                x
            })
            .collect();

            handle.set_move_rows(Rc::new(slint::VecModel::from(moves.clone())).into());
        }
    }

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
