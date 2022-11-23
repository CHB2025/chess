use std::env;

use chess_board::Board;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("2 arguments required: a fen and a depth")
    }
    let fen = &args[1];
    let depth: usize = args[2].parse().unwrap();
    let mut game = Board::from_fen(fen).unwrap();
    game.divided_perft(depth);
}
