use haifisch_core::board::{Board, color::Color, piece::Piece};

#[must_use]
pub const fn piece_value_centipawns(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 100,
        Piece::Knight => 300,
        Piece::Bishop => 350,
        Piece::Rook => 500,
        Piece::Queen => 900,
        Piece::King => 0,
    }
}

/// Total material in centipawns for `(white, black)`.
#[must_use]
pub fn count_material(board: &Board) -> (i32, i32) {
    let mut white = 0i32;
    let mut black = 0i32;

    for piece in Piece::ALL {
        let value = piece_value_centipawns(piece);
        white += value
            * board
                .pieces_of(piece, Color::White)
                .popcount()
                .cast_signed();
        black += value
            * board
                .pieces_of(piece, Color::Black)
                .popcount()
                .cast_signed();
    }

    (white, black)
}

/// Material-only evaluation from the side-to-move's perspective.
#[must_use]
#[inline]
pub fn evaluate(board: &Board) -> i32 {
    let (white, black) = count_material(board);
    let mut score = 0;

    score += white - black;

    match board.side_to_move() {
        Color::White => score,
        Color::Black => -score,
    }
}
