
use crate::fen_master::*;
use crate::{SanMoveRow, SanMove, model::MoveReader, model::MoveNode};
use slint::{Model, ModelRc, VecModel};
use slint::{ToSharedString};
use std::rc::Rc;

pub fn to_san_move_rows(reader: &mut MoveReader, move_color: shakmaty::Color) {

    let current_move = reader.moves[reader.current.unwrap()].clone();

    match move_color {
        shakmaty::Color::White => {
           let prev_move_row = reader.san_move_rows
                    .iter()
                    .enumerate()
                    .find(|(_, x)| x.black.id == current_move.previous.unwrap() as i32);

            if let Some((pi, prev_row)) = prev_move_row && reader.is_new_var == false {

                let mut pv = prev_row.clone();
                pv.black.next_id = reader.current.unwrap() as i32;
                reader.san_move_rows[pi] = pv;
            }

            match reader.is_new_var {
                false => {

                    reader.turn_number += 1;
                    let move_row = create_white_move_row(reader.clone(), current_move);
                    reader.san_move_rows.push(move_row)
                },
                true => {

                    let pick_up_from = reader.pick_up_position_from.last().unwrap().clone();
                    let prev_white_move = reader.moves[pick_up_from].clone();

                    let (pi, prev_move_row) = reader.san_move_rows
                        .iter()
                        .enumerate()
                        .find(|(_, x)| x.white.id == pick_up_from as i32)
                        .unwrap()
                        .to_owned();

                    let mut updated_row = prev_move_row.clone();

                    updated_row.white.variations = get_variations(&prev_white_move.variations);

                    let move_row = create_white_move_row(reader.clone(), current_move);

                    reader.san_move_rows[pi] = updated_row.clone();
                    reader.san_move_rows.push(move_row)
                }
            }
        },
        shakmaty::Color::Black => {

            let (pi, prev_move_row) = reader.san_move_rows
                .iter()
                .enumerate()
                .rev()
                .find(|(_i,x)| x.white.id == current_move.previous.clone().map(|x| x as i32).unwrap_or(-2) ||
                              (reader.is_new_var && x.white.id == reader.current.unwrap() as i32))
                .unwrap()
                .to_owned();

            let mut updated_row = prev_move_row.clone();

            updated_row.white.next_id = reader.current.unwrap() as i32;

            let black_move = SanMove {
               id: reader.current.unwrap() as i32,
               san_text: current_move.san.to_shared_string(),
               fen: fen_from_position(&current_move.position),
               next_id: -1,
               previous_id: updated_row.white.id,
               variations: get_variations(&current_move.variations),
               parent_line: current_move.parent_line.map(|x| x as i32).unwrap_or(-1),
               hide_move: true,
               variation_id: current_move.variation_id,
               from_square: current_move.from_square.to_shared_string(),
               to_square: current_move.to_square.to_shared_string(),
            };

            match reader.is_new_var {
                false => {
                    match reader.is_end_var {
                        false => {
                            updated_row.black = black_move;
                            reader.san_move_rows[pi] = updated_row.clone();
                        },
                        true => {
                           let new_row = SanMoveRow {
                                white: empty_san_move(),
                                black: black_move,
                                turn: reader.turn_number,
                                depth: reader.depth,
                            };

                            reader.san_move_rows.push(new_row);
                            reader.san_move_rows[pi] = updated_row.clone();
                        }
                    }
                },
                true =>{
                    let new_row = SanMoveRow {
                            white: empty_san_move(),
                            black: black_move,
                            turn: reader.turn_number,
                            depth: reader.depth,
                        };

                    let updated = prev_move_row.clone();

                    if let Some(model) = updated.black.variations
                        .as_any()
                        .downcast_ref::<VecModel<i32>>() {

                       model.push(reader.current.unwrap() as i32);
                    }

                    reader.san_move_rows.push(new_row);
                    reader.san_move_rows[pi] = updated.clone();
                }
            }
        }
    }
}

 fn create_white_move_row(reader: MoveReader, current_move: MoveNode) -> SanMoveRow {
    SanMoveRow {
        white: SanMove {
            san_text: current_move.san.to_shared_string(),
            fen: fen_from_position(&current_move.position),
            id: reader.current.unwrap() as i32,
            next_id: match current_move.next {
                Some(i) => i as i32,
                None => -1
            },
            previous_id: match current_move.previous {
                Some(i) => i as i32,
                None => -1
            },
            variations: get_variations(&current_move.variations),
            parent_line: current_move.parent_line.map(|x| x as i32).unwrap_or(-1),
            hide_move: true,
            variation_id: current_move.variation_id,
            from_square: current_move.from_square.to_shared_string(),
            to_square: current_move.to_square.to_shared_string(),
        },
        black: empty_san_move(),
        depth: reader.depth,
        turn: reader.turn_number,
    }
 }

 pub fn get_variations(variations: &Vec<usize>) -> ModelRc<i32> {
    Rc::new(slint::VecModel::from(variations
                    .iter()
                    .map(|&v| v as i32)
                    .collect::<Vec<i32>>())).into()
 }

 fn empty_san_move() -> SanMove {
    SanMove {
        id: -1,
        next_id: -1,
        previous_id: -1,
        fen: "".into(),
        san_text: "".into(),
        variations: Rc::new(slint::VecModel::from([].to_vec())).into(),
        parent_line: -1,
        hide_move: true,
        variation_id: -1,
        from_square: "".into(),
        to_square: "".into(),
    }
 }