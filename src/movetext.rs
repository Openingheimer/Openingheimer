
use std::rc::Rc;
use slint::Model;
use slint::VecModel;

use crate::{SanMoveRow, SanMove, model::*};

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