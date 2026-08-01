use crate::model::*;
use crate::grand_master::*;
use std::iter::repeat;
use slint::{SharedString, ToSharedString};
use shakmaty::{fen::Fen, EnPassantMode};
use shakmaty::Chess;

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

	let piece = get_piece_from_fen_code(&current_square_code);
	let destination_piece = get_piece_from_fen_code(&destination_square_code);

	let fen64 = piece_placement.replace("/", "");

	let our_color =  match fen_fields[1] {
			"w" => Color::White,
			_ => Color::Black
		};

	let pinned_pieces = get_pinned_pieces(&fen64.chars().collect(), our_color.clone(), to64);

	FenBoard {
		piece_placement: piece_placement.clone().into(),
		fen64: fen64.to_shared_string(),
		active_color: our_color.clone(),
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
		en_passant_capture: false,
		from_coords: from_coords,
		to_coords: to_coords,
		move_type: MoveType::Normal,
		san_move: "".into(),
		pinned_pieces: pinned_pieces,
		start_fen64: fen64.to_shared_string(),
	}
}

pub fn to_fen64(fen: SharedString) -> SharedString {

	let fen_fields : Vec<&str> = fen.split(' ').collect();
	let piece_placement = pad_empty_squares(fen_fields[0].to_string());

	piece_placement.replace("/", "").into()
}

pub fn apply_move(mut move_result: MoveResult) -> MoveResult {

	let mut aborted_move = move_result.clone();
	aborted_move.success = false;

    update_piece_placement(&mut move_result.fenboard);
    update_castle_rights(&mut move_result.fenboard);
    update_ply(&mut move_result.fenboard);
    update_en_passant(&mut move_result.fenboard);
    update_move_type(&mut move_result.fenboard);
    update_san_move(&mut move_result.fenboard);

	let our_king_still_in_check = is_king_in_check(move_result.fenboard.piece_placement.clone(), &&aborted_move.fenboard.active_color);

	match our_king_still_in_check {
		true => aborted_move,
		false => move_result
	}
}

fn update_piece_placement(fenboard: &mut FenBoard) {

    let from_piece = fenboard.from_piece.as_ref().unwrap();

    let mut piece_placement: Vec<char> = fenboard.piece_placement.chars().collect();

    piece_placement[fenboard.to_fen71] = from_piece.as_fen();
    piece_placement[fenboard.from_fen71] = '.';

	update_castled_rooks(fenboard.clone(), &mut piece_placement);
	update_en_passant_piece_placement(fenboard.clone(), &mut piece_placement);

	if fenboard.move_type == MoveType::Promotion || fenboard.move_type == MoveType::CapturePromotion {
		piece_placement[fenboard.to_fen71] = get_promotion_piece(fenboard.from_piece.clone().unwrap().color);
	}

    fenboard.piece_placement = piece_placement.into_iter().collect::<String>().into();
}

fn update_castled_rooks(fenboard: FenBoard, piece_placement: &mut Vec<char>) {

	if fenboard.move_type == MoveType::Castle {

		match fenboard.from_piece.as_ref().unwrap().color {
			Color::White => {
				match fenboard.to64 {
					62 => {
						piece_placement[offset_slashes(square_to_index64("h1".into()))] = '.';
						piece_placement[offset_slashes(square_to_index64("f1".into()))] = 'R';
					},
					_ => {
						piece_placement[offset_slashes(square_to_index64("a1".into()))] = '.';
						piece_placement[offset_slashes(square_to_index64("d1".into()))] = 'R';
					},
				}
			},
			Color::Black => match fenboard.to64 {
					6 => {
						piece_placement[offset_slashes(square_to_index64("h8".into()))] = '.';
						piece_placement[offset_slashes(square_to_index64("f8".into()))] = 'r';
					},
					_ => {
						piece_placement[offset_slashes(square_to_index64("a8".into()))] = '.';
						piece_placement[offset_slashes(square_to_index64("d8".into()))] = 'r';
					},
				},
		}
	}
}

fn update_en_passant_piece_placement(fenboard: FenBoard, piece_placement: &mut Vec<char>) {

	if fenboard.en_passant_capture {
        let capture_offset = match fenboard.active_color {
            Color::White => 8,
            Color::Black => -8,
        };
        let captured_pawn = offset_slashes(square_to_index64(fenboard.en_passant.clone()) + capture_offset);

        piece_placement[captured_pawn] = '.';
    }
}

