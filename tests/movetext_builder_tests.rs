
#[cfg(test)]
mod move_text_builder_tests {
    use openingheimer::pgn_import::*;

    #[test]
    fn create_first_move() {
        let result = read_pgn("1. e4".into());

    }

    #[test]
    fn end_of_line_has_none_next() {
        let result = read_pgn("1. e4 e5".into());
    }

    #[test]
    fn sets_next() {
        let result = read_pgn("1. e4 e5".into());

    }

    #[test]
    fn sets_previous() {
        let result = read_pgn("1. e4 e5".into());

    }

    #[test]
    fn creates_moves() {
        let result = read_pgn("1. e4 e5 2. d4 d5 3. Nf3 Nf6".into());

    }

    #[test]
    fn handle_child_variation() {
        let result = read_pgn("1. e4 c5 (1... h5 2. h4) *".into());
        let c5 = &result[1];
        let h5 = &result[2];
        let h4 = &result[3];

    }

    #[test]
    fn handle_child_variation_continuations() {
        let result = read_pgn("1. e4 c5 (1... h5) 2. Nf3".into());
    }

    #[test]
    fn handle_multiple_variations_at_same_move() {
        let result = read_pgn("1. e4 c5 (1... h5 2. Nf3) (1... e5) 2. Nc3".into());
    }

    #[test]
    fn handle_nested_child_variations() {
        let result = read_pgn("1. e4 c5 (1... h5 2. h4 (2. d4 d5 (2... a5) 3. e5) 2... Nc6) 2. Na3 *".into());

    }
}