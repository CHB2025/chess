use std::{
    collections::{hash_map::Entry, HashMap},
    env,
};

use chess_board::Board;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("2 arguments required: a fen and a depth")
    }
    let fen = &args[1];
    let depth: usize = args[2].parse().unwrap();
    let mut game = Board::from_fen(fen).unwrap();
    divided_perft(&mut game, depth);
}

pub fn divided_perft(board: &mut Board, depth: usize) {
    let mut tps: HashMap<u64, HashMap<usize, usize>> = HashMap::new();
    let mut total = 0;
    for m in board.pseudolegal_moves(board.is_white_to_move()) {
        if let Ok(_) = board.make(m) {
            let mc = perft_with_map(board, depth - 1, &mut tps);
            total += mc;
            board.unmake();
            println!("{m}: {mc}");
        }
    }
    println!();
    println!("Nodes Searched: {total}");
}

fn perft_with_map(
    board: &mut Board,
    depth: usize,
    tps: &mut HashMap<u64, HashMap<usize, usize>>,
) -> usize {
    if depth == 0 {
        return 1;
    }
    if let Entry::Occupied(e) = tps
        .entry(board.get_hash())
        .or_insert(HashMap::new())
        .entry(depth)
    {
        return *e.get();
    }

    let value = board
        .pseudolegal_moves(board.is_white_to_move())
        .into_iter()
        .filter_map(|m| {
            if let Ok(_) = board.make(m) {
                let t = Some(perft_with_map(board, depth - 1, tps));
                board.unmake();
                t
            } else {
                None
            }
        })
        .sum();
    *tps.entry(board.get_hash())
        .or_insert(HashMap::new())
        .entry(depth)
        .or_insert(value)
}
