
use std::rc::Rc;
use slint::Model;
use slint::VecModel;

use crate::{SanMoveRow, SanMove, model::*};

pub fn update_move_list(move_request: MoveRequest) -> MoveResponse {

    let mut moves = move_request.moves.clone();
    let current_move = move_request.current_move.clone();
    let fenboard = move_request.fenboard.clone();
    let last_move_in_variation = move_request.last_move_in_variation.clone();

    let new_variation = (current_move.id != last_move_in_variation || moves.is_empty()) && current_move.next_id != 0;

    let variation = match new_variation {
        true => next_variation_id(),
        false => current_move.variation
    };

    if moves.is_empty() {
       let first_move = create_first_move(&fenboard, &current_move, next_variation_id());

       moves.push(first_move.clone());

       let response = MoveResponse {
            current_move: first_move.white.clone(),
            moves: moves,
            last_move_in_variation: first_move.white.id,
            scroll_y: 0.0,
            scroll_x: 0.0,
       };

       return response;
    }

    let new_move = create_ply(&fenboard, &current_move, &mut moves, variation, new_variation);

    let san_move = match fenboard.active_color {
        Color::White => new_move.black.clone(),
        Color::Black => new_move.white.clone(),
    };

    let (x, y) = get_scroll_point(moves.clone(), current_move.id, new_move.depth);

    MoveResponse {
            current_move: san_move.clone(),
            moves: moves,
            last_move_in_variation: san_move.id,
            scroll_x: x,
            scroll_y: y,
       }
}

pub fn create_ply(fenboard: &FenBoard, current_move: &SanMove, moves: &mut Vec<SanMoveRow>, variation: i32, new_variation: bool) -> SanMoveRow {

    let mut new_move = complete_black_move_or_get_new(fenboard.clone(), moves, &current_move, variation, new_variation);

    let (mut last_move, index) = get_last_move_in_variation(&moves, &current_move, variation);

    match fenboard.active_color {
        Color::White => {
            match new_variation {
                false => moves[index] = new_move.clone(),
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
                            .position(|x| x.white.id == current_move.next_id)
                            .unwrap() + 1;

                    new_move.depth = new_move.depth + 1;
                    moves.splice(splice_index..splice_index, [new_move.clone()].into_iter());
                }
            }
        }
    };

    new_move
}

fn get_last_move_in_variation(moves: &Vec<SanMoveRow>, current_move: &SanMove, variation: i32) -> (SanMoveRow, usize) {

    let index = moves
        .clone()
        .iter()
        .rposition(|x| x.white.variation == variation);

    match index {
        Some(m) => (moves[m].clone(), index.unwrap() as usize),
        None => moves.clone().into_iter()
        .enumerate()
        .find(|(_i, x)| x.white.id == current_move.id || x.black.id == current_move.id)
        .map(|(i, x)|  (x, i))
        .unwrap()
    }
}

fn complete_black_move_or_get_new(fenboard: FenBoard, moves: &mut Vec<SanMoveRow>,
    current_move: &SanMove,
    variation: i32,
    new_variation: bool) -> SanMoveRow {

    let (last_move, _) = get_last_move_in_variation(&moves, &current_move, variation);

    match fenboard.active_color {
        Color::White => create_black_move(fenboard, current_move, moves, variation, new_variation),
        Color::Black => SanMoveRow {
            white: create_white_move(&fenboard, current_move, moves, variation, new_variation),
            black: SanMove::default(),
            depth: last_move.depth,
            turn: last_move.turn + 1,
        }
    }
}

fn create_white_move(fenboard: &FenBoard, current_move: &SanMove, moves: &mut Vec<SanMoveRow>, variation: i32, new_variation: bool) -> SanMove {

    let mut branches: Vec<i32> = current_move.parent_branches
        .clone()
        .as_any()
        .downcast_ref::<VecModel<i32>>()
        .map(|m| m.iter().collect())
        .unwrap_or_default();

    match new_variation {
        true => branches.push(current_move.variation),
        false => ()
    };

    let id = next_id();

    let last_move_in_variation = moves
        .clone()
        .iter()
        .rposition(|x| x.black.variation == variation);

     match last_move_in_variation {
        Some(m) => {
            let mut last_move = moves[m].clone();
            last_move.black.next_id = id;
            moves[m] = last_move;
        },
        None => ()
    };

    SanMove {
            id: id,
            fen: fenboard.to_fen(),
            san_text: fenboard.san_move.clone(),
            variation: variation,
            previous_id: current_move.id,
            next_id: 0,
            parent_branches: Rc::new(slint::VecModel::from(branches)).into(),
        }
}

fn create_black_move(fenboard: FenBoard,  current_move: &SanMove, moves: &mut Vec<SanMoveRow>, variation: i32, new_variation: bool) -> SanMoveRow {

    let last_move_in_variation = moves
        .iter()
        .rposition(|x| x.white.variation == variation);

    let mut last_move = match last_move_in_variation {
        Some(m) => moves[m].clone(),
        None => moves.iter().find(|x| x.white.id == current_move.id).unwrap().clone()
    };

    let id = next_id();

    if current_move.variation == variation {
        last_move.white.next_id = id;
    }

    let branches: VecModel<i32> = current_move.parent_branches
                .clone()
                .as_any()
                .downcast_ref::<VecModel<i32>>()
                .unwrap()
                .iter()
                .collect();

    match new_variation {
        true => branches.push(current_move.variation),
        false => ()
    };

    SanMoveRow {
        turn: last_move.turn,
        depth: last_move.depth,
        white: last_move.white.clone(),
        black: SanMove {
            id: id,
            fen: fenboard.to_fen(),
            san_text: fenboard.san_move,
            variation: variation,
            previous_id: last_move.white.id,
            next_id: 0,
            parent_branches: Rc::new(slint::VecModel::from(branches)).into()
        }
    }
}

pub fn create_first_move(fenboard: &FenBoard, current_move: &SanMove,  variation: i32) -> SanMoveRow {

    let mut empty: Vec<SanMoveRow> = Vec::new();
    let moves: &mut Vec<SanMoveRow> = &mut empty;

    let white = create_white_move(fenboard, current_move, moves, variation, false);
    let mut black = SanMove::default();

    black.previous_id = white.id;
    black.variation = variation;

    SanMoveRow {
        turn: 1,
        depth: 1,
        white: white,
        black: black
    }
}

 pub fn get_scroll_point(moves: Vec<SanMoveRow>, current_move_id: i32, depth: i32) -> (f32, f32) {

    let scroll_to = moves.clone()
        .iter()
        .position(|x| x.white.id == current_move_id || x.black.id == current_move_id)
        .unwrap() + 1;

    let row_height = 40.0;
    let move_text_height = 450.0;
    let right_pad = match depth.clone() {
        1 => 0.0,
        _ => 15.0
    };

    let y = -(scroll_to as f32 * row_height) + move_text_height - row_height - 15.0;
    let x = ((depth - 1) as f32 * -45.0) - right_pad;

    (x, y)
 }