use slint::SharedString;

use crate::fen_parser::*;

#[derive(Clone)]
pub enum PieceType {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King
}

#[derive(PartialEq)]
#[derive(Clone)]
pub enum Color {
	White,
	Black
}

#[derive(Clone)]
#[derive(Default)]
pub struct MoveResult {
	sucess: bool,
	piece_placement: String,
	en_passant: String,
	en_passant_capture: bool,
}

#[derive(Clone)]
pub struct Piece {
	pub piece_type: PieceType,
	pub color: Color,
}

#[allow(dead_code)]
pub trait PieceBrain {
    fn as_value(&self) -> i32;
	fn as_fen(&self) -> char;
	fn is_legal_move(&self, fen64: String, from64: i32, to64: i32, en_passant: String) -> MoveResult;
}

impl PieceBrain for Piece {

	fn is_legal_move(&self, fen64: String, from64: i32, to64: i32, en_passant: String) -> MoveResult {

        match self.piece_type {
            PieceType::Pawn => try_pawn_move(self.clone(), fen64, from64, to64, en_passant),
            // PieceType::Bishop => 'b',
            // PieceType::Knight => 'n',
            // PieceType::Rook => 'r',
            // PieceType::Queen => 'q',
            // PieceType::King => 'k',
			_ => MoveResult { sucess: false, piece_placement: fen64 , en_passant: en_passant, en_passant_capture: false}
        }
    }

    fn as_value(&self) -> i32 {
        match self.piece_type {
            PieceType::Pawn => 1,
            PieceType::Bishop => 3,
            PieceType::Knight => 3,
            PieceType::Rook => 5,
            PieceType::Queen => 10,
            PieceType::King => 0,
        }
    }

	fn as_fen(&self) -> char {
        let piece = match self.piece_type {
            PieceType::Pawn => 'p',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        };

		match self.color {
			Color::White => piece.to_ascii_uppercase(),
			_ => piece,
		}
    }
}

pub fn try_make_move(start_fen: SharedString, start_square: SharedString, end_square: SharedString) -> (bool, SharedString, SharedString) {

	let from64 = square_to_index64(start_square);
	let to64 = square_to_index64(end_square);

	let mut fen = parse_fen(start_fen);

	let player_color = match fen.active_color.as_str() {
		"w" => Color::White,
		_ => Color::Black
	};

	let move_result = make_move(fen.piece_placement.clone(), from64, to64, player_color, fen.en_passant.clone());

	if move_result.sucess {
		fen.piece_placement = move_result.piece_placement;
		fen.en_passant = move_result.en_passant;

		match fen.active_color.as_str() {
		    "w" => fen.active_color = "b".to_string(),
		    _ => {
		        fen.full_move_number += 1;
		        fen.active_color = "w".to_string();
		    }
		}
	}

	(move_result.sucess, fen.to_fen(), fen.active_color.into())
}

fn make_move(piece_placement: String, start64: i32, to64: i32, turn: Color, en_passant_avail: String) -> MoveResult {

	let mut fen_piece_placement: Vec<char> = piece_placement.chars().collect();

	let start_fen_index = offset_slashes(start64);
	let to_fen_index = offset_slashes(to64);

	let current_square_code = fen_piece_placement[start_fen_index];
	let destination_square_code = fen_piece_placement[to_fen_index];

	let piece = get_piece_from_code(current_square_code);
	let destination_piece = get_piece_from_code(destination_square_code);

	let move_result = match &piece {
		Some(p) =>  can_make_move(
			piece_placement.replace("/", ""),
			p.clone(),
			destination_piece,
			start64,
			to64,
			turn.clone(),
			en_passant_avail.clone()),
		_ => MoveResult {
			sucess:  false,
			piece_placement: piece_placement,
			en_passant: en_passant_avail.clone(),
			en_passant_capture: false
		},
	};

	let mut en_passant = en_passant_avail.clone();

	if move_result.sucess {
		fen_piece_placement[to_fen_index] = current_square_code;
		fen_piece_placement[start_fen_index] = '.';

		if move_result.en_passant_capture {

			let capture_offset: i32 = match turn {
				Color::White  => 8,
				Color::Black => -8
			};

			let captured_pawn = offset_slashes(square_to_index64(en_passant_avail.into()) + capture_offset);

			fen_piece_placement[captured_pawn] = '.';
		}
		en_passant = get_en_passant(piece.unwrap().clone(), start64, to64);
	}

	MoveResult {
		sucess: move_result.sucess,
		piece_placement: fen_piece_placement.into_iter().collect(),
		en_passant: en_passant.to_string(),
		en_passant_capture: move_result.en_passant_capture
	 }
}

