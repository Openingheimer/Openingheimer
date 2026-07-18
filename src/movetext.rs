#![allow(dead_code)]
#![allow(unused_variables)]

use std::rc::Rc;
use slint::Model;
use slint::{VecModel};
use crate::{SanMoveRow, SanMove, model::*};
use crate::pgn_import::PgnReader;

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