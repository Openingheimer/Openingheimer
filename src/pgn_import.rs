
use crate::model::MoveNode;
use std::ops::ControlFlow;
use std::io;
use pgn_reader::shakmaty;
use pgn_reader::shakmaty::Position;
use shakmaty::Chess;
use pgn_reader::{Visitor, Reader, SanPlus, Skip };

pub struct PgnReader {
    pub moves: Vec<MoveNode>,
    pub current: Option<usize>,
    pub pick_up_position_from: Vec<usize>,
}

pub fn read_pgn(pgn: String) -> Vec<MoveNode> {

    let mut visitor = PgnReader {
        moves: [].to_vec(),
        current: None,
        pick_up_position_from: [].to_vec(),
    };

    let mut reader = Reader::new(io::Cursor::new(&pgn));

    let _ = reader.read_game(&mut visitor);

    visitor.moves
}

impl Visitor for PgnReader {
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
            position: position,
            next: None,
            previous: self.current,
            variations: Vec::new(),
        };

        self.current = Some(self.moves.len());
        self.moves.push(new_move);

        ControlFlow::Continue(())
    }

    fn begin_variation(&mut self, _movetext: &mut Self::Movetext) -> ControlFlow<Self::Output, Skip> {

       let current = self.current.unwrap();
       let previous = self.moves[current].previous;
       let new_index = self.moves.len();

       self.moves[current].variations.push(new_index);
       self.pick_up_position_from.push(current);
       self.current = previous;

       ControlFlow::Continue(Skip(false))
    }

    fn end_variation(&mut self, _movetext: &mut Self::Movetext) -> ControlFlow<Self::Output> {

       self.current = self.pick_up_position_from.pop();

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