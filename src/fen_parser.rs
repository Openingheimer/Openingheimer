use crate::grand_master::*;
use std::iter::repeat;

struct Fen {
	piece_placement: String,
	active_color: String,
	castling_availablity: String,
	en_passant: String,
	half_move_clock: i32,
	full_move_number: i32
}

impl Fen {
    fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            unpad_empty_squares(self.piece_placement.clone()),
            self.active_color,
            self.castling_availablity,
            self.en_passant,
            self.half_move_clock,
            self.full_move_number
        )
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

pub fn get_piece_from_fen64(fen: String, index: i32) -> Option<Piece> {

	let piece_placement: Vec<char> = fen.chars().collect();

	get_piece_from_fen(piece_placement[index as usize])
}

pub fn get_piece_from_fen(fen: char) -> Option<Piece> {

	let color = match fen {
		c if c.is_uppercase() => Color::White,
		_ => Color::Black
	};

	let piece_type = match fen.to_ascii_lowercase() {
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

fn parse_fen(fen: String) -> Fen {

	let fen_fields : Vec<&str> = fen.split(' ').collect();

	Fen {
		piece_placement: pad_empty_squares(fen_fields[0].to_string()),
		active_color: fen_fields[1].to_string(),
		castling_availablity: fen_fields[2].to_string(),
		en_passant: fen_fields[3].to_string(),
		half_move_clock: fen_fields[4].parse::<i32>().unwrap(),
		full_move_number: fen_fields[5].parse::<i32>().unwrap(),
	}
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

