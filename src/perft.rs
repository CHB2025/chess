use std::collections::hash_map::Entry;
use std::collections::HashMap;

use chess_board::Board;

pub fn divided_perft(board: &mut Board, depth: usize) {
    let mut tps: HashMap<u64, HashMap<usize, usize>> = HashMap::new();
    let total: usize = board
        .moves()
        .into_iter()
        .filter_map(|mv| {
            if board.make(mv).is_ok() {
                let t = perft_with_map(board, depth - 1, &mut tps);
                println!("{}: {}", mv, t);
                board.unmake();
                Some(t)
            } else {
                None
            }
        })
        .sum();
    println!("Nodes searched: {}", total);
}

pub fn perft(board: &mut Board, depth: usize) -> usize {
    let mut tps: HashMap<u64, HashMap<usize, usize>> = HashMap::new();
    perft_with_map(board, depth, &mut tps)
}

fn perft_with_map(
    board: &mut Board,
    depth: usize,
    tps: &mut HashMap<u64, HashMap<usize, usize>>,
) -> usize {
    if depth == 0 {
        return 1;
    }
    if let Entry::Occupied(e) = tps.entry(board.get_hash()).or_default().entry(depth) {
        return *e.get();
    }
    let value = if depth == 0 {
        board.moves().len()
    } else {
        board
            .moves()
            .into_iter()
            .filter_map(|m| {
                if board.make(m).is_ok() {
                    let t = Some(perft_with_map(board, depth - 1, tps));
                    board.unmake();
                    t
                } else {
                    None
                }
            })
            .sum()
    };
    *tps.entry(board.get_hash())
        .or_default()
        .entry(depth)
        .or_insert(value)
}
