#![allow(dead_code)]

use crate::grand_master::*;
use crate::fen_master::*;
use crate::SanMove;
use crate::SanMoveRow;
use shakmaty::Chess;
use slint::SharedString;
use std::sync::atomic::{AtomicU64, Ordering};

pub trait PieceBrain {
    #[allow(dead_code)]
    fn as_value(&self) -> i32;
	fn as_fen(&self) -> char;
	fn try_move_piece(&self, fenboard: FenBoard) -> MoveResult;
    fn get_moves(&self, fen: SharedString, from64: i32) -> Vec<SharedString>;
}

impl PieceBrain for Piece {

	fn try_move_piece(&self, fenboard: FenBoard) -> MoveResult {

        match self.piece_type {
            PieceType::Pawn => try_pawn_move(self.clone(), fenboard.clone()),
            PieceType::Bishop => try_bishop_move(fenboard.clone()),
            PieceType::Knight => try_knight_move(fenboard.clone()),
            PieceType::Rook => try_rook_move(fenboard.clone()),
            PieceType::Queen => try_queen_move(fenboard.clone()),
            PieceType::King => try_king_move(fenboard.clone())
        }
    }

    fn get_moves(&self, fen: SharedString, from64: i32) -> Vec<SharedString>{

        match self.piece_type {
            PieceType::Bishop => get_bishop_moves(fen, from64),
            PieceType::Knight => get_knight_moves(from64),
            PieceType::Rook => get_rook_moves(fen, from64),
            PieceType::Queen => get_queen_moves(fen, from64),
            PieceType::King => get_king_moves(fen, from64),
			PieceType::Pawn => get_pawn_attacked_squares(self.clone(), from64)
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

#[derive(Clone)]
pub struct FenBoard {
	pub piece_placement: SharedString,
    pub fen64: SharedString,
	pub active_color: Color,
	pub castling_availablity: SharedString,
	pub en_passant: SharedString,
	pub half_move_clock: i32,
	pub full_move_number: i32,
    pub from_piece: Option<Piece>,
    pub to_piece: Option<Piece>,
    pub from_fen71: usize,
    pub to_fen71: usize,
    pub from64: i32,
    pub to64: i32,
    pub en_passant_capture: bool,
    pub from_coords: SharedString,
    pub to_coords: SharedString,
    pub move_type: MoveType,
    pub san_move: SharedString,
}

impl FenBoard {
    pub fn to_fen(&self) -> SharedString {
        format!(
            "{} {} {} {} {} {}",
            unpad_empty_squares(self.piece_placement.clone()),
            match self.active_color.clone() {
                Color::White => "w",
                Color::Black => "b"
            },
            self.castling_availablity,
            self.en_passant,
            self.half_move_clock,
            self.full_move_number
        ).into()
    }
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum PieceType {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King
}

#[derive(Clone)]
#[derive(Default)]
pub struct MovePath {
	pub north: Vec<i32>,
	pub south: Vec<i32>,
	pub east: Vec<i32>,
	pub west: Vec<i32>,
	pub ne: Vec<i32>,
	pub nw: Vec<i32>,
	pub se: Vec<i32>,
	pub sw: Vec<i32>,
}

#[derive(Clone)]
#[derive(PartialEq)]
pub enum MoveType {
    Normal,
    Capture,
    Castle,
    Promotion,
    CapturePromotion
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Default)]
pub enum Color {
    #[default]
	White,
	Black
}

impl Color {
    pub fn as_str(&self) -> &'static str {
        match self {
            Color::White => "w",
            Color::Black => "b",
        }
    }
}

#[derive(Clone)]
pub struct MoveResult {
	pub success: bool,
    pub fenboard: FenBoard,
}

#[derive(Clone)]
pub struct Piece {
	pub piece_type: PieceType,
	pub color: Color,
}

#[derive(Clone)]
pub struct MoveRequest {
    pub moves: Vec<SanMoveRow>,
	pub fenboard: FenBoard,
    pub current_move: SanMove,
    pub last_move_in_variation: i32,
}

#[derive(Clone)]
pub struct MoveResponse {
    pub moves: Vec<SanMoveRow>,
    pub current_move: SanMove,
    pub last_move_in_variation: i32,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

#[derive(Clone)]
pub struct MoveReader {
    pub moves: Vec<MoveNode>,
    pub san_move_rows: Vec<SanMoveRow>,
    pub current: Option<usize>,
    pub pick_up_position_from: Vec<usize>,
    pub pick_up_turn_from: Vec<i32>,
    pub turn_number: i32,
    pub depth: i32,
    pub is_new_var: bool,
    pub is_end_var: bool,
}

#[derive(Clone)]
#[derive(Default)]
pub struct MoveNode {
    pub san: String,
    pub position: Chess,
    pub previous: Option<usize>,
    pub next: Option<usize>,
    pub variations: Vec<usize>,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
static NEXT_VAR_ID: AtomicU64 = AtomicU64::new(1);

pub fn next_id() -> i32 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed) as i32
}

pub fn next_variation_id() -> i32 {
    NEXT_VAR_ID.fetch_add(1, Ordering::Relaxed) as i32
}
