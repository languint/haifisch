use haifisch_core::board::{Board, r#move::Move};
use haifisch_core::movegen::{MoveList, generate_legal, in_check};

use crate::eval;

pub const INF: i32 = 32_000;

pub const MATE_SCORE: i32 = 30_000;

pub const DEFAULT_DEPTH: u32 = 6;

/// Result of a root search.
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub nodes: u64,
    pub depth: u32,
}

#[must_use]
pub fn search(board: &mut Board, depth: u32) -> SearchResult {
    let mut nodes = 0u64;
    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);

    if moves.is_empty() {
        let score = if in_check(board, board.side_to_move()) {
            -MATE_SCORE
        } else {
            0
        };
        return SearchResult {
            best_move: None,
            score,
            nodes: 1,
            depth,
        };
    }

    if depth == 0 {
        return SearchResult {
            best_move: None,
            score: eval::evaluate(board),
            nodes: 1,
            depth: 0,
        };
    }

    let mut best_move = None;
    let mut best_score = -INF;
    let mut alpha = -INF;
    let beta = INF;

    for &m in moves.as_slice() {
        let undo = board.make_move(m);
        nodes += 1;
        let score = -negamax(board, depth - 1, 1, -beta, -alpha, &mut nodes);
        board.unmake_move(m, undo);

        if score > best_score {
            best_score = score;
            best_move = Some(m);
        }
        if score > alpha {
            alpha = score;
        }
    }

    SearchResult {
        best_move,
        score: best_score,
        nodes,
        depth,
    }
}

fn negamax(
    board: &mut Board,
    depth: u32,
    ply: u32,
    mut alpha: i32,
    beta: i32,
    nodes: &mut u64,
) -> i32 {
    if depth == 0 {
        *nodes += 1;
        return eval::evaluate(board);
    }

    let mut moves = MoveList::new();
    generate_legal(board, &mut moves);

    if moves.is_empty() {
        *nodes += 1;
        return if in_check(board, board.side_to_move()) {
            -MATE_SCORE + ply.cast_signed()
        } else {
            0
        };
    }

    let mut best = -INF;
    for &m in moves.as_slice() {
        let undo = board.make_move(m);
        *nodes += 1;
        let score = -negamax(board, depth - 1, ply + 1, -beta, -alpha, nodes);
        board.unmake_move(m, undo);

        if score > best {
            best = score;
        }
        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_startpos_returns_a_move() {
        let mut board = Board::startpos();
        let result = search(&mut board, 2);
        assert!(result.best_move.is_some());
        assert!(result.nodes > 0);
    }

    #[test]
    fn search_mate_position_has_no_move() {
        let Ok(mut board) =
            Board::from_fen("rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3")
        else {
            return;
        };
        let result = search(&mut board, 1);
        assert!(result.best_move.is_none());
        assert!(result.score <= -MATE_SCORE + 100);
    }
}
