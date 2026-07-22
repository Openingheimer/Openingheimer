
use crate::SanMoveRow;
use crate::model::MoveReader;
use crate::model::MoveNode;
use crate::movetext::*;
use std::ops::ControlFlow;
use std::io;
use pgn_reader::shakmaty;
use pgn_reader::shakmaty::Position;
use shakmaty::Chess;
use pgn_reader::{Visitor, Reader, SanPlus, Skip, Nag };
use slint::ToSharedString;

pub fn read_pgn(pgn: String) -> Vec<MoveNode> {
    let reader = parse_pgn(pgn);

    reader.moves
}

pub fn read_as_move_text(pgn: String) -> Vec<SanMoveRow> {
    let reader = parse_pgn(pgn);

    reader.san_move_rows
}

fn parse_pgn(pgn: String) -> MoveReader {
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
            Some(cm) => {
                self.moves[cm].next = Some(self.moves.len());
                self.moves[cm].position.clone()
            }
            None => Chess::new(),
        };

        let next_move = san_plus.san.to_move(&position).unwrap();

        position.play_unchecked(next_move);

        let new_move = MoveNode {
            san: san_plus.san.to_string(),
            position: position.clone(),
            next: None,
            previous: self.current,
            variations: Vec::new(),
        };

        self.current = Some(self.moves.len());
        self.moves.push(new_move);

        update_rows(self, !position.turn());

        self.is_new_var = false;
        self.is_end_var = false;

        ControlFlow::Continue(())
    }

    fn begin_variation(&mut self, _movetext: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {

       let current = self.current.unwrap();
       let previous = self.moves[current].previous;
       let new_index = self.moves.len();

       self.moves[current].variations.push(new_index);

       self.pick_up_position_from.push(current);
       self.pick_up_turn_from.push(self.turn_number);

       self.current = previous;
       self.depth += 1;
       self.is_new_var = true;

       ControlFlow::Continue(Skip(false))
    }

    fn end_variation(&mut self, _movetext: &mut Self::Movetext) -> ControlFlow<Self::Output> {

       self.current = self.pick_up_position_from.pop();
       self.turn_number = self.pick_up_turn_from.pop().unwrap();
       self.depth -= 1;
       self.is_end_var = true;

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
            },
            shakmaty::Color::White => {
                move_row_node.white.san_text = last.san.to_shared_string();
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

fn nag_to_symbol(n: u8) -> &'static str {
    match n {
        1 => "!",
        2 => "?",
        3 => "!!",
        4 => "??",
        5 => "!?",
        6 => "?!",
        _ => "",
    }
}