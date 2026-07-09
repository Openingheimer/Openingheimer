use slint::SharedString;

use crate::fen_master::*;
use crate::model::*;

pub fn try_make_move(start_fen: SharedString, start_square: SharedString, end_square: SharedString) -> MoveResult {

	let fenboard = to_fenboard(start_fen, start_square, end_square);

	make_move(fenboard)
}

fn make_move(fenboard: FenBoard) -> MoveResult {

	let mut move_result = match fenboard.from_piece.clone() {
		Some(piece) => can_make_move(fenboard, piece),
		_ => MoveResult {
			success: false,
			fenboard: fenboard
		},
	};

    if move_result.success {
        move_result = apply_move(move_result);
    }

	move_result
}

fn can_make_move(fenboard: FenBoard, piece: Piece) -> MoveResult {

	let mut move_result = piece.is_legal_move(fenboard.clone());

	if move_result.success == false ||
	   piece.color != fenboard.active_color ||
	   fenboard.from64 == fenboard.to64 ||
	   friendly_fire(piece.clone(), fenboard.to_piece) {

		move_result.success = false;
	}

	move_result.clone()
}

pub fn try_pawn_move(pawn: Piece, mut fenboard: FenBoard) -> MoveResult {

	let (pawn_capture, en_passant) = is_pawn_capture(pawn.clone(), fenboard.clone());
	let legal_move = pawn_capture || is_marching_forward(pawn.clone(), fenboard.clone());
	let promoted = is_promotion(fenboard.to64, fenboard.active_color.clone());

	fenboard.en_passant_capture = en_passant;

	fenboard.move_type = match pawn_capture {
		true if promoted => MoveType::CapturePromotion,
		false if promoted => MoveType::Promotion,
		true => MoveType::Capture,
		false => MoveType::Normal,
	};

	MoveResult { success: legal_move, fenboard: fenboard}
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

fn is_promotion(to64: i32, color:Color) -> bool {

	match color {
		Color::White => to64 < 8,
		Color::Black => to64 > 55,
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