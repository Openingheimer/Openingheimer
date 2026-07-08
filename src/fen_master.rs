use crate::model::*;
use std::iter::repeat;
use slint::SharedString;

 pub fn to_fenboard(fen: SharedString, from_coords: SharedString, to_coords: SharedString) -> FenBoard {

	let fen_fields : Vec<&str> = fen.split(' ').collect();

	let piece_placement = pad_empty_squares(fen_fields[0].to_string());

	let from64 = square_to_index64(from_coords.clone());
	let to64 = square_to_index64(to_coords.clone());

	let from_fen_index = offset_slashes(from64);
	let to_fen_index = offset_slashes(to64);

	let fen_piece_placement: Vec<char> = piece_placement.clone().chars().collect();

	let current_square_code = fen_piece_placement[from_fen_index];
	let destination_square_code = fen_piece_placement[to_fen_index];

	let piece = get_piece_from_fen_code(current_square_code);
	let destination_piece = get_piece_from_fen_code(destination_square_code);

	FenBoard {
		piece_placement: piece_placement.clone().into(),
		fen64: piece_placement.replace("/", "").into(),
		active_color: match fen_fields[1] {
			"w" => Color::White,
			_ => Color::Black
		},
		castling_availablity: fen_fields[2].to_string().into(),
		en_passant: fen_fields[3].to_string().into(),
		half_move_clock: fen_fields[4].parse::<i32>().unwrap(),
		full_move_number: fen_fields[5].parse::<i32>().unwrap(),
		from_piece: piece,
		to_piece: destination_piece,
		from_fen71: from_fen_index,
		to_fen71: to_fen_index,
		from64: from64,
		to64: to64,
		san_move: to_coords.into()
	}
}

 pub fn get_piece_code_from_fen(fen: String, cell_index: i32) -> String {

    let piece_placement = fen.split(' ').next().unwrap();
    let mut ranks = piece_placement.split('/');

    let rank_index = (cell_index / 8) as usize;
    let file_index = (cell_index % 8) as usize;

    let cell_rank = ranks.nth(rank_index).unwrap_or("");
    let row = pad_empty_squares(cell_rank.to_string());

    let value = row
        .chars()
        .nth(file_index)
        .unwrap_or('.');

    value.to_string().into()
}

 pub fn get_piece_from_fen_code(fen_code: char) -> Option<Piece> {

	let color = match fen_code {
		c if c.is_uppercase() => Color::White,
		_ => Color::Black
	};

	let piece_type = match fen_code.to_ascii_lowercase() {
	    'p' => Some(PieceType::Pawn),
	    'b' => Some(PieceType::Bishop),
	    'n' => Some(PieceType::Knight),
	    'r' => Some(PieceType::Rook),
	    'q' => Some(PieceType::Queen),
	    'k' => Some(PieceType::King),
	    _ => None,
	};

	match piece_type {
		Some(p) => Some(Piece { piece_type: p, color: color }),
		None => None,
	}
}

pub fn get_piece_color_from_square(fen: SharedString, square: SharedString) -> SharedString {

	let square64 = square_to_index64(square.clone());

    let piece_code = get_piece_code_from_fen(fen.into(), square64);

	 match piece_code.as_str() {
        "P" | "N" | "B" | "R" | "Q" | "K" => "w".into(),
        "p" | "n" | "b" | "r" | "q" | "k" => "b".into(),
        _ => SharedString::new(),
    }
}

pub fn square_to_index64(square: SharedString) -> i32 {

	let chars: Vec<char> = square.to_lowercase().chars().collect();

	let file = (chars[0] as u8 - b'a') as i32;
	let rank = chars[1].to_digit(10).unwrap() as i32;

	((8 - rank) * 8) + file
}

pub fn offset_slashes(index: i32) -> usize {
	(index + (index / 8)) as usize
}

fn pad_empty_squares(piece_placement: String) -> String {

	let mut fen = String::new();

    for c in piece_placement.chars() {
        match c.to_digit(10) {
            Some(n) => fen.extend(repeat('.').take(n as usize)),
            None => fen.push(c),
        }
    }

    fen
}

pub fn unpad_empty_squares(piece_placement: SharedString) -> SharedString {

	let mut fen = String::new();
    let mut empty_accum = 0;

    for c in piece_placement.chars() {
        match c {
            '.' => empty_accum += 1,
            _ =>
			{
				if empty_accum != 0 {
					fen.push_str(&empty_accum.to_string());
				}

				fen.push(c);
				empty_accum = 0;
			}
        }
    }

	if empty_accum != 0 {
		fen.push_str(&empty_accum.to_string());
	}

    fen.into()
}

pub fn apply_move(fenboard: &FenBoard, move_result: &MoveResult, fen_piece_placement: &mut Vec<char>, en_passant: &mut String) {

	fen_piece_placement[fenboard.to_fen71] = fenboard.from_piece.as_ref().unwrap().as_fen();
    fen_piece_placement[fenboard.from_fen71] = '.';

    if move_result.en_passant_capture {
        let capture_offset = match fenboard.active_color {
            Color::White => 8,
            Color::Black => -8,
        };

        let captured_pawn = offset_slashes(square_to_index64(fenboard.en_passant.clone()) + capture_offset);

        fen_piece_placement[captured_pawn] = '.';
    }

    *en_passant = get_en_passant(fenboard.from_piece.as_ref().unwrap().clone(), fenboard.from64, fenboard.to64);
}

pub fn add_ply(fenboard: &mut FenBoard) {
    match fenboard.active_color {
        Color::White => fenboard.active_color = Color::Black,
        Color::Black => {
            fenboard.full_move_number += 1;
            fenboard.active_color = Color::White;
        }
    }
}

pub fn get_en_passant(piece: Piece, start64: i32, to64: i32) -> String {

	let mut en_passant = "-".to_string();

	match piece.piece_type {
		PieceType::Pawn => {
			let moved_two_squares = (start64 - to64).abs() == 16;

			if moved_two_squares {
				let rank = (((start64 % 8)) as u8 + b'a') as char;

				en_passant = match piece.color {
					Color::White => format!("{}3", rank),
					Color::Black => format!("{}6", rank),
				}
			}

			en_passant
		},
		_ => en_passant
	}
}