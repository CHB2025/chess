use std::collections::{hash_map::Entry, HashMap};
use std::time::Instant;

use chess_board::Board;

fn divided_perft(board: &mut Board, depth: usize) {
    let mut tps: HashMap<u64, HashMap<usize, usize>> = HashMap::new();
    let total: usize = board.pseudolegal_moves(board.is_white_to_move()).into_iter().filter_map(|mv| {
        if board.make(mv).is_ok() {
            let t = perft_with_map(board, depth - 1, &mut tps);
            println!("{}: {}", mv, t);
            board.unmake();
            Some(t)
        } else {
            None
        }
    }).sum();
    println!("Nodes searched: {}", total);
}

fn perft(board: &mut Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }
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
    if let Entry::Occupied(e) = tps
        .entry(board.get_hash())
        .or_default()
        .entry(depth)
    {
        return *e.get();
    }
    let value = board
        .pseudolegal_moves(board.is_white_to_move())
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
        .sum();
    *tps.entry(board.get_hash())
        .or_default()
        .entry(depth)
        .or_insert(value)
}

struct PerftResult {
    name: String,
    fen: String,
    depth: Vec<usize>,
    nodes: Vec<usize>,
}

fn perft_positions() -> Vec<PerftResult> {
    vec![
        PerftResult {
            name: "Initial".to_owned(),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5],
            nodes: vec![20, 400, 8902, 4_865_609],
        },
        PerftResult {
            name: "Kiwipete".to_owned(),
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5],
            nodes: vec![48, 2039, 97_862, 193_690_690],
        },
        PerftResult {
            name: "Endgame".to_owned(),
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5],
            nodes: vec![14, 191, 2812, 67_4624],
        },
        PerftResult {
            name: "Middlegame".to_owned(),
            fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5],
            nodes: vec![6, 264, 9467, 15_833_292],
        },
        PerftResult {
            name: "Talkchess".to_owned(),
            fen: "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_owned(),
            depth: vec![1, 2, 3, 5],
            nodes: vec![44, 1486, 62_379, 89_941_194],
        },
        PerftResult {
            name: "Edwards 2".to_owned(),
            fen: "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
                .to_owned(),
            depth: vec![1, 2, 3, 5],
            nodes: vec![46, 2_079, 89_890, 164_075_551],
        },
    ]
}

#[test]
fn test_perft() {
    for test in &perft_positions() {
        let mut b = Board::from_fen(&test.fen).unwrap();
        for (i, depth) in test.depth.iter().enumerate() {
            println!("Testing {} to depth {}", test.name, depth);
            assert_eq!(test.nodes[i], perft(&mut b, *depth));
        }
    }
}

#[test]
fn test_kiwipete5() {
    let positions = perft_positions();
    let mut b = Board::from_fen(&positions[1].fen).unwrap();
    let now = Instant::now();
    perft(&mut b, 5);
    let elapsed = now.elapsed();
    println!("Running perft with depth 5 on kiwipete took {} seconds.", elapsed.as_secs());
}

//#[test]
//fn test_divided_perft() {
//    let fen = "rnbqk1nr/pppp1ppp/4p3/8/8/b2P4/PPPKPPPP/RNBQ1BNR w kq - 2 3";
//    let mut b = Board::from_fen(fen).unwrap();
//    divided_perft(&mut b, 1);
//}
