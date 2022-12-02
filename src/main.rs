use std::io;

use chess_board::moves::Move;
use chess_board::Board;

fn main() -> io::Result<()> {
    let mut chess = Board::default();

    loop {
        println!("Current Board:\n{:?}", chess);
        print!("Input move to make: ");
        let mut mv_string = String::new();
        io::stdin().read_line(&mut mv_string)?;
        let mv: Move = match mv_string.trim().parse() {
            Ok(m) => m,
            Err(e) => {
                println!("{e}");
                continue;
            }
        };
        match chess.make(mv) {
            Ok(_) => println!("Valid Move!"),
            Err(e) => println!("Illegal move: {}", e),
        }
    }
}
