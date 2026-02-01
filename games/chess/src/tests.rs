use crate::meeples::{Color, Meeple, Type};

#[cfg(test)]
mod pawn_tests {
    use super::*;

    fn create_empty_board() -> [[Option<Meeple>; 8]; 8] {
        [[None; 8]; 8]
    }

    #[test]
    fn test_white_pawn_forward_move() {
        let mut board = create_empty_board();
        let pawn = Meeple::new((4, 6), Type::Pawn, Color::White, 1.0);
        board[4][6] = Some(pawn);

        let moves = pawn.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(4, 5)), "White pawn should move one square forward");
        assert!(moves.contains(&(4, 4)), "White pawn should move two squares from starting position");
    }

    #[test]
    fn test_black_pawn_forward_move() {
        let mut board = create_empty_board();
        let pawn = Meeple::new((4, 1), Type::Pawn, Color::Black, 1.0);
        board[4][1] = Some(pawn);

        let moves = pawn.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(4, 2)), "Black pawn should move one square forward");
        assert!(moves.contains(&(4, 3)), "Black pawn should move two squares from starting position");
    }

    #[test]
    fn test_white_pawn_capture() {
        let mut board = create_empty_board();
        let white_pawn = Meeple::new((4, 5), Type::Pawn, Color::White, 1.0);
        let black_pawn = Meeple::new((3, 4), Type::Pawn, Color::Black, 1.0);
        board[4][5] = Some(white_pawn);
        board[3][4] = Some(black_pawn);

        let moves = white_pawn.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(3, 4)), "White pawn should be able to capture diagonally");
    }

    #[test]
    fn test_pawn_blocked_by_piece() {
        let mut board = create_empty_board();
        let white_pawn = Meeple::new((4, 5), Type::Pawn, Color::White, 1.0);
        let blocking_pawn = Meeple::new((4, 4), Type::Pawn, Color::White, 1.0);
        board[4][5] = Some(white_pawn);
        board[4][4] = Some(blocking_pawn);

        let moves = white_pawn.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(!moves.contains(&(4, 4)), "Pawn should not move into occupied square");
        assert!(!moves.contains(&(4, 3)), "Pawn should not be able to move two squares if blocked");
    }
}

#[cfg(test)]
mod knight_tests {
    use super::*;

    fn create_empty_board() -> [[Option<Meeple>; 8]; 8] {
        [[None; 8]; 8]
    }

    #[test]
    fn test_knight_movement_center() {
        let mut board = create_empty_board();
        let knight = Meeple::new((4, 4), Type::Knight, Color::White, 2.7);
        board[4][4] = Some(knight);

        let moves = knight.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert_eq!(moves.len(), 8, "Knight in center should have 8 possible moves");
        assert!(moves.contains(&(6, 5)), "Knight should move to (6, 5)");
        assert!(moves.contains(&(6, 3)), "Knight should move to (6, 3)");
        assert!(moves.contains(&(2, 5)), "Knight should move to (2, 5)");
        assert!(moves.contains(&(2, 3)), "Knight should move to (2, 3)");
    }

    #[test]
    fn test_knight_movement_edge() {
        let mut board = create_empty_board();
        let knight = Meeple::new((0, 0), Type::Knight, Color::White, 2.7);
        board[0][0] = Some(knight);

        let moves = knight.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert_eq!(moves.len(), 2, "Knight in corner should have 2 possible moves");
    }

    #[test]
    fn test_knight_cannot_capture_own_piece() {
        let mut board = create_empty_board();
        let knight = Meeple::new((4, 4), Type::Knight, Color::White, 2.7);
        let own_pawn = Meeple::new((6, 5), Type::Pawn, Color::White, 1.0);
        board[4][4] = Some(knight);
        board[6][5] = Some(own_pawn);

        let moves = knight.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(!moves.contains(&(6, 5)), "Knight should not capture own piece");
    }
}

#[cfg(test)]
mod bishop_tests {
    use super::*;

    fn create_empty_board() -> [[Option<Meeple>; 8]; 8] {
        [[None; 8]; 8]
    }

