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

	let mut move_result = piece.try_move_piece(fenboard.clone());

	if move_result.success == false ||
	   piece.color != fenboard.active_color ||
	   fenboard.from64 == fenboard.to64 ||
	   is_piece_pinned(&fenboard) ||
	   friendly_fire(&piece, &fenboard.to_piece) {

		move_result.success = false;
	}

	move_result.clone()
}

pub fn try_rook_move(fenboard: FenBoard) -> MoveResult {

	let legal_moves = get_rook_moves(fenboard.to_fen(), fenboard.from64);

	MoveResult { success: legal_moves.contains(&fenboard.to_coords), fenboard: fenboard }
}

pub fn get_rook_moves(fen: SharedString, from64: i32) -> Vec<SharedString> {

	let move_path = get_straight_moves(from64);

	get_influence_path(move_path, fen)
		.into_iter()
		.map(|x| index64_to_square(x))
		.collect()
}

fn get_straight_moves(from64: i32) -> MovePath {

	let (file, rank) = get_file_rank(from64);

	MovePath {
		west: (1..file).map(|f| from64 - f).collect(),
		east: (1..=8 - file).map(|f| from64 + f).collect(),
		north:  (1..=(8 - rank)).map(|r| from64 - (r * 8)).collect(),
		south:  (1..rank).map(|r| from64 + (r * 8)).collect(),
		..Default::default()
	}
}

pub fn try_bishop_move(fenboard: FenBoard) -> MoveResult {

	let legal_moves = get_bishop_moves(fenboard.to_fen(), fenboard.from64);

	MoveResult { success: legal_moves.contains(&fenboard.to_coords), fenboard: fenboard }
}

pub fn get_bishop_moves(fen: SharedString, from64: i32) -> Vec<SharedString> {

	let move_path = get_diagonal_moves(from64);

	get_influence_path(move_path, fen)
		.into_iter()
		.map(|x| index64_to_square(x))
		.collect()
}

fn get_diagonal_moves(from64: i32) -> MovePath {

	let (file, _) = get_file_rank(from64);

	let se: Vec<i32> = (1..=8 - file).map(|f| from64 + (f * 9)).collect();
	let sw : Vec<i32> = (1..file).map(|f| from64 + (f * 7)).collect();
	let ne : Vec<i32> = (1..=8 - file).map(|f| from64 - (f * 7)).collect();
	let nw : Vec<i32> = (1..file).map(|f| from64 - (f * 9)).collect();

	MovePath {
		se: se.iter().copied().filter(|x| *x >= 0 && *x <= 63).collect(),
		sw: sw.iter().copied().filter(|x| *x >= 0 && *x <= 63).collect(),
		ne: ne.iter().copied().filter(|x| *x >= 0 && *x <= 63).collect(),
		nw: nw.iter().copied().filter(|x| *x >= 0 && *x <= 63).collect(),
		..Default::default()
	}
}

pub fn try_queen_move(fenboard: FenBoard) -> MoveResult {

	let legal_moves = get_queen_moves(fenboard.to_fen(), fenboard.from64);

	MoveResult { success: legal_moves.contains(&fenboard.to_coords), fenboard: fenboard }
}

pub fn get_queen_moves(fen: SharedString, from64: i32) -> Vec<SharedString> {

	let mut legal_moves = get_rook_moves(fen.clone(), from64);

	legal_moves.extend(get_bishop_moves(fen, from64));

	legal_moves
}

pub fn try_king_move(fenboard: FenBoard) -> MoveResult {

	let fen = fenboard.to_fen();
	let legal_moves = get_king_moves(fen.clone(), fenboard.from64);

	let attacked_squares = get_enemy_attacked_squares(fen.clone(), fenboard.from64);

	let can_move = legal_moves.contains(&fenboard.to_coords) &&
				   attacked_squares.contains(&fenboard.to_coords) == false;

	MoveResult { success: can_move, fenboard: fenboard }
}

