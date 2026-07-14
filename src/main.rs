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
use slint::VecModel;
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

        let moves = update_move_list(&handle, move_result.fenboard.clone());

        handle.set_fen(move_result.fenboard.to_fen());
        handle.set_player_color(move_result.fenboard.active_color.as_str().into());
        handle.set_move_rows(Rc::new(slint::VecModel::from(moves)).into());
    }

    move_result.success
}

fn do_go_to_position(ui: &Weak<AppWindow>, san_move: SanMove) {
    let handle = ui.unwrap();

    let player_color: Vec<&str> = san_move.fen.split(' ').collect();

    println!("Id {}", san_move.id);
    println!("Prev {}", san_move.previous_id);
    println!("Nex {}", san_move.next_id);
    println!("Variation {}", san_move.variation);

    for x in san_move.parent_branches.iter() {
        println!("Parent Branch - {}", x);
    }
   // println!("Setting Last Move in Var {}", san_move.id);
    handle.set_fen(san_move.fen.clone());
    handle.set_current_move(san_move.clone());
    handle.set_player_color(player_color[1].to_shared_string());
   // handle.set_last_move_in_variation(san_move.id);
   // handle.set_active_variation(san_move.variation);
}

fn update_move_list(ui: &AppWindow, fenboard: FenBoard) -> Vec<SanMoveRow> {
    let ui_handle = ui.as_weak();
    let handle = ui_handle.unwrap();

    let mut moves: Vec<SanMoveRow> = handle.get_move_rows().iter().collect();
    let mut new_variation = is_new_variation(&ui, &moves);
    let current_move = handle.get_current_move();

    let variation = match new_variation {
        true => {
            println!("New Var");
            next_variation_id()
        },
        false => current_move.variation
    };

    if moves.is_empty() {
       let first_move = create_first_move(&fenboard, &current_move, next_variation_id());
       handle.set_current_move(first_move.white.clone());
       handle.set_last_move_in_variation(first_move.white.id);
       moves.push(first_move);

       return moves;
    }

    let mut new_move = complete_black_move_or_get_new(fenboard.clone(), &mut moves, &current_move, variation, new_variation);

    let san_move = match fenboard.active_color {
        Color::White => new_move.black.clone(),
        Color::Black => new_move.white.clone(),
    };

    let (mut last_move, index) = get_last_move_in_variation(&moves, &current_move, variation);

    match fenboard.active_color {
        Color::White => {
            match new_variation {
                false => {
                    moves[index] = new_move.clone()
                },
                true => {
                     let splice_index = moves
                            .clone()
                            .iter()
                            .position(|x| x.white.id == current_move.id)
                            .unwrap() + 1;

                    new_move.depth = new_move.depth + 1;
                    new_move.white = SanMove::default();
                    new_move.white.san_text = "..".into();
                    new_move.white.fen = new_move.black.fen.clone();
                    moves.splice(splice_index..splice_index, [new_move.clone()].into_iter());
                }
            }
        },
        Color::Black => {
            last_move.black.next_id = new_move.white.id.clone();

            match new_variation {
                false => {

                    let splice_index = match moves
                            .clone()
                            .iter()
                            .rposition(|x| {
                                x.black.parent_branches.iter().any(|b| b == variation) ||
                                x.white.parent_branches.iter().any(|b| b == variation)
                            }) {
                                Some(m) => {
                                    match variation {
                                        1 => moves.iter().count(),
                                        _ => m + 1
                                    }
                                }
                                None => {
                                    moves.iter().position(|x| x.black.next_id == new_move.white.id).unwrap() + 1
                                }
                            };
                    moves.splice(splice_index..splice_index, [new_move.clone()].into_iter());
                },
                true => {
                     let splice_index = moves
                            .clone()
                            .iter()
                            .position(|x| x.black.id == current_move.id)
                            .unwrap() + 1;

                    new_move.depth = new_move.depth + 1;
                    moves.splice(splice_index..splice_index, [new_move.clone()].into_iter());
                }
            }
        }
    };

    handle.set_current_move(san_move.clone());
    handle.set_last_move_in_variation(san_move.id);

    moves
}

fn is_new_variation(ui: &AppWindow, moves: &Vec<SanMoveRow>) -> bool {
    let ui_handle = ui.as_weak();
    let handle = ui_handle.unwrap();
    let current_move = handle.get_current_move();
    let last_move_in_variation = handle.get_last_move_in_variation();

    (current_move.id != last_move_in_variation || moves.is_empty()) && current_move.next_id != 0
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
