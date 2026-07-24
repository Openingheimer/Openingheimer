use slint::SharedString;
use slint::ToSharedString;

use crate::model::*;
use crate::grand_master::*;
use crate::fen_master::*;
use crate::movetext::*;
use crate::SanMove;

 pub fn check_move(puzzle_master: &mut PuzzleMaster) -> (bool, SanMove) {

	let cloned = puzzle_master.clone();
	let moves = puzzle_master.move_reader.moves.clone();

	let result = try_make_move(cloned.fen, cloned.start_square, cloned.end_square);

	match puzzle_master.next{

		Some(m) => {
			let mainline_move = moves[m].clone();

			let mut opening_moves = moves
				    .iter()
				    .enumerate()
				    .filter(|(i, _)| mainline_move.variations.contains(i))
				    .map(|(i, x)| (i, x.clone()))
				    .collect::<Vec<_>>();

			opening_moves.push((m, mainline_move));

			let move_made = result.fenboard.san_move.to_string();

			let is_opening_move = opening_moves.iter().find(|x| x.1.san == move_made);

			match is_opening_move {
				Some(pm) if pm.1.next.is_none() &&
							pm.1.parent_line.is_some() => {

					let parent_line_idx = pm.1.parent_line.unwrap();

					let parent_line_move = moves[parent_line_idx].clone();
					let fen = fen_from_position(&parent_line_move.position);

					puzzle_master.next = parent_line_move.next;

					let san_move = create_san_move(parent_line_idx as i32, parent_line_move, fen);

					(true, san_move)
				}
				Some(pm) => {
					puzzle_master.next = pm.1.next;

					let san_move = create_san_move(pm.0 as i32, pm.1.clone(), result.fenboard.to_fen());

					(true, san_move)
				}
				None => (false, SanMove{..Default::default()})
			}
		}
		None => (false, SanMove{..Default::default()}),
	}
 }

 fn create_san_move(id: i32, move_node: MoveNode, fen: SharedString) -> SanMove{
	SanMove {
		san_text: move_node.san.to_shared_string(),
		fen: fen,
		id: id,
		next_id: match move_node.next {
			Some(ni) => ni as i32,
			None => -1
		},
		previous_id: match move_node.previous {
			Some(pi) => pi as i32,
			None => -1
		},
		variations: get_variations(&move_node.variations),
		parent_line: move_node.parent_line.map(|x| x as i32).unwrap_or(-1),
	}
 }