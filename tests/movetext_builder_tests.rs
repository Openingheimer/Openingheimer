
#[cfg(test)]
mod move_text_builder_tests {
    use openingheimer::pgn_import::*;
    use slint::Model;

    #[test]
    fn create_first_ply() {
        let result = read_as_move_text("1. e4".into());

        assert_eq!(result.iter().count(), 1);

        let move_row = &result[0];

        assert_eq!(move_row.white.id, 0);
        assert_eq!(move_row.white.san_text, "e4");
        assert_eq!(move_row.white.fen, "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string());
        assert_eq!(move_row.white.previous_id, -1);
        assert_eq!(move_row.white.next_id, -1);
        assert_eq!(move_row.white.variations.iter().count(), 0);
    }

    #[test]
    fn create_first_move() {
        let result = read_as_move_text("1. e4 e5".into());

        assert_eq!(result.iter().count(), 1);
        assert_eq!(result[0].white.san_text, "e4");
        assert_eq!(result[0].white.id, 0);
        assert_eq!(result[0].white.previous_id, -1);
        assert_eq!(result[0].white.next_id, 1);
        assert_eq!(result[0].black.san_text, "e5");
        assert_eq!(result[0].black.id, 1);
        assert_eq!(result[0].black.previous_id, 0);
        assert_eq!(result[0].black.next_id, -1);
        assert_eq!(result[0].black.fen, "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2".to_string());
    }

    #[test]
    fn create_second_move() {
        let result = read_as_move_text("1. e4 e5 2. Nf3 Nf6".into());

        assert_eq!(result.iter().count(), 2);
        assert_eq!(result[0].black.san_text, "e5");
        assert_eq!(result[0].black.previous_id, 0);
        assert_eq!(result[0].black.next_id, 2);
        assert_eq!(result[1].white.san_text, "Nf3");
        assert_eq!(result[1].white.previous_id, 1);
        assert_eq!(result[1].white.id, 2);
        assert_eq!(result[1].white.previous_id, 1);
        assert_eq!(result[1].white.next_id, 3);
        assert_eq!(result[1].black.san_text, "Nf6");
        assert_eq!(result[1].black.id, 3);
        assert_eq!(result[1].black.previous_id, 2);
        assert_eq!(result[1].black.next_id, -1);
        assert_eq!(result[0].turn, 1);
        assert_eq!(result[1].turn, 2);
        assert_eq!(result[1].depth, 1);
    }

    #[test]
    fn handle_child_variation_black() {
        let result = read_as_move_text("1. e4 c5 (1... h5 2. h4 d5) *".into());

        assert_eq!(result.iter().count(), 3);
        assert_eq!(result[1].black.san_text, "h5");
        assert_eq!(result[1].white.san_text, "..");
        assert_eq!(result[1].white.next_id, -1);
        assert_eq!(result[1].turn, 1);
        assert_eq!(result[1].turn, 1);
        assert_eq!(result[1].depth, 2);
        assert_eq!(result[2].depth, 2);
        assert_eq!(result[2].turn, 2);
        assert_eq!(result[0].black.san_text, "c5");
        assert_eq!(result[0].black.next_id, -1);
        assert_eq!(result[0].black.variations.iter().count(), 1);
        assert_eq!(result[1].black.previous_id, 0);
    }

    #[test]
    fn handle_child_variation_white() {
        let result = read_as_move_text("1. e4 e5 2. Nf3 (2. d4) *".into());

        assert_eq!(result.iter().count(), 3);
        assert_eq!(result[1].white.san_text, "Nf3");
        assert_eq!(result[1].white.variations.iter().count(), 1);
        assert_eq!(result[1].white.next_id, -1);

        assert_eq!(result[2].white.san_text, "d4");
        assert_eq!(result[2].white.previous_id, 1);
        assert_eq!(result[2].turn, 2);
        assert_eq!(result[2].depth, 2);
    }


    #[test]
    fn handle_child_variation_continuations_black_var_start() {
        let result = read_as_move_text("1. e4 c5 (1... h5 2. h4 g5 3. g4) 2. Nf3 *".into());

        assert_eq!(result.iter().count(), 5);
        assert_eq!(result[4].white.san_text, "Nf3");
        assert_eq!(result[4].turn, 2);
        assert_eq!(result[4].white.previous_id, 1);
        assert_eq!(result[0].black.next_id, 6);
    }

    #[test]
    fn handle_child_variation_continuations_white_var_start() {
        let result = read_as_move_text("1. e4 c5 2. Nf3 (2. Bc4 Nf6 3. d4 cxd4) 2... Nc6 *".into());

        assert_eq!(result.iter().count(), 5);
        assert_eq!(result[4].black.san_text, "Nc6");
        assert_eq!(result[4].black.previous_id, 2);
        assert_eq!(result[4].white.san_text, "..");
        assert_eq!(result[1].white.san_text, "Nf3");
        assert_eq!(result[1].white.next_id, 7);
        assert_eq!(result[1].white.variations.iter().count(), 1);
    }

    #[test]
    fn handle_multiple_variations_at_same_move_black_var_start() {
        let result = read_as_move_text("1. e4 c5 (1... h5 2. h4 g5) (1... d5 2. exd5) 2. Bc4 *".into());
        let c5 = result[0].black.clone();

        assert_eq!(result.iter().count(), 6);
        assert_eq!(c5.san_text, "c5");
        assert_eq!(c5.variations.row_count(), 2);
        assert_eq!(c5.variations.row_data(0).unwrap(), 2);
        assert_eq!(c5.variations.row_data(1).unwrap(), 5);
        assert_eq!(c5.next_id, 7);
    }

    #[test]
    fn handle_multiple_variations_at_same_move_white_var_start() {
        let result = read_as_move_text("1. e4 c5 2. Bc4 (2. Nf3 d5) (2. Bb5 Nc6 3. f4) 2... h5 *".into());

        let bc4 = result[1].white.clone();

        assert_eq!(result.iter().count(), 6);
        assert_eq!(bc4.san_text, "Bc4");
        assert_eq!(bc4.variations.row_count(), 2);
        assert_eq!(bc4.variations.row_data(0).unwrap(), 3);
        assert_eq!(bc4.variations.row_data(1).unwrap(), 5);
        assert_eq!(bc4.next_id, 8);
    }

      #[test]
    fn handle_nested_child_variations() {
        let result = read_as_move_text("1. e4 c5 (1... h5 2. h4 (2. d4 d5 (2... a5) 3. e5) 2... Nc6) 2. Na3 *".into());
        let c5 = &result[0].black;
        let h4 = &result[2].white;
        let d4 = &result[3].white;
        let d5 = &result[3].black;
        let a5 = &result[4].black;
        let na3 = &result[7].white;

        assert_eq!(result.iter().count(), 8);
        assert_eq!(c5.san_text, "c5".to_string());
        assert_eq!(c5.variations.iter().count(), 1);
        assert_eq!(c5.next_id, 9);
        assert_eq!(h4.san_text, "h4".to_string());
        assert_eq!(h4.next_id, 8);
        assert_eq!(h4.variations.iter().count(), 1);
        assert_eq!(d4.previous_id, 2);
        assert_eq!(d5.san_text, "d5");
        assert_eq!(d5.variations.iter().count(), 1);
        assert_eq!(d5.next_id, 7);
        assert_eq!(a5.san_text, "a5");
        assert_eq!(a5.next_id, -1);
        assert_eq!(na3.san_text, "Na3");
        assert_eq!(na3.previous_id, 1);
    }
}