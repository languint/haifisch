use std::io::Write;

use haifisch_core::board::{Board, r#move::Move, piece::Piece, square::Square};
use haifisch_core::movegen::{MoveList, generate_legal};

use crate::search::{self, DEFAULT_DEPTH, MATE_SCORE};

/// UCI engine state.
pub struct Engine {
    board: Board,
    moves: MoveList,
}

impl Engine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            board: Board::startpos(),
            moves: MoveList::new(),
        }
    }

    /// Handle one UCI command line. Returns `false` when the engine should quit.
    pub fn handle(&mut self, line: &str, out: &mut impl Write) -> bool {
        let line = line.trim();
        if line.is_empty() {
            return true;
        }

        let mut tokens = line.split_whitespace();
        let Some(cmd) = tokens.next() else {
            return true;
        };

        match cmd {
            "uci" => {
                writeln_ok(out, "id name Haifisch 0.1.0");
                writeln_ok(out, "id author languint");
                writeln_ok(out, "uciok");
            }
            "isready" => writeln_ok(out, "readyok"),
            "ucinewgame" => {
                self.board = Board::startpos();
            }
            "position" => self.cmd_position(tokens),
            "go" => self.cmd_go(tokens, out),
            "quit" => return false,
            _ => {}
        }

        true
    }

    fn cmd_position<'a>(&mut self, mut tokens: impl Iterator<Item = &'a str>) {
        let Some(kind) = tokens.next() else {
            return;
        };

        match kind {
            "startpos" => {
                self.board = Board::startpos();
                if tokens.next() == Some("moves") {
                    self.apply_moves(tokens);
                }
            }
            "fen" => {
                let mut fen_parts = Vec::new();
                for token in tokens.by_ref() {
                    if token == "moves" {
                        break;
                    }
                    fen_parts.push(token);
                }
                if fen_parts.is_empty() {
                    return;
                }
                let fen = fen_parts.join(" ");
                let Ok(board) = Board::from_fen(&fen) else {
                    return;
                };
                self.board = board;
                self.apply_moves(tokens);
            }
            _ => {}
        }
    }

    fn apply_moves<'a>(&mut self, tokens: impl Iterator<Item = &'a str>) {
        for token in tokens {
            let Some((from, to, promo)) = parse_uci_move(token) else {
                break;
            };
            generate_legal(&mut self.board, &mut self.moves);
            let Some(m) = find_legal(&self.moves, from, to, promo) else {
                break;
            };
            let _ = self.board.make_move(m);
        }
    }

    fn cmd_go<'a>(&mut self, tokens: impl Iterator<Item = &'a str>, out: &mut impl Write) {
        let depth = parse_go_depth(tokens).unwrap_or(DEFAULT_DEPTH);
        let result = search::search(&mut self.board, depth);

        let score_str = format_uci_score(result.score);
        writeln_ok(
            out,
            &format!(
                "info depth {} score {} nodes {}",
                result.depth, score_str, result.nodes
            ),
        );

        match result.best_move {
            Some(m) => writeln_ok(out, &format!("bestmove {}", format_move(m))),
            None => writeln_ok(out, "bestmove 0000"),
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_go_depth<'a>(mut tokens: impl Iterator<Item = &'a str>) -> Option<u32> {
    while let Some(token) = tokens.next() {
        if token == "depth" {
            let depth = tokens.next()?.parse().ok()?;
            return Some(depth);
        }
    }
    None
}

fn format_uci_score(score: i32) -> String {
    if score.abs() >= MATE_SCORE - 1000 {
        let ply = MATE_SCORE - score.abs();
        let mates_in = (ply + 1) / 2;
        if score > 0 {
            format!("mate {mates_in}")
        } else {
            format!("mate -{mates_in}")
        }
    } else {
        format!("cp {score}")
    }
}

fn writeln_ok(out: &mut impl Write, line: &str) {
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

fn parse_uci_move(input: &str) -> Option<(Square, Square, Option<Piece>)> {
    let bytes = input.as_bytes();
    if bytes.len() != 4 && bytes.len() != 5 {
        return None;
    }
    let from = parse_square(&input[..2])?;
    let to = parse_square(&input[2..4])?;
    let promo = if bytes.len() == 5 {
        Some(parse_promo(bytes[4])?)
    } else {
        None
    };
    Some((from, to, promo))
}

fn parse_square(s: &str) -> Option<Square> {
    let bytes = s.as_bytes();
    if bytes.len() != 2 {
        return None;
    }
    if !matches!(bytes[0], b'a'..=b'h') || !matches!(bytes[1], b'1'..=b'8') {
        return None;
    }
    Some(unsafe { Square::from_str_unchecked(s) })
}

const fn parse_promo(ch: u8) -> Option<Piece> {
    match ch.to_ascii_lowercase() {
        b'n' => Some(Piece::Knight),
        b'b' => Some(Piece::Bishop),
        b'r' => Some(Piece::Rook),
        b'q' => Some(Piece::Queen),
        _ => None,
    }
}

fn find_legal(moves: &MoveList, from: Square, to: Square, promo: Option<Piece>) -> Option<Move> {
    moves
        .as_slice()
        .iter()
        .copied()
        .find(|m| m.from() == from && m.to() == to && m.promotion() == promo)
}

fn format_move(m: Move) -> String {
    let mut s = String::with_capacity(5);
    s.push(m.from().file().to_char());
    s.push(m.from().rank().to_char());
    s.push(m.to().file().to_char());
    s.push(m.to().rank().to_char());
    if let Some(promo) = m.promotion() {
        s.push(match promo {
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen | Piece::Pawn | Piece::King => 'q',
        });
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uci_handshake() {
        let mut engine = Engine::new();
        let mut out = Vec::new();
        assert!(engine.handle("uci", &mut out));
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("uciok"));
        assert!(text.contains("id name Haifisch"));
    }

    #[test]
    fn position_and_go_emits_bestmove() {
        let mut engine = Engine::new();
        let mut out = Vec::new();
        assert!(engine.handle("ucinewgame", &mut out));
        assert!(engine.handle("position startpos moves e2e4", &mut out));
        out.clear();
        assert!(engine.handle("go depth 2", &mut out));
        let text = String::from_utf8_lossy(&out);
        assert!(text.contains("info depth 2"));
        assert!(text.contains("bestmove "));
    }

    #[test]
    fn quit_stops_loop() {
        let mut engine = Engine::new();
        let mut out = Vec::new();
        assert!(!engine.handle("quit", &mut out));
    }

    #[test]
    fn parse_depth_from_go() {
        assert_eq!(
            parse_go_depth("depth 5 movetime 1000".split_whitespace()),
            Some(5)
        );
        assert_eq!(parse_go_depth("movetime 1000".split_whitespace()), None);
    }
}
