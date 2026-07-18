
#[cfg(test)]
mod pgn_import_tests {
    use openingheimer::pgn_import::*;


    #[test]
    fn create_first_move() {
        let result = read_pgn("1. e4".into());

        assert_eq!(result.iter().count(), 1);
        assert_eq!(result[0].san, "e4");
        assert_eq!(result[0].previous, None);
    }

    #[test]
    fn end_of_line_has_none_next() {
        let result = read_pgn("1. e4 e5".into());

        assert_eq!(result[1].next, None);
    }

    #[test]
    fn sets_next() {
        let result = read_pgn("1. e4 e5".into());

        assert_eq!(result[0].next, Some(1));
    }

    #[test]
    fn sets_previous() {
        let result = read_pgn("1. e4 e5".into());

        assert_eq!(result[1].previous, Some(0));
    }

    #[test]
    fn creates_moves() {
        let result = read_pgn("1. e4 e5 2. d4 d5 3. Nf3 Nf6".into());

        assert_eq!(result.iter().count(), 6);
    }

    #[test]
    fn handle_child_variation() {
        let result = read_pgn("1. e4 c5 (1... h5 2. h4) *".into());
        let c5 = &result[1];
        let h5 = &result[2];
        let h4 = &result[3];

        assert_eq!(c5.next, None);
        assert_eq!(c5.variations.iter().count(), 1);
        assert_eq!(c5.variations[0], 2);

        assert_eq!(h5.san, "h5".to_string());
        assert_eq!(h5.previous, Some(0));
        assert_eq!(h5.next, Some(3));

        assert_eq!(h4.san, "h4".to_string());
        assert_eq!(h4.previous, Some(2));
        assert_eq!(h4.next, None);
    }

    #[test]
    fn handle_child_variation_continuations() {
        let result = read_pgn("1. e4 c5 (1... h5) 2. Nf3".into());
        let c5 = &result[1];
        let nf3 = &result[3];

        assert_eq!(c5.next, Some(3));
        assert_eq!(nf3.san, "Nf3");
        assert_eq!(nf3.previous, Some(1));
    }

    #[test]
    fn handle_multiple_variations_at_same_move() {
        let result = read_pgn("1. e4 c5 (1... h5 2. Nf3) (1... e5) 2. Nc3".into());
        let c5 = &result[1];
        let nc3 = &result[5];

        assert_eq!(c5.variations.iter().count(), 2);
        assert_eq!(c5.variations[0], 2);
        assert_eq!(c5.variations[1], 4);
        assert_eq!(c5.next, Some(5));

        assert_eq!(nc3.san, "Nc3".to_string());
        assert_eq!(nc3.previous, Some(1));
    }

    #[test]
    fn handle_nested_child_variations() {
        let result = read_pgn("1. e4 c5 (1... h5 2. h4 (2. d4 d5 (2... a5) 3. e5) 2... Nc6) 2. Na3 *".into());
        let c5 = &result[1];
        let h4 = &result[3];
        let d5 = &result[5];
        let a5 = &result[6];
        let na3 = &result[9];

        assert_eq!(c5.san, "c5".to_string());
        assert_eq!(c5.variations.iter().count(), 1);
        assert_eq!(c5.next, Some(9));
        assert_eq!(d5.next, Some(7));
        assert_eq!(h4.san, "h4".to_string());
        assert_eq!(h4.next, Some(8));
        assert_eq!(a5.next, None);
        assert_eq!(na3.previous, Some(1));
    }
}