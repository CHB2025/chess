pub struct PerftResult {
    pub name: String,
    pub fen: String,
    pub depth: Vec<usize>,
    pub nodes: Vec<usize>,
}

pub fn perft_positions() -> Vec<PerftResult> {
    vec![
        PerftResult {
            name: "Initial".to_owned(),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5, 6, 7],
            nodes: vec![20, 400, 8902, 4_865_609, 119_060_324, 3_195_901_860],
        },
        PerftResult {
            name: "Kiwipete".to_owned(),
            fen: "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5, 6],
            nodes: vec![48, 2039, 97_862, 193_690_690, 8_031_647_685],
        },
        PerftResult {
            name: "Endgame".to_owned(),
            fen: "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5, 6, 7],
            nodes: vec![14, 191, 2812, 67_4624, 11_030_083, 178_633_661],
        },
        PerftResult {
            name: "Middlegame".to_owned(),
            fen: "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_owned(),
            depth: vec![1, 2, 3, 5, 6],
            nodes: vec![6, 264, 9467, 15_833_292, 706_045_033],
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
            depth: vec![1, 2, 3, 5, 6],
            nodes: vec![46, 2_079, 89_890, 164_075_551, 6_923_051_137],
        },
    ]
}
