use chb_chess::Board;
use std::collections::HashMap;

mod common;

#[cfg(test)]
fn collision_detection(
    board: &mut Board,
    depth: usize,
    map: &mut HashMap<u64, HashMap<usize, usize>>,
) -> usize {
    let nodes = if depth == 1 {
        board.legal_moves().len()
    } else {
        let mvs = board.legal_moves();
        mvs.into_iter()
            .map(|m| {
                unsafe { board.make_unchecked(m) };
                let nodes = collision_detection(board, depth - 1, map);
                board.unmake();
                nodes
            })
            .sum()
    };

    let expected = map
        .entry(board.hash())
        .or_default()
        .entry(depth)
        .or_insert(nodes);

    assert_eq!(&nodes, expected, "Mismatch at depth {}", depth);

    nodes
}

#[ignore]
#[test]
fn collision_detection_test() {
    for test in &common::perft_positions() {
        let mut b = Board::from_fen(&test.fen).unwrap();
        for (depth, nodes) in test.depth.iter().zip(test.nodes.iter()) {
            let mut map: HashMap<u64, HashMap<usize, usize>> = HashMap::default();
            println!("Testing collision in {} to depth {}", test.name, depth);
            assert_eq!(nodes, &collision_detection(&mut b, *depth, &mut map))
        }
    }
}