pub fn get_king_moves(fen: SharedString, from64: i32) -> Vec<SharedString> {

	let straight = get_straight_moves(from64);
	let diagonal = get_diagonal_moves(from64);

	let king_path = MovePath {
		north: straight.north.get(0).copied().into_iter().collect(),
		south: straight.south.get(0).copied().into_iter().collect(),
		east: straight.east.get(0).copied().into_iter().collect(),
		west: straight.west.get(0).copied().into_iter().collect(),
		ne: diagonal.ne.get(0).copied().into_iter().collect(),
		nw: diagonal.nw.get(0).copied().into_iter().collect(),
		se: diagonal.se.get(0).copied().into_iter().collect(),
		sw: diagonal.sw.get(0).copied().into_iter().collect(),
	};

	get_influence_path(king_path, fen)
		.into_iter()
		.map(|x| index64_to_square(x))
		.collect()
}

fn get_enemy_attacked_squares(fen: SharedString, from64: i32) -> Vec<SharedString> {

	let fen64: Vec<char> = to_fen64(fen.clone()).chars().collect();
	let our_color = get_color_from_square(fen.clone(), index64_to_square(from64));

	fen64
		.iter()
		.enumerate()
	    .map(|(i, x)| (i, get_piece_from_fen_code(&x)))
		.filter(|(_, x)| x.as_ref().is_some_and(|piece| piece.color != our_color))
		.flat_map(|(i, x)| x.unwrap().get_moves(fen.clone(), i as i32))
		.collect()
}

fn is_piece_pinned(fenboard: &FenBoard) -> bool {
	get_pinned_pieces(&fenboard).contains(&fenboard.from64)
}

fn get_pinned_pieces(fenboard: &FenBoard) -> Vec<i32> {

	let fen64: Vec<char> = fenboard.fen64.chars().collect();
	let our_color = fenboard.from_piece.clone().unwrap().color;

	let enemy_pinners: Vec<(i32, PieceType)> = fen64
		.clone()
		.into_iter()
		.enumerate()
	    .map(|(i, x)| (i, get_piece_from_fen_code(&x)))
		.filter(|(_, x)| x.as_ref().is_some_and(|piece| piece.color != our_color &&
				(piece.piece_type == PieceType::Bishop ||
				piece.piece_type == PieceType::Rook ||
				piece.piece_type == PieceType::Queen)))
		.map(|(i,x)| (i as i32, x.unwrap().piece_type))
		.collect();

	let search_paths : Vec<(i32, MovePath)> = enemy_pinners
		.clone()
		.iter()
		.map(|(i, piece)| match piece {
	        PieceType::Bishop => (*i, get_diagonal_moves(*i)),
			PieceType::Rook => (*i, get_straight_moves(*i)),
			PieceType::Queen => {

				let straight_moves = get_straight_moves(*i);
				let diagonal_moves = get_diagonal_moves(*i);

				(*i, MovePath {
					north: straight_moves.north,
					west: straight_moves.west,
					east: straight_moves.east,
					south: straight_moves.south,
					ne: diagonal_moves.ne,
					nw: diagonal_moves.nw,
					se: diagonal_moves.se,
					sw: diagonal_moves.sw,
				 })
			},

			_ => (*i, MovePath {..Default::default()}),
    })
	.collect();

	search_paths
		.iter()
		.map(|x| search_for_pinned_pieces(&fen64, x, &our_color, &fenboard.to64))
		.filter(|x| x.is_some())
		.map(|x| x.unwrap())
		.collect()
}

fn search_for_pinned_pieces(fen64: &Vec<char>, move_path: &(i32, MovePath), our_color: &Color, to64: &i32) -> Option<i32> {

	[(&move_path.0, &move_path.1.south),
	(&move_path.0, &move_path.1.north),
	(&move_path.0, &move_path.1.east),
	(&move_path.0, &move_path.1.west),
	(&move_path.0, &move_path.1.ne),
	(&move_path.0, &move_path.1.nw),
	(&move_path.0, &move_path.1.se),
	(&move_path.0, &move_path.1.sw)]
	.iter()
	.find_map(|direction| find_pinned_piece(fen64, direction, our_color, to64))
}