fn update_castle_rights(fenboard: &mut FenBoard) {

	let from_piece = fenboard.from_piece.clone().unwrap();
	let mut availability = fenboard.castling_availablity.clone();

	if availability != "-" {
		availability = match from_piece.piece_type {
			PieceType::King => {
				match from_piece.color {
					Color::White => availability.chars()
							.filter(|&x| x != 'K')
							.filter(|&x| x != 'Q')
							.collect(),
					Color::Black => availability.chars()
							.filter(|&x| x != 'k')
							.filter(|&x| x != 'q')
							.collect()
				}
			},
			PieceType::Rook => {
				match from_piece.color {
					Color::White => match fenboard.from_coords.as_str() {
						"a1" => availability.chars().filter(|&x| x != 'Q').collect(),
						 _ => availability.chars().filter(|&x| x != 'K').collect()
					}
					Color::Black => match fenboard.from_coords.as_str() {
						"a8" => availability.chars().filter(|&x| x != 'q').collect(),
						 _ => availability.chars().filter(|&x| x != 'k').collect()
					}
				}
			},
			_ => availability
		};

		if availability.is_empty() {
			availability = "-".into();
		}
	}

	fenboard.castling_availablity = availability;
}

fn update_ply(fenboard: &mut FenBoard) {
    match fenboard.active_color {
        Color::White => {
            fenboard.active_color = Color::Black;
        }
        Color::Black => {
            fenboard.full_move_number += 1;
            fenboard.active_color = Color::White;
        }
    }
}