fn can_make_move(
	fen64: String,
	piece: Piece,
	destination_piece: Option<Piece>,
	from64: i32,
	to64: i32,
	turn: Color,
	en_passant_avail: String) -> MoveResult {

	let mut move_result = piece.is_legal_move(fen64, from64, to64, en_passant_avail);

	if piece.color != turn ||
	   from64 == to64 ||
	   friendly_fire(piece.clone(), destination_piece) ||
	   move_result.sucess == false {

		move_result.sucess = false;
	}

	move_result.clone()
}

fn friendly_fire(piece: Piece, destination_piece: Option<Piece>) -> bool {

	if let Some(dp) = destination_piece {
		if piece.color == dp.color{
			return true;
		}
	}

	false
}

fn get_en_passant(piece: Piece, start64: i32, to64: i32) -> String {

	let mut en_passant = "-".to_string();

	match piece.piece_type {
		PieceType::Pawn => {
			let moved_two_squares = (start64 - to64).abs() == 16;

			if moved_two_squares {
				let rank = (((start64 % 8)) as u8 + b'a') as char;

				en_passant = match piece.color {
					Color::White => format!("{}3", rank),
					_ => format!("{}6", rank),
				}
			}

			en_passant
		},
		_ => en_passant
	}
}

fn try_pawn_move(pawn: Piece, fen64: String, from64: i32, to64: i32, en_passant: String) -> MoveResult {

	let marching_forward = is_marching_forward(pawn.clone(), fen64.clone(), from64, to64);
	let (pawn_capture, en_passant) = is_pawn_capture(fen64.clone(), from64, to64, pawn.color, en_passant);

	let legal_move = marching_forward || pawn_capture;

	MoveResult { sucess: legal_move, en_passant_capture: en_passant, ..Default::default() }
}

fn is_marching_forward(pawn: Piece, fen64: String, from64: i32, to64: i32) -> bool {

	let can_move_two = match pawn.color {
		Color::White => (48..=55).contains(&from64),
		Color::Black => (8..=15).contains(&from64),
	};

	let requested_move = (from64 - to64).abs();
	let moving_two = can_move_two && requested_move == 16;
	let moving_forward = forwards_movement(pawn.color.clone(), from64, to64);

	let legal_move = moving_forward && (requested_move == 8 || (can_move_two && requested_move == 16));
	let clear_path = path_is_clear(fen64.clone(), get_pawn_path(from64, to64, moving_two, pawn.color.clone()));

	return clear_path && legal_move
}

fn is_pawn_capture(fen64: String, from64: i32, to64: i32, color: Color, en_passant: String) -> (bool, bool) {

	let piece_at_destination = get_piece_from_fen64(fen64, to64);
	let requested_move = (from64 - to64).abs();

	let a_pawn = from64 % 8 == 0;
	let h_pawn = (from64 + 1) % 8 == 0;
	let rook_pawn = a_pawn || h_pawn;

	let attempted_capture = match rook_pawn {
		false => requested_move == 7 || requested_move == 9,
		true => match a_pawn {
			true => match color {
				Color::White => requested_move == 7,
				Color::Black => requested_move == 9,
			},
			false => match color {
				Color::White => requested_move == 9,
				Color::Black => requested_move == 7,
			},
		},
	} && forwards_movement(color.clone(), from64, to64);

	match piece_at_destination {
		Some(p) if p.color != color && attempted_capture  => (true, false),
		None => {
			match en_passant.as_str() {
				"-" => (false, false),
				_ => {
				    let en_passant_capture = square_to_index64(en_passant.clone().into()) == to64 && attempted_capture;

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

fn path_is_clear(fen: String, squares: Vec<i32>) -> bool {
	 squares
        .iter()
        .all(|&square| get_piece_from_code(fen.chars().nth(square as usize).unwrap()).is_none())
}