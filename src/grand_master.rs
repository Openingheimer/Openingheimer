use slint::SharedString;

use crate::fen_master::*;
use crate::model::*;

pub fn try_make_move(start_fen: SharedString, start_square: SharedString, end_square: SharedString) -> (bool, SharedString, SharedString) {

	let mut fenboard = to_fenboard(start_fen, start_square, end_square);

	let move_result = make_move(fenboard.clone());

	if move_result.sucess {
		fenboard.piece_placement = move_result.piece_placement.into();
		fenboard.en_passant = move_result.en_passant.into();
		add_ply(&mut fenboard);
	}

	(move_result.sucess, fenboard.to_fen(), fenboard.active_color.as_str().into())
}

fn make_move(fenboard: FenBoard) -> MoveResult {

	let move_result = match fenboard.from_piece.clone() {
		Some(p) => can_make_move(fenboard.clone(), p),
		_ => MoveResult {
			sucess:  false,
			piece_placement: fenboard.piece_placement.clone().into(),
			en_passant: fenboard.en_passant.clone().into(),
			en_passant_capture: false
		},
	};

	let mut piece_placement: Vec<char> = fenboard.piece_placement.chars().collect();

	let mut en_passant = fenboard.en_passant.clone().into();

    if move_result.sucess {
        apply_move(&fenboard, &move_result, &mut piece_placement, &mut en_passant);
    }

	MoveResult {
		sucess: move_result.sucess,
		piece_placement: piece_placement.into_iter().collect(),
		en_passant: en_passant.to_string(),
		en_passant_capture: move_result.en_passant_capture
	 }
}

fn can_make_move(fenboard: FenBoard, piece: Piece) -> MoveResult {

	let mut move_result = piece.is_legal_move(fenboard.clone());

	if piece.color != fenboard.active_color ||
	   fenboard.from64 == fenboard.to64 ||
	   friendly_fire(piece.clone(), fenboard.to_piece) ||
	   move_result.sucess == false {

		move_result.sucess = false;
	}

	move_result.clone()
}

pub fn try_pawn_move(pawn: Piece, fenboard: FenBoard) -> MoveResult {

	let (pawn_capture, en_passant) = is_pawn_capture(pawn.clone(), fenboard.clone());
	let legal_move = pawn_capture || is_marching_forward(pawn.clone(), fenboard);

	MoveResult { sucess: legal_move, en_passant_capture: en_passant, ..Default::default() }
}

fn is_marching_forward(pawn: Piece, fenboard: FenBoard) -> bool {

	let can_move_two = match pawn.color {
		Color::White => (48..=55).contains(&fenboard.from64),
		Color::Black => (8..=15).contains(&fenboard.from64),
	};

	let requested_move = (fenboard.from64 - fenboard.to64).abs();
	let moving_two = can_move_two && requested_move == 16;
	let moving_forward = forwards_movement(pawn.color.clone(), fenboard.from64, fenboard.to64);

	let legal_move = moving_forward && (requested_move == 8 || (can_move_two && requested_move == 16));
	let clear_path = path_is_clear(fenboard.fen64, get_pawn_path(fenboard.from64, fenboard.to64, moving_two, pawn.color.clone()));

	clear_path && legal_move
}

fn is_pawn_capture(pawn: Piece, fenboard: FenBoard) -> (bool, bool) {

	let requested_move = (fenboard.from64 - fenboard.to64).abs();

	let a_pawn = fenboard.from64 % 8 == 0;
	let h_pawn = (fenboard.from64 + 1) % 8 == 0;
	let rook_pawn = a_pawn || h_pawn;

	let attempted_capture = match rook_pawn {
		false => requested_move == 7 || requested_move == 9,
		true => match a_pawn {
			true => match pawn.color.clone() {
				Color::White => requested_move == 7,
				Color::Black => requested_move == 9,
			},
			false => match pawn.color.clone() {
				Color::White => requested_move == 9,
				Color::Black => requested_move == 7,
			},
		},
	} && forwards_movement(pawn.color.clone(), fenboard.from64, fenboard.to64);

	match fenboard.to_piece {
		Some(p) if p.color != pawn.color && attempted_capture  => (true, false),
		None => {
			match fenboard.en_passant.clone().as_str() {
				"-" => (false, false),
				_ => {
				    let en_passant_capture = square_to_index64(fenboard.en_passant.clone().into()) == fenboard.to64 && attempted_capture;

					(en_passant_capture, en_passant_capture)
				}
			}
		}
		_ => (false, false),
	}
}

fn forwards_movement(color: Color, from64: i32, to64: i32) -> bool {
	match color {
		Color::White => from64 > to64,
		Color::Black => from64 < to64
	}
}

fn get_pawn_path(from64: i32, to64: i32, moving_two: bool, color: Color) -> Vec<i32> {
	let mut path = vec![to64];

	if moving_two {
		match color {
		    Color::White => path.push(from64 - 8),
	        Color::Black => path.push(from64 + 8),
		}
	}

	path
}

fn friendly_fire(piece: Piece, destination_piece: Option<Piece>) -> bool {

	if let Some(dp) = destination_piece {
		if piece.color == dp.color{
			return true;
		}
	}

	false
}

fn path_is_clear(fen64: SharedString, squares: Vec<i32>) -> bool {
	 squares
        .iter()
        .all(|&square| get_piece_from_fen_code(fen64.chars().nth(square as usize).unwrap()).is_none())
}