use chb_chess::Board;

mod common;

#[ignore]
#[test]
fn test_perft() {
    for test in &common::perft_positions() {
        let mut b = Board::from_fen(&test.fen).unwrap();
        for (i, depth) in test.depth.iter().enumerate() {
            println!("Testing {} to depth {}", test.name, depth);
            assert_eq!(test.nodes[i], b.perft(*depth));
        }
    }
}