fn update_en_passant(fenboard: &mut FenBoard) {

    let piece = fenboard.from_piece.as_ref().unwrap().clone();
    let from64 = fenboard.from64;
    let to64 = fenboard.to64;

	let mut en_passant = "-".to_string();

	fenboard.en_passant = match piece.piece_type.clone() {
		PieceType::Pawn => {
			let moved_two_squares = (from64 - to64).abs() == 16;

			if moved_two_squares {
				let rank = (((from64 % 8)) as u8 + b'a') as char;

				en_passant = match piece.color {
					Color::White => format!("{}3", rank),
					Color::Black => format!("{}6", rank),
				}
			}

			en_passant.into()
		},
		_ => en_passant.into()
	}

}

 fn update_move_type(fenboard: &mut FenBoard) {

   let from_piece = fenboard.from_piece.clone();
   let to_piece = fenboard.to_piece.clone();
   let current_move_type = fenboard.move_type.clone();

	fenboard.move_type = match (from_piece, to_piece) {
		(Some(f), _) if f.piece_type == PieceType::Pawn => current_move_type,
		(Some(f), Some(t)) if f.color != t.color => MoveType::Capture,
		_ => current_move_type
	};
}

 fn update_san_move(fenboard: &mut FenBoard) {

	let piece = fenboard.from_piece.as_ref().unwrap();
    let file = fenboard.from_coords.chars().nth(0).unwrap();

	let disambiguate = disambiguate_pieces(fenboard.clone(), piece.clone());

	let san: SharedString = match piece.piece_type {
		PieceType::Pawn if
			fenboard.move_type == MoveType::Capture ||
			fenboard.move_type == MoveType::CapturePromotion => file.to_shared_string(),
	    PieceType::Pawn => SharedString::new(),
	    _  => piece.as_fen().to_ascii_uppercase().to_shared_string() + &disambiguate,
	};

	let promotion_piece = &get_promotion_piece(piece.color.clone()).to_string();

	let to_square = match fenboard.move_type {
		MoveType::Normal => fenboard.to_coords.clone(),
		MoveType::Capture => "x".to_shared_string() + &fenboard.to_coords,
		MoveType::Castle => match &fenboard.to64 {
			2 | 58 => "O-O-O".to_shared_string(),
			6 | 62 => "O-O".to_shared_string(),
			_ => panic!("Illegal Castle")
		},
		MoveType::Promotion => fenboard.to_coords.to_shared_string() + "=" + &promotion_piece.to_uppercase(),
		MoveType::CapturePromotion => "x".to_shared_string() + &fenboard.to_coords + "=" + &promotion_piece.to_uppercase(),
		MoveType::Check => panic!("Check Move Type is not implemented here"),
		MoveType::Incorrect => panic!("Incorrect Move Type is not implemented here"),
		MoveType::EndOfLine => panic!("EndOfLine Move Type is not implemented here"),
	};

	let opposing_color = match &piece.color {
		Color::White => Color::Black,
		Color::Black => Color::White
	};

	let is_king_in_check = is_king_in_check(fenboard.piece_placement.clone(), &opposing_color);

	let is_check: SharedString = match is_king_in_check {
		true => "+".into(),
		false => "".into()
	};

	fenboard.san_move = match fenboard.move_type {
		MoveType::Castle => to_square + &is_check,
		_ => san + &to_square + &is_check
	};

	fenboard.move_type = match is_king_in_check {
		true => MoveType::Check,
		false => fenboard.move_type.clone()
	}
}

 fn get_promotion_piece(color: Color) -> char {
	match color {
		Color::White => 'Q',
		Color::Black => 'q',
	}
 }

 pub fn get_piece_from_fen(fen: SharedString, square: SharedString) -> Option<Piece> {

     let piece_code = get_piece_code_from_fen(fen, square_to_index64(square));

	 get_piece_from_fen_code(&piece_code.chars().next().unwrap())
}

 pub fn get_piece_code_from_fen(fen: SharedString, cell_index: i32) -> SharedString {

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

 pub fn get_piece_from_fen64(fen64: &Vec<char>, from64: i32) -> Option<Piece> {
     get_piece_from_fen_code(&fen64[from64 as usize])
}

 pub fn get_piece_from_fen_code(fen_code: &char) -> Option<Piece> {

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

pub fn get_king_square(fen64: SharedString, color: &Color) -> SharedString {

	let index = fen64
		.chars()
		.enumerate()
		.map(|(i, x)| (i, get_piece_from_fen_code(&x)))
		.filter(|(_i, x)| x.is_some())
		.map(|(i, x)| (i, x.unwrap()))
		.find(|(_i, x)| x.piece_type == PieceType::King && x.color == *color)
		.unwrap()
		.0 as i32;

	index64_to_square(index)
}

fn disambiguate_pieces(fenboard: FenBoard, piece: Piece) -> String {

	let pieces_to_qualify: Vec<i32> =
		get_confusers(fenboard.start_fen64.clone(), piece, fenboard.pinned_pieces)
		    .iter()
		    .map(|(i, p)| {
		        (
		            i.clone(),
		            p.get_moves(fenboard.start_fen64.clone(), *i)
		                .iter()
		                .filter(|x| index64_to_square(fenboard.to64) == x)
						.map(|x| x.clone())
						.nth(0)
		        )
		    })
			.filter(|x| x.1.is_some())
			.map(|(i, _)| i)
		    .collect();

	match pieces_to_qualify.iter().count() {
	    count if count > 1 => {

			let moved_piece = pieces_to_qualify
				.iter()
				.cloned()
				.find(|x| *x == fenboard.from64)
				.map(|x| index64_to_square(x))
				.unwrap();

			let other_pieces: Vec<SharedString> = pieces_to_qualify
				.iter()
				.cloned()
				.filter(|x| *x != fenboard.from64)
				.map(|x| index64_to_square(x))
				.collect();

			let moved_piece_file = moved_piece.chars().nth(0).unwrap();
			let moved_piece_rank = moved_piece.chars().nth(1).unwrap();

			let other_piece_file = other_pieces
				.iter()
				.nth(0)
				.unwrap()
				.chars()
				.nth(0)
				.unwrap();

			//only 2 pieces implemented
			match moved_piece_file == other_piece_file {
				true => moved_piece_rank.to_string(),
				false => moved_piece_file.to_string(),
			}
		},
		_ => String::new(),
	}
}

fn get_confusers(star_fen64: SharedString, piece: Piece, pinned_pieces: Vec<i32>) -> Vec<(i32, Piece)> {
	star_fen64
		.chars()
		.enumerate()
		.map(|(i, x)| (i, get_piece_from_fen_code(&x)))
		.filter(|(_, x)| x.is_some())
		.map(|(i, x)| (i as i32, x.unwrap()))
		.filter(|(_, x)| x.piece_type == piece.piece_type &&
						 x.piece_type != PieceType::Pawn &&
						 x.color == piece.color)
		.filter(|(i, _)| is_piece_pinned(pinned_pieces.clone(), *i) == false)
		.collect()
}

pub fn get_piece_color_from_square(fen: SharedString, square: SharedString) -> SharedString {

	if square == ""{
		return "".into();
	}

	let square64 = square_to_index64(square.clone());

    let piece_code = get_piece_code_from_fen(fen.into(), square64);

	 match piece_code.as_str() {
        "P" | "N" | "B" | "R" | "Q" | "K" => "w".into(),
        "p" | "n" | "b" | "r" | "q" | "k" => "b".into(),
        _ => SharedString::new(),
    }
}

pub fn get_color_from_square(fen: SharedString, square: SharedString) -> Color {
	match get_piece_color_from_square(fen, square).as_str() {
		"w" => Color::White,
		_ => Color::Black
	}
}

pub fn square_to_index64(square: SharedString) -> i32 {

	let chars: Vec<char> = square.to_lowercase().chars().collect();

	let file = (chars[0] as u8 - b'a') as i32;
	let rank = chars[1].to_digit(10).unwrap() as i32;

	((8 - rank) * 8) + file
}

pub fn index64_to_square(index: i32) -> SharedString {

    let file = index % 8;
    let rank = 8 - (index / 8);

    let file_char = (b'a' + file as u8) as char;

    format!("{}{}", file_char, rank).into()
}

fn offset_slashes(index: i32) -> usize {
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

pub fn fen_from_position(position: &Chess) -> SharedString {
    Fen::from_position(position, EnPassantMode::Always).to_shared_string()
 }
