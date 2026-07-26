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
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let ui_handle = ui.as_weak();
    let audio_player = AudioPlayer::new();

    //set_full_screen(&ui)?;

    let opening_position = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let player_turn = opening_position.split(' ').nth(1).unwrap();
    ui.set_fen(opening_position.into());
    ui.set_player_color(player_turn.into());

    let puzzle_master = Rc::new(RefCell::new(initial_position(&ui_handle)));

    ui.global::<Callbacks>().on_piece_from_fen(|fen, index|
        get_piece_code_from_fen(fen, index).into());

    ui.global::<Callbacks>().on_color_on_square(|fen, square|
        get_piece_color_from_square(fen, square));

    let ui_clone = ui_handle.clone();
    let puzzle_master_clone = puzzle_master.clone();
    ui.global::<Callbacks>().on_make_move(move |fen, origin, destination| -> bool {

        if origin == "" || destination == "" {
            return false;
        }

        let handle = ui_clone.unwrap();

        let mut pm = puzzle_master_clone.borrow_mut();

        pm.fen = fen;
        pm.start_square = origin.clone();
        pm.end_square = destination.clone();

        let (success, san_move, finished_line, finished_line_move, move_type) = do_make_move(&mut pm);

        if success {
            do_go_to_position(&ui_clone, san_move, finished_line, finished_line_move);
        }

        audio_player.play_sound(&move_type);

        success
    });

    let ui_clone = ui_handle.clone();
    let puzzle_master_clone = puzzle_master.clone();
    ui.global::<Callbacks>().on_go_to_position(move |san_move| {

        let mut pm = puzzle_master_clone.borrow_mut();
        pm.next = Some(san_move.next_id as usize);

        do_go_to_position(&ui_clone, san_move, false, -1);
    });

    let ui_clone = ui_handle.clone();
    let puzzle_master_clone = puzzle_master.clone();
    ui.global::<Callbacks>().on_import_pgn(move |pgn| {
        let handle = ui_clone.clone().unwrap();

        let reader = parse_pgn(pgn.into());

       *puzzle_master_clone.borrow_mut() = PuzzleMaster {
            move_reader: reader.clone(),
            next: Some(0),
            ..Default::default()
        };

        handle.set_fen(opening_position.to_shared_string());
        handle.set_player_color("w".to_shared_string());
        handle.set_current_move(SanMove{ id: -2, ..Default::default() });
        handle.set_move_rows(Rc::new(slint::VecModel::from(reader.san_move_rows)).into());
    });

    let ui_clone = ui_handle.clone();
    ui.global::<Callbacks>().on_choose_pgn(move ||{

       let handle = ui_clone.clone().unwrap();
       let dir_path = r"C:\ChessPgn";

       let entries: Vec<PgnItem> = match fs::read_dir(dir_path) {
            Ok(read_dir) => read_dir
                .filter_map(|entry| {
                    let entry = entry.ok()?;

                    if !entry.file_type().ok()?.is_file() {
                        return None;
                    }

                    let file_name: SharedString = entry.file_name().into_string().ok()?.into();
                    let file_data: SharedString = fs::read_to_string(entry.path()).ok()?.into();

                    Some( PgnItem { filename: file_name, contents: file_data })
                })
                .collect(),
            Err(e) => {
                eprintln!("Error reading directory: {}", e);
                return;
            }
        };

         handle.set_openings(Rc::new(slint::VecModel::from(entries)).into());
    });

    ui.run()?;

    Ok(())
}

fn initial_position(ui: &Weak<AppWindow>) -> PuzzleMaster {

    let handle = ui.unwrap();
    let reader = parse_pgn("1. e4 c5 2. Nf3 d6 (2... Nc6 3. Bb5 g6 (3... e6 4. d4)) (2... g6 3. d4 cxd4) 3.
d4 cxd4 4. Nxd4 *".into());

    handle.set_current_move(SanMove{ id: -2, ..Default::default() });
    handle.set_move_rows(Rc::new(slint::VecModel::from(reader.san_move_rows.clone())).into());

    PuzzleMaster {
         move_reader: reader,
         next: Some(0),
         ..Default::default()
    }
}

fn do_make_move(puzzle_master: &mut PuzzleMaster) -> (bool, SanMove, bool, i32, MoveType) {

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
            for i in 0..move_rows.row_count() {
                let mut row = move_rows.row_data(i).unwrap();

                if row.white.variation_id == san_move.variation_id
                    || row.black.variation_id == san_move.variation_id
                {
                    row.white.hide_move = row.white.id > san_move.id;
                    row.black.hide_move = row.black.id > san_move.id;

                    move_rows.set_row_data(i, row);
                }
            }
        }
    }

    let scroll_to = move_rows.clone()
        .iter()
        .enumerate()
        .find(|(_, x)| x.white.id == san_move.id || x.black.id == san_move.id)
        .unwrap();

    handle.invoke_scroll_to_index_y(scroll_to.0 as i32);
    handle.invoke_scroll_to_index_x(scroll_to.1.depth);

    handle.set_fen(san_move.fen.clone());
    handle.set_current_move(san_move.clone());
    handle.set_player_color(player_color[1].to_shared_string());
    handle.invoke_clear_active_coords();
    handle.set_last_move_from(san_move.from_square.clone());
    handle.set_last_move_to(san_move.to_square.clone());
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
