use slint::ToSharedString;

use crate::model::*;
use crate::grand_master::*;
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
				Some(pm) => {
					puzzle_master.next = pm.1.next;

					match puzzle_master.next {
						None => println!("End of Var"),
						_ => ()
					};

					let san_move = SanMove {
						san_text: pm.1.san.to_shared_string(),
						fen: result.fenboard.to_fen(),
						id: pm.0 as i32,
						next_id: match pm.1.next {
							Some(ni) => ni as i32,
							None => -1
						},
						previous_id: match pm.1.previous {
							Some(pi) => pi as i32,
							None => -1
						},
						variations: get_variations(&pm.1.variations)
					};

					(true, san_move)
				}
				None => (false, SanMove{..Default::default()})
			}
		}
		None => (false, SanMove{..Default::default()}),
	}
 }