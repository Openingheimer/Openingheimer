#![allow(dead_code)]

use crate::grand_master::*;
use crate::fen_master::*;
use slint::SharedString;

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
                Color::Black => "B"
            },
            self.castling_availablity,
            self.en_passant,
            self.half_move_clock,
            self.full_move_number
        ).into()
    }
}

#[derive(Clone)]
pub enum PieceType {
	Pawn,
	Bishop,
	Knight,
	Rook,
	Queen,
	King
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

pub trait PieceBrain {
    #[allow(dead_code)]
    fn as_value(&self) -> i32;
	fn as_fen(&self) -> char;
	fn is_legal_move(&self, fenboard: FenBoard) -> MoveResult;
}

impl PieceBrain for Piece {

	fn is_legal_move(&self, fenboard: FenBoard) -> MoveResult {

        match self.piece_type {
            PieceType::Pawn => try_pawn_move(self.clone(), fenboard.clone()),
            // PieceType::Bishop => 'b',
            // PieceType::Knight => 'n',
            // PieceType::Rook => 'r',
            // PieceType::Queen => 'q',
            // PieceType::King => 'k',
			_ => MoveResult {
                success: false,
                fenboard: fenboard,
            }
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

