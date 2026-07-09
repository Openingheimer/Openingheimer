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

pub fn try_knight_move(fenboard: FenBoard) -> MoveResult {

	// let moves = get_knight_moves(fenboard.fen64.clone(), fenboard.from64.clone());
	// let requested_move = (fenboard.to64 - fenboard.from64).abs();

	MoveResult { success: true, fenboard: fenboard }
}

pub fn get_knight_moves(from64: i32) -> Vec<SharedString> {

	let legal_moves = vec![6, 10];

	let moves: Vec<SharedString> = legal_moves.into_iter()
		.flat_map(|m| [m + from64, from64 - m])
		.filter(|&m| m > 0)
		.map(|m| index64_to_square(m))
	    .collect();

	moves
}

pub fn try_rook_move(fenboard: FenBoard) -> MoveResult {

	let legal_moves = get_rook_moves(fenboard.to_fen(), fenboard.from64);

	MoveResult { success: legal_moves.contains(&fenboard.to_coords), fenboard: fenboard }
}

pub fn get_rook_moves(fen: SharedString, from64: i32) -> Vec<SharedString> {
	let color = get_color_from_square(fen.clone(), index64_to_square(from64));
	let move_path = get_straight_moves(from64);

	get_legal_move_path(move_path, fen, color)
		.into_iter()
		.map(|x| index64_to_square(x))
		.collect()
}

fn get_straight_moves(from64: i32) -> MovePath {

	let (file, rank) = get_file_rank(from64);

	MovePath {
		west: (1..=file).map(|f| from64 - f).collect(),
		east: (1..=7 - file).map(|f| from64 + f).collect(),
		north:  (1..=(8 - rank)).map(|r| from64 - (r * 8)).collect(),
		south:  (1..rank).map(|r| from64 + (r * 8)).collect(),
		..Default::default()
	}
}

fn get_legal_move_path(move_path: MovePath, fen: SharedString, color: Color) -> Vec<i32> {

	let north = get_legal_path(&move_path.north, &fen, &color);
	let south = get_legal_path(&move_path.south, &fen, &color);
	let east = get_legal_path(&move_path.east, &fen, &color);
	let west = get_legal_path(&move_path.west, &fen, &color);
	let ne = get_legal_path(&move_path.ne, &fen, &color);
	let nw = get_legal_path(&move_path.nw, &fen, &color);
	let se = get_legal_path(&move_path.se, &fen, &color);
	let sw = get_legal_path(&move_path.sw, &fen, &color);

	let mut moves = north;

	moves.extend(south);
	moves.extend(east);
	moves.extend(west);
	moves.extend(ne);
	moves.extend(nw);
	moves.extend(se);
	moves.extend(sw);

	moves
}

fn get_legal_path(moves: &Vec<i32>, fen: &SharedString, color: &Color) -> Vec<i32> {

	let fen64: Vec<char> = to_fen64(fen.clone()).chars().collect();

	let first_contact_at = moves
		.iter()
		.take_while(|x| square_is_empty(&fen64, **x))
		.count();

	let capture_leeway = match moves.get(first_contact_at) {
	    Some(x) => match get_piece_from_fen(fen.clone(), index64_to_square(*x)) {
			Some(p) if p.color != color.clone() => 1,
			_ => 0
		},
	    None => 0,
	};

	moves.clone()
		.into_iter()
		.take(first_contact_at + capture_leeway)
		.collect()
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
	let pawn_path = get_pawn_path(fenboard.from64, fenboard.to64, moving_two, pawn.color.clone());
	let clear_path = pawn_path_is_clear(fenboard.fen64, pawn_path);

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

fn get_file_rank(from64: i32) -> (i32, i32) {

	let chars: Vec<char> = index64_to_square(from64).to_lowercase().chars().collect();

	let file = (chars[0] as u8 - b'a') as i32;
	let rank = chars[1].to_digit(10).unwrap() as i32;

	(file, rank)
}

fn friendly_fire(piece: Piece, destination_piece: Option<Piece>) -> bool {

	if let Some(dp) = destination_piece {
		if piece.color == dp.color{
			return true;
		}
	}

	false
}

fn pawn_path_is_clear(fen64: SharedString, squares: Vec<i32>) -> bool {
	let fen_chars: Vec<char> = fen64.chars().collect();
	 squares
        .iter()
        .all(|&square| square_is_empty(&fen_chars, square))
}

fn square_is_empty(fen64: &Vec<char>, square: i32) -> bool {
	get_piece_from_fen_code(fen64[square as usize]).is_none()
}