fn find_pinned_piece(fen64: &Vec<char>, move_path: &(&i32, &Vec<i32>), our_color: &Color, to64: &i32) -> Option<i32> {

	let pieces: Vec<(usize, i32, Option<Piece>)>  = move_path.1
		.iter()
		.enumerate()
		.map(|(i, i64)| (i, *i64, get_piece_from_fen64(&fen64, *i64)))
		.collect();

	let our_king = pieces
		.iter()
		.find(|piece| piece.2.as_ref().is_some_and(|p| p.piece_type == PieceType:: King && p.color == *our_color));

	let front_line_piece = pieces
		.iter()
		.find(|piece| piece.2.as_ref().is_some_and(|p| p.piece_type != PieceType::King));

	match (our_king, front_line_piece) {

		(Some(k), Some(fl)) if fl.2.as_ref().unwrap().color == *our_color => {

			let is_pinned = ((fl.0 + 1)..(k.0)).all(|x| square_is_empty(fen64, move_path.1[x]));
			let capturing_the_pinner = *move_path.0 == *to64;
			let moving_off_pinned_path = *&move_path.1.contains(*&to64) == false;

			match is_pinned {
				true if capturing_the_pinner == false && moving_off_pinned_path => {
					Some(fl.1)
				},
				_ => None
			}
		},
		(_, _) => None
	}
}

