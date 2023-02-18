use std::{io, env};

use chb_chess::{Board, BoardError, Move};

fn main() -> io::Result<()> {
    let mut chess = Board::default();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let cmd = &args[1];
        let arg = &args[2..].join(" ");
        handle_command(&mut chess, cmd, arg).unwrap();
        return Ok(());
    }

    loop {
        println!("Current Board:\n{:?}", chess);
        println!("Input command: ");
        let mut mv_string = String::new();
        io::stdin().read_line(&mut mv_string)?;
        mv_string = mv_string.trim().to_owned();
        if let Some((command, arg)) = mv_string.split_once(' ') {
            if let Err(e) = handle_command(&mut chess, command, arg) {
                println!("Error handling command: {}", e);
            }
        }
    }
}

fn handle_command(chess: &mut Board, cmd: &str, arg: &str) -> Result<(), BoardError> {
    match cmd.to_lowercase().as_str() {
        "make" => {
            let mv: Move = arg.trim().parse()?;
            chess.make(mv)?;
        }
        "unmake" => chess.unmake(),
        "perft" => {
            let depth: usize = arg.parse()?;
            chess.divided_perft(depth);
        }
        "fen" => {
            println!("Received Fen: \"{}\"", arg);
            *chess = Board::from_fen(arg)?;
        },
        "print" => println!("{}", chess),
        _ => println!("Unknown command. Ignoring"),
    };
    Ok(())
}
