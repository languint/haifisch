use crate::board::{
    Board,
    castling::CastlingRights,
    color::Color,
    file::File,
    piece::Piece,
    rank::Rank,
    square::Square,
};

/// FEN for the standard starting position.
pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Error produced while parsing a FEN string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FenError {
    InvalidFormat,
    InvalidPiece,
    InvalidSide,
    InvalidCastling,
    InvalidEnPassant,
    InvalidClock,
}

impl Board {
    /// Parse a FEN string into a [`Board`].
    ///
    /// # Errors
    /// Returns [`FenError`] when the FEN is malformed.
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let mut parts = fen.split_whitespace();
        let placement = parts.next().ok_or(FenError::InvalidFormat)?;
        let side = parts.next().ok_or(FenError::InvalidFormat)?;
        let castling = parts.next().ok_or(FenError::InvalidFormat)?;
        let ep = parts.next().ok_or(FenError::InvalidFormat)?;
        let halfmove = parts.next().unwrap_or("0");
        let fullmove = parts.next().unwrap_or("1");

        let mut board = Self::new();

        let mut rank = 7i8;
        let mut file = 0i8;
        for ch in placement.chars() {
            match ch {
                '/' => {
                    if file != 8 || rank <= 0 {
                        return Err(FenError::InvalidFormat);
                    }
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    file += (ch as u8 - b'0').cast_signed();
                    if file > 8 {
                        return Err(FenError::InvalidFormat);
                    }
                }
                _ => {
                    let (piece, color) = piece_from_char(ch).ok_or(FenError::InvalidPiece)?;
                    if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                        return Err(FenError::InvalidFormat);
                    }
                    let sq = Square::from_parts(
                        unsafe { File::from_u8_unchecked(file.cast_unsigned()) },
                        unsafe { Rank::from_u8_unchecked(rank.cast_unsigned()) },
                    );
                    board.set_piece(piece, color, sq);
                    file += 1;
                }
            }
        }
        if rank != 0 || file != 8 {
            return Err(FenError::InvalidFormat);
        }

        board.side_to_move = match side {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenError::InvalidSide),
        };

        board.castling = parse_castling(castling)?;
        board.ep_square = parse_ep(ep)?;
        board.halfmove_clock = halfmove.parse().map_err(|_| FenError::InvalidClock)?;
        board.fullmove_number = fullmove.parse().map_err(|_| FenError::InvalidClock)?;
        if board.fullmove_number == 0 {
            return Err(FenError::InvalidClock);
        }

        board.recompute_hash();
        Ok(board)
    }

    /// Serialize this position to a FEN string.
    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut out = String::with_capacity(64);
        for rank in (0..8).rev() {
            let mut empty = 0u8;
            for file in 0..8 {
                let sq = Square::from_parts(
                    unsafe { File::from_u8_unchecked(file) },
                    unsafe { Rank::from_u8_unchecked(rank) },
                );
                if let Some((piece, color)) = self.piece_at(sq) {
                    if empty > 0 {
                        out.push(char::from(b'0' + empty));
                        empty = 0;
                    }
                    out.push(piece_to_char(piece, color));
                } else {
                    empty += 1;
                }
            }
            if empty > 0 {
                out.push(char::from(b'0' + empty));
            }
            if rank > 0 {
                out.push('/');
            }
        }

        out.push(' ');
        out.push(self.side_to_move().to_char());
        out.push(' ');
        out.push_str(&castling_to_string(self.castling()));
        out.push(' ');
        match self.ep_square() {
            Some(sq) => {
                out.push(sq.file().to_char());
                out.push(sq.rank().to_char());
            }
            None => out.push('-'),
        }
        out.push(' ');
        out.push_str(&self.halfmove_clock().to_string());
        out.push(' ');
        out.push_str(&self.fullmove_number().to_string());
        out
    }
}

const fn piece_from_char(ch: char) -> Option<(Piece, Color)> {
    let (piece, color) = match ch {
        'P' => (Piece::Pawn, Color::White),
        'N' => (Piece::Knight, Color::White),
        'B' => (Piece::Bishop, Color::White),
        'R' => (Piece::Rook, Color::White),
        'Q' => (Piece::Queen, Color::White),
        'K' => (Piece::King, Color::White),
        'p' => (Piece::Pawn, Color::Black),
        'n' => (Piece::Knight, Color::Black),
        'b' => (Piece::Bishop, Color::Black),
        'r' => (Piece::Rook, Color::Black),
        'q' => (Piece::Queen, Color::Black),
        'k' => (Piece::King, Color::Black),
        _ => return None,
    };
    Some((piece, color))
}

const fn piece_to_char(piece: Piece, color: Color) -> char {
    let ch = match piece {
        Piece::Pawn => 'p',
        Piece::Knight => 'n',
        Piece::Bishop => 'b',
        Piece::Rook => 'r',
        Piece::Queen => 'q',
        Piece::King => 'k',
    };
    match color {
        Color::White => match ch {
            'p' => 'P',
            'n' => 'N',
            'b' => 'B',
            'r' => 'R',
            'q' => 'Q',
            'k' => 'K',
            _ => ch,
        },
        Color::Black => ch,
    }
}

fn parse_castling(s: &str) -> Result<CastlingRights, FenError> {
    if s == "-" {
        return Ok(CastlingRights::NONE);
    }
    let mut rights = CastlingRights::NONE;
    for ch in s.chars() {
        let add = match ch {
            'K' => CastlingRights::WHITE_KING,
            'Q' => CastlingRights::WHITE_QUEEN,
            'k' => CastlingRights::BLACK_KING,
            'q' => CastlingRights::BLACK_QUEEN,
            _ => return Err(FenError::InvalidCastling),
        };
        rights.insert(add);
    }
    Ok(rights)
}

fn castling_to_string(rights: CastlingRights) -> String {
    if rights.is_empty() {
        return "-".to_owned();
    }
    let mut s = String::with_capacity(4);
    if rights.contains(CastlingRights::WHITE_KING) {
        s.push('K');
    }
    if rights.contains(CastlingRights::WHITE_QUEEN) {
        s.push('Q');
    }
    if rights.contains(CastlingRights::BLACK_KING) {
        s.push('k');
    }
    if rights.contains(CastlingRights::BLACK_QUEEN) {
        s.push('q');
    }
    s
}

fn parse_ep(s: &str) -> Result<Option<Square>, FenError> {
    if s == "-" {
        return Ok(None);
    }
    if s.len() != 2 {
        return Err(FenError::InvalidEnPassant);
    }
    let bytes = s.as_bytes();
    if !matches!(bytes[0], b'a'..=b'h') || !matches!(bytes[1], b'3' | b'6') {
        return Err(FenError::InvalidEnPassant);
    }
    Ok(Some(unsafe { Square::from_str_unchecked(s) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_round_trip() {
        let board = Board::from_fen(STARTPOS_FEN);
        assert!(board.is_ok());
        if let Ok(board) = board {
            assert_eq!(board.to_fen(), STARTPOS_FEN);
            assert_eq!(board.side_to_move(), Color::White);
            assert_eq!(board.castling(), CastlingRights::ALL);
            assert_eq!(board.ep_square(), None);
        }
    }

    #[test]
    fn kiwipete_round_trip() {
        const FEN: &str =
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let board = Board::from_fen(FEN);
        assert!(board.is_ok());
        if let Ok(board) = board {
            assert_eq!(board.to_fen(), FEN);
        }
    }
}
