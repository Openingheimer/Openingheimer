#![allow(unused_variables)]

use crate::model::MoveReader;
use crate::model::MoveNode;
use crate::model::MoveType;
use crate::model::next_variation_id;
use crate::movetext::*;
use std::ops::ControlFlow;
use std::io;
use pgn_reader::shakmaty;
use pgn_reader::shakmaty::Position;
use shakmaty::Chess;
use pgn_reader::{Visitor, Reader, SanPlus, Skip, Nag };
use slint::ToSharedString;

pub fn parse_pgn(pgn: String) -> MoveReader {
    let mut visitor = MoveReader {
        moves: [].to_vec(),
        current: None,
        pick_up_position_from: [].to_vec(),
        pick_up_turn_from: [].to_vec(),
        san_move_rows: [].to_vec(),
        turn_number: 0,
        depth: 1,
        is_new_var: false,
        is_end_var: false,
        parent_lines: [].to_vec(),
        variation_ids: [next_variation_id()].to_vec(),
    };

    let mut reader = Reader::new(io::Cursor::new(&pgn));

    let _ = reader.read_game(&mut visitor);

    visitor
}

impl Visitor for MoveReader {
    type Tags = ();
    type Movetext = usize;
    type Output = usize;

    fn san(&mut self, _movetext: &mut Self::Movetext, san_plus: SanPlus) -> ControlFlow<Self::Output> {

        let mut position = match self.current {
            Some(cm) if self.is_new_var == false => {
                self.moves[cm].next = Some(self.moves.len());
                self.moves[cm].position.clone()
            }
            Some(cm) => {
                self.moves[cm].position.clone()
            },
            _ => Chess::new(),
        };

        let next_move = san_plus.san.to_move(&position).unwrap();

        position.play_unchecked(next_move);

        let suffix = match san_plus.suffix {
            Some(x) => x.to_string(),
            _ => "".to_string()
        };

        let new_move = MoveNode {
            san: san_plus.san.to_string() + &suffix.to_string(),
            position: position.clone(),
            next: None,
            previous: self.current,
            variations: Vec::new(),
            parent_line: self.parent_lines.iter().last().copied(),
            variation_id: self.variation_ids.iter().last().copied().unwrap(),
            from_square: next_move.from().unwrap().to_string(),
            to_square: next_move.to().to_string(),
            move_type: as_move_type(san_plus, suffix),
        };

        self.current = Some(self.moves.len());
        self.moves.push(new_move);

        to_san_move_rows(self, !position.turn());

        self.is_new_var = false;
        self.is_end_var = false;

        ControlFlow::Continue(())
    }

    fn begin_variation(&mut self, _movetext: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {

       let current = self.current.unwrap();
       let previous = self.moves[current].previous;
       let new_index = self.moves.len();

       self.moves[current].variations.push(new_index);

       self.parent_lines.push(previous.unwrap());
       self.pick_up_position_from.push(current);
       self.pick_up_turn_from.push(self.turn_number);
       self.variation_ids.push(next_variation_id());

       self.current = previous;
       self.depth += 1;
       self.is_new_var = true;

       ControlFlow::Continue(Skip(false))
    }

    fn end_variation(&mut self, _movetext: &mut Self::Movetext) -> ControlFlow<Self::Output> {

       self.current = self.pick_up_position_from.pop();
       self.turn_number = self.pick_up_turn_from.pop().unwrap();
       self.variation_ids.pop();
       self.depth -= 1;
       self.is_end_var = true;
       self.parent_lines.pop();

       ControlFlow::Continue(())
    }

    fn nag( &mut self, _movetext: &mut Self::Movetext, nag: Nag) -> ControlFlow<Self::Output> {

       let last = self.moves.last_mut().unwrap();

       last.san.push_str(&format!(" {}", nag_to_symbol(nag.0)));

       let move_row_node = self.san_move_rows
            .iter_mut()
            .find(|x| x.black.id == self.current.unwrap() as i32 ||
                      x.white.id == self.current.unwrap() as i32)
            .unwrap();

        match !last.position.turn() {
            shakmaty::Color::Black => {
                move_row_node.black.san_text = last.san.to_shared_string();
                move_row_node.blunder = move_row_node.blunder || last.san.contains("??");
            },
            shakmaty::Color::White => {
                move_row_node.white.san_text = last.san.to_shared_string();
                move_row_node.blunder = move_row_node.blunder || last.san.contains("??");
            }
        }

       ControlFlow::Continue(())
    }

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, _tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(0)
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        movetext
    }
}

fn as_move_type(san_plus: SanPlus, suffix: String) -> MoveType {
    match san_plus.san {

        _ if suffix.contains("+") => MoveType::Check,

        shakmaty::san::San::Normal { role, file, rank, capture, to, promotion } => {
            if promotion.is_some(){
                return MoveType::Promotion;
            }
            if capture {
                return MoveType::Capture;
            }

            MoveType::Normal
        },

        shakmaty::san::San::Castle(_) => MoveType::Castle,
        _ => MoveType::Normal
    }
}

fn nag_to_symbol(n: u8) -> &'static str {
    match n {
        1 => "!",
        2 => "?",
        3 => "!!",
        4 => "??",
        5 => "!?",
        6 => "?!",
        146 => "N",
        _ => "",
    }
}