use crate::grand_master::*;
use std::iter::repeat;
use slint::SharedString;

pub struct Fen {
	pub fen64: String,
	pub active_color: String,
	pub castling_availablity: String,
	pub en_passant: String,
	pub half_move_clock: i32,
	pub full_move_number: i32
}

impl Fen {
    pub fn to_fen(&self) -> SharedString {
        format!(
            "{} {} {} {} {} {}",
            unpad_empty_squares(self.fen64.clone()),
            self.active_color,
            self.castling_availablity,
            self.en_passant,
            self.half_move_clock,
            self.full_move_number
        ).into()
    }
}

 pub fn parse_fen(fen: SharedString) -> Fen {

	let fen_fields : Vec<&str> = fen.split(' ').collect();

	Fen {
		fen64: pad_empty_squares(fen_fields[0].to_string()),
		active_color: fen_fields[1].to_string(),
		castling_availablity: fen_fields[2].to_string(),
		en_passant: fen_fields[3].to_string(),
		half_move_clock: fen_fields[4].parse::<i32>().unwrap(),
		full_move_number: fen_fields[5].parse::<i32>().unwrap(),
	}
}

 pub fn get_piece_from_fen64(fen64: String, index: i32) -> Option<Piece> {

	let piece_placement: Vec<char> = fen64.chars().collect();

	get_piece_from_code(piece_placement[index as usize])
}

 pub fn get_piece_from_code(fen_code: char) -> Option<Piece> {

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

pub fn get_piece_color_from_square(fen: SharedString, square: SharedString) -> SharedString {

	let square64 = square_to_index64(square.clone());

    let piece_code = get_piece_code_from_fen(fen.into(), square64);

	get_piece_color(piece_code).into()
}

fn get_piece_color(piece_code: String) -> String {
    match piece_code.as_str() {
        "P" | "N" | "B" | "R" | "Q" | "K" => "w".to_string(),
        "p" | "n" | "b" | "r" | "q" | "k" => "b".to_string(),
        _ => String::new(),
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

fn unpad_empty_squares(piece_placement: String) -> String {

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

    fen
}