fn get_influence_path(move_path: MovePath, fen: SharedString) -> Vec<i32> {

	let north = path_to_first_piece(&move_path.north, &fen);
	let south = path_to_first_piece(&move_path.south, &fen);
	let east = path_to_first_piece(&move_path.east, &fen);
	let west = path_to_first_piece(&move_path.west, &fen);
	let ne = path_to_first_piece(&move_path.ne, &fen);
	let nw = path_to_first_piece(&move_path.nw, &fen);
	let se = path_to_first_piece(&move_path.se, &fen);
	let sw = path_to_first_piece(&move_path.sw, &fen);

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

fn path_to_first_piece(moves: &Vec<i32>, fen: &SharedString) -> Vec<i32> {

	let fen64: Vec<char> = to_fen64(fen.clone()).chars().collect();

	let first_contact_at = moves
		.iter()
		.take_while(|x| square_is_empty(&fen64, **x))
		.count();

	let piece_on_square_leeway = match moves.get(first_contact_at) {
	    Some(_) => 1,
	    None => 0,
	};

	moves.clone()
		.into_iter()
		.take(first_contact_at + piece_on_square_leeway)
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
	let moving_forward = forwards_movement(&pawn.color, fenboard.from64, fenboard.to64);

	let legal_move = moving_forward && (requested_move == 8 || (can_move_two && requested_move == 16));
	let pawn_path = get_pawn_path(fenboard.from64, fenboard.to64, moving_two, pawn.color.clone());
	let clear_path = pawn_path_is_clear(fenboard.fen64, pawn_path);

	clear_path && legal_move
}

fn is_pawn_capture(pawn: Piece, fenboard: FenBoard) -> (bool, bool) {

	let attempted_capture = is_attempting_pawn_capture(&pawn, fenboard.from64, fenboard.to64);

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

fn is_attempting_pawn_capture(pawn: &Piece, from64: i32, to64: i32) -> bool {

	let requested_move = (from64 - to64).abs();

	let a_pawn = from64 % 8 == 0;
	let h_pawn = (from64 + 1) % 8 == 0;
	let rook_pawn = a_pawn || h_pawn;

	(match rook_pawn {
		false => requested_move == 7 || requested_move == 9,
		true => match a_pawn {
			true => match pawn.color {
				Color::White => requested_move == 7,
				Color::Black => requested_move == 9,
			},
			false => match pawn.color {
				Color::White => requested_move == 9,
				Color::Black => requested_move == 7,
			},
		},
	}) && forwards_movement(&pawn.color, from64, to64)
}

pub fn get_pawn_attacked_squares(pawn: Piece, from64: i32) -> Vec<SharedString> {

	let attacked_squares = match pawn.color {
		Color::White => [(from64 - 7), (from64 - 9)],
		Color::Black => [(from64 + 7), (from64 + 9)],
	};

	attacked_squares
	.iter()
	.filter(|x| is_attempting_pawn_capture(&pawn, from64, **x))
	.map(|x| index64_to_square(*x))
	.collect()
}

fn is_promotion(to64: i32, color:Color) -> bool {
	match color {
		Color::White => to64 < 8,
		Color::Black => to64 > 55,
	}
}

fn forwards_movement(color: &Color, from64: i32, to64: i32) -> bool {
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

pub fn try_knight_move(fenboard: FenBoard) -> MoveResult {

	let moves: Vec<SharedString> = get_knight_moves(fenboard.from64);

	MoveResult { success: moves.contains(&fenboard.to_coords), fenboard: fenboard }
}

pub fn get_knight_moves(from64: i32) -> Vec<SharedString> {

	let octopus_moves : Vec<i32> = vec![6, 10, 17, 15]
		.iter()
		.flat_map(|&x| [x, -x])
		.collect();

	let (file, rank) = get_file_rank(from64);

	let moves: Vec<i32> = match(file, rank) {
		(_, _) if is_octopus_knight(file, rank) => octopus_moves,
		(1, 1) => [-6, -15].to_vec(),
		(8, 8) => [6, 15].to_vec(),
		(1, 8) => [10, 17].to_vec(),
		(8, 1) => [-10, -17].to_vec(),
		(1, 7) => [-6, 10, 17].to_vec(),
		(1, 2) => [-6, 10, -15,].to_vec(),
		(8, 2) => [-10, -17, 6].to_vec(),
		(8, 7) => [-10, 6, 15].to_vec(),
		(1, _) => [-6, 10, -15, 17].to_vec(),
		(8, _) => [6, -10, 15, -17].to_vec(),
		(2, 8) => [10, 15, 17].to_vec(),
		(7, 8) => [6, 15, 17].to_vec(),
		(2, 1) => [-6, -15, -17].to_vec(),
		(7, 1) => [-10, -15, -17].to_vec(),
		(2, 2) => [-15, -17, -6, 10].to_vec(),
		(7, 7) => [-10, 6, 15, 17].to_vec(),
		(2, 7) => [-6, 10, 15, 17].to_vec(),
		(7, 2) => [6, -10, -15, -17].to_vec(),
		(2, _) => [-17, -15, -6, 10, 15, 17].to_vec(),
		(7, _) => [-17, -15, -10, 6, 15, 17 ].to_vec(),
		(_, 7) => [-10, -6, 6, 10, 15, 17].to_vec(),
		(_, 2) => [-10, -15, -17, -6, 6, 10].to_vec(),
		(_, 1) => [-17, -15, -10, -6].to_vec(),
		(_, 8) => [17, 15, 10, 6].to_vec(),
		_ => [].to_vec()
	};

	moves.into_iter()
	.map(|x| index64_to_square(from64 + x))
	.collect()
}

fn is_octopus_knight(file: i32, rank: i32) -> bool {
	(file >= 3 && file <= 6) && (rank <= 6 && rank >= 3)
}

fn get_file_rank(from64: i32) -> (i32, i32) {

	let chars: Vec<char> = index64_to_square(from64).to_lowercase().chars().collect();

	let file = (chars[0] as u8 - b'a') as i32 + 1;
	let rank = chars[1].to_digit(10).unwrap() as i32;

	(file, rank)
}

fn friendly_fire(piece: &Piece, destination_piece: &Option<Piece>) -> bool {
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
	get_piece_from_fen_code(&fen64[square as usize]).is_none()
}