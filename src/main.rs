// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod grand_master;
mod fen_parser;

use std::error::Error;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::PhysicalSize;
use crate::fen_parser::*;
use crate::grand_master::*;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    let start_pos = String::from("rnb1kbnr/1pqppp1p/2p4P/1P6/3P2p1/p4N2/P1P1PPP1/RNBQKB1R b KQkq - 1 9");

	// let start_square: String = args.get(1).expect("").parse().expect("");
	// let end_square: String = args.get(2).expect("").parse().expect("");

	// let from64 = square_to_index64(start_square);
	// let to64 = square_to_index64(end_square);

	// let mut fen = parse_fen(start_pos);
	// let player_turn = match fen.active_color.as_str() {
	// 	"w" => Color::White,
	// 	_ => Color::Black
	// };

	// let move_result = try_making_move(fen.piece_placement.clone(), from64, to64, player_turn);

	// if move_result.sucess {
	// 	fen.piece_placement = move_result.piece_placement;
	// 	fen.en_passant = move_result.en_passant;

	// 	match fen.active_color.as_str() {
	// 	    "w" => fen.active_color = "b".to_string(),
	// 	    _ => {
	// 	        fen.full_move_number += 1;
	// 	        fen.active_color = "w".to_string();
	// 	    }
	// 	}
	// }
	// else{
	// 	println!("{}", move_result.reason);
	// }

	// println!("{}", fen.to_fen());

    set_screen_size(&ui)?;

     ui.global::<Callbacks>().on_piece_from_fen(|fen, index| {
        get_piece_code_from_fen(fen.to_string(), index).into()
    });

    ui.run()?;

    Ok(())
}

fn set_screen_size(ui: &AppWindow) -> Result<(), Box<dyn Error>> {
    let ui_weak = ui.as_weak();
    slint::invoke_from_event_loop(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let window = ui.window();
            window.with_winit_window(|winit_window| {

                if let Some(monitor) = winit_window.current_monitor() {
                    let size = monitor.size();
                    window.set_size(PhysicalSize::new(size.width, size.height));
                    window.set_maximized(true);
                    //window.set_fullscreen(true);
                }
            });
        }
    })?;

    return Ok(())
}