    #[test]
    fn test_bishop_diagonal_movement() {
        let mut board = create_empty_board();
        let bishop = Meeple::new((4, 4), Type::Bishop, Color::White, 3.0);
        board[4][4] = Some(bishop);

        let moves = bishop.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(5, 5)), "Bishop should move diagonally up-right");
        assert!(moves.contains(&(6, 6)), "Bishop should move diagonally up-right");
        assert!(moves.contains(&(7, 7)), "Bishop should move to edge");
        assert!(moves.contains(&(3, 3)), "Bishop should move diagonally down-left");
        assert!(moves.contains(&(3, 5)), "Bishop should move diagonally up-left");
    }

    #[test]
    fn test_bishop_blocked_by_piece() {
        let mut board = create_empty_board();
        let bishop = Meeple::new((4, 4), Type::Bishop, Color::White, 3.0);
        let blocker = Meeple::new((6, 6), Type::Pawn, Color::White, 1.0);
        board[4][4] = Some(bishop);
        board[6][6] = Some(blocker);

        let moves = bishop.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(5, 5)), "Bishop should move one square");
        assert!(!moves.contains(&(6, 6)), "Bishop should not capture own piece");
        assert!(!moves.contains(&(7, 7)), "Bishop should not move past blocker");
    }

    #[test]
    fn test_bishop_can_capture_opponent() {
        let mut board = create_empty_board();
        let bishop = Meeple::new((4, 4), Type::Bishop, Color::White, 3.0);
        let opponent = Meeple::new((6, 6), Type::Pawn, Color::Black, 1.0);
        board[4][4] = Some(bishop);
        board[6][6] = Some(opponent);

        let moves = bishop.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(6, 6)), "Bishop should capture opponent piece");
        assert!(!moves.contains(&(7, 7)), "Bishop should not move past captured piece");
    }
}

#[cfg(test)]
mod rook_tests {
    use super::*;

    fn create_empty_board() -> [[Option<Meeple>; 8]; 8] {
        [[None; 8]; 8]
    }

    #[test]
    fn test_rook_straight_movement() {
        let mut board = create_empty_board();
        let rook = Meeple::new((4, 4), Type::Rook, Color::White, 5.0);
        board[4][4] = Some(rook);

        let moves = rook.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        // Horizontal
        assert!(moves.contains(&(5, 4)), "Rook should move right");
        assert!(moves.contains(&(7, 4)), "Rook should move to edge");
        assert!(moves.contains(&(0, 4)), "Rook should move to left edge");
        // Vertical
        assert!(moves.contains(&(4, 0)), "Rook should move up");
        assert!(moves.contains(&(4, 7)), "Rook should move down");
    }

    #[test]
    fn test_rook_blocked_by_piece() {
        let mut board = create_empty_board();
        let rook = Meeple::new((4, 4), Type::Rook, Color::White, 5.0);
        let blocker = Meeple::new((4, 2), Type::Pawn, Color::White, 1.0);
        board[4][4] = Some(rook);
        board[4][2] = Some(blocker);

        let moves = rook.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert!(moves.contains(&(4, 3)), "Rook should move one square");
        assert!(!moves.contains(&(4, 2)), "Rook should not capture own piece");
        assert!(!moves.contains(&(4, 1)), "Rook should not move past blocker");
    }
}

#[cfg(test)]
mod queen_tests {
    use super::*;

    fn create_empty_board() -> [[Option<Meeple>; 8]; 8] {
        [[None; 8]; 8]
    }

    #[test]
    fn test_queen_combined_movement() {
        let mut board = create_empty_board();
        let queen = Meeple::new((4, 4), Type::Queen, Color::White, 9.0);
        board[4][4] = Some(queen);

        let moves = queen.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        // Should include rook moves
        assert!(moves.contains(&(5, 4)), "Queen should move like rook (horizontal)");
        assert!(moves.contains(&(4, 5)), "Queen should move like rook (vertical)");
        // Should include bishop moves
        assert!(moves.contains(&(5, 5)), "Queen should move like bishop (diagonal)");
        assert!(moves.contains(&(3, 3)), "Queen should move like bishop (diagonal)");
    }
}

#[cfg(test)]
mod king_tests {
    use super::*;

    fn create_empty_board() -> [[Option<Meeple>; 8]; 8] {
        [[None; 8]; 8]
    }

    #[test]
    fn test_king_one_square_movement() {
        let mut board = create_empty_board();
        let king = Meeple::new((4, 4), Type::King, Color::White, 0.0);
        board[4][4] = Some(king);

        let moves = king.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert_eq!(moves.len(), 8, "King should have 8 possible moves from center");
        assert!(moves.contains(&(3, 3)), "King should move diagonally");
        assert!(moves.contains(&(4, 3)), "King should move vertically");
        assert!(moves.contains(&(5, 4)), "King should move horizontally");
    }

    #[test]
    fn test_king_edge_movement() {
        let mut board = create_empty_board();
        let king = Meeple::new((0, 0), Type::King, Color::White, 0.0);
        board[0][0] = Some(king);

        let moves = king.show_moves(&board, &((42, 42), (42, 42)), &vec![]);
        assert_eq!(moves.len(), 3, "King in corner should have 3 possible moves");
    }

    #[test]
    fn test_king_cannot_move_into_attack() {
        let mut board = create_empty_board();
        let white_king = Meeple::new((4, 4), Type::King, Color::White, 0.0);
        let black_rook = Meeple::new((5, 4), Type::Rook, Color::Black, 5.0);
        board[4][4] = Some(white_king);
        board[5][4] = Some(black_rook);

        let moves = white_king.show_moves(&board, &((42, 42), (42, 42)), &vec![black_rook]);
        assert!(moves.contains(&(5, 4)), "King should be able to capture attacking piece");
    }
}
