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
pub struct MoveResult {
	sucess: bool,
	reason: String, //todo remove
	piece_placement: String,
	en_passant: String,
}

#[derive(Clone)]
pub struct Piece {
	pub piece_type: PieceType,
	pub color: Color,
}

pub trait PieceBrain {
	#[allow(dead_code)]
    fn as_value(&self) -> i32;
	fn as_fen(&self) -> char;
	fn is_legal_move(&self, fen64: String, from64: i32, to64: i32) -> bool;
}

impl PieceBrain for Piece {
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

	fn is_legal_move(&self, fen64: String, from64: i32, to64: i32) -> bool {

        match self.piece_type {
            PieceType::Pawn => can_pawn_move(self.clone(), fen64, from64, to64),
            // PieceType::Bishop => 'b',
            // PieceType::Knight => 'n',
            // PieceType::Rook => 'r',
            // PieceType::Queen => 'q',
            // PieceType::King => 'k',
			_ => false
        }
    }
}

pub fn try_making_move(start_fen: SharedString, start_square: SharedString, end_square: SharedString) -> (bool, SharedString, SharedString) {

	let from64 = square_to_index64(start_square);
	let to64 = square_to_index64(end_square);

	let mut fen = parse_fen(start_fen);
	let player_turn = match fen.active_color.as_str() {
		"w" => Color::White,
		_ => Color::Black
	};

	let move_result = try_move(fen.piece_placement.clone(), from64, to64, player_turn);

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

pub fn try_move(padded_piece_placement: String, start64: i32, to64: i32, turn: Color) -> MoveResult {

	let mut piece_placement: Vec<char> = padded_piece_placement.chars().collect();

	let start_fen_index = offset_slashes(start64);
	let to_fen_index = offset_slashes(to64);

	let current_square_fen = piece_placement[start_fen_index];
	let destination_square_fen = piece_placement[to_fen_index];

	let piece = get_piece_from_fen(current_square_fen);
	let destination_piece = get_piece_from_fen(destination_square_fen);

	let (move_made, reason) = match &piece {
		Some(p) =>  can_make_move(padded_piece_placement.replace("/", ""), p.clone(), destination_piece, start64, to64, turn),
		_ => (false, "No Piece Selected".into()),
	};

	let mut en_passant = "-".to_string();

	if move_made {
		piece_placement[to_fen_index] = current_square_fen;
		piece_placement[start_fen_index] = '.';
		en_passant = get_en_passant(piece.unwrap().clone(), start64, to64);
	}

	MoveResult {
		sucess: move_made,
		reason: reason,
		piece_placement: piece_placement.into_iter().collect(),
		en_passant: en_passant.to_string(),
	 }
}

fn can_make_move(
	fen64: String,
	piece: Piece,
	destination_piece: Option<Piece>,
	start64: i32,
	end64: i32,
	turn: Color) -> (bool, String) {

	if piece.color != turn {
		return (false, "Not Your Turn".into());
	}

	if start64 == end64 {
		return (false, "Cannot move to the same square".into());
	}

	if friendly_fire(piece.clone(), destination_piece) {
		return (false, "Friendly Piece Occupies Square".to_string());
	}

	if piece.is_legal_move(fen64, start64, end64) == false {
		return (false, piece.as_fen().to_string() + " cannot move to destination square");
	}

	(true, String::new())
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

fn offset_slashes(index: i32) -> usize {
	(index + (index / 8)) as usize
}

fn square_to_index64(square: SharedString) -> i32 {

	let chars: Vec<char> = square.to_lowercase().chars().collect();

	let file = (chars[0] as u8 - b'a') as i32;
	let rank = chars[1].to_digit(10).unwrap() as i32;

	((8 - rank) * 8) + file
}

fn can_pawn_move(pawn: Piece, fen64: String, from64: i32, to64: i32) -> bool {

	let can_move_two = match pawn.color {
		Color::White => (48..=55).contains(&from64),
		Color::Black => (8..=15).contains(&from64),
	};

	let requested_move = (from64 - to64).abs();
	let is_moving_two = can_move_two && requested_move == 16;
	let move_is_legal = requested_move == 8 || (can_move_two && requested_move == 16);
	let clear_path = path_is_clear(fen64.clone(), get_pawn_path(from64, to64, is_moving_two, pawn.color.clone()));

	//todo en passant
	(clear_path && move_is_legal) ||
	is_pawn_capture(fen64.clone(), from64, to64, pawn.color)
}

fn is_pawn_capture(fen64: String, from64: i32, to64: i32, color: Color) -> bool {

	let piece_at_destination = get_piece_from_fen64(fen64, to64);
	let requested_move = (from64 - to64).abs();

	if let Some(p) = piece_at_destination {
		if p.color != color && (requested_move == 7 || requested_move == 9) {
			return true;
		}
	}

	false
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
        .all(|&square| get_piece_from_fen(fen.chars().nth(square as usize).unwrap()).is_none())
}