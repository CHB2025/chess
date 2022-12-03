use std::str::FromStr;

use crate::error::{BoardError, ErrorKind};
use crate::position::HybridPosition;
use crate::{piece, piece::Piece, square::Square, Board};
use regex::Regex;

impl Board {
    pub fn to_fen(&self) -> String {
        let mut output = String::new();

        output += self.position.to_string().as_str();

        output += &format!(" {}", if self.white_to_move { "w" } else { "b" });

        if self.castle.iter().all(|x| !x) {
            output += " -";
        } else {
            output.push(' ');
            for (i, &can_castle) in self.castle.iter().enumerate() {
                let side = Piece::try_from(((i & 1) | ((i >> 1) * piece::BLACK)) as u8).unwrap();
                if can_castle {
                    output += &format!("{}", side);
                }
            }
        }

        if let Some(ept) = self.ep_target {
            output += &format!(" {}", ept)
        } else {
            output += " -"
        }

        output += &format!(" {}", self.halfmove);
        output += &format!(" {}", self.fullmove);

        return output;
    }

    pub fn from_fen(fen: impl Into<String>) -> Result<Self, BoardError> {
        create_board(fen)
    }
}

pub fn create_board<S: Into<String>>(fen: S) -> Result<Board, BoardError> {
    let f: String = fen.into();
    if !is_valid(&f) {
        return Err(BoardError::new(
            ErrorKind::InvalidInput,
            "Invalid FEN input.",
        ));
    }

    // Should never be used since FEN is checked
    let short_err = || BoardError::new(ErrorKind::InvalidInput, "Missing sections of FEN");

    let mut board = Board {
        position: HybridPosition::empty(),
        white_to_move: true,
        castle: [false; 4],
        ep_target: None,
        halfmove: 0,
        fullmove: 0,
        move_history: vec![],
        hash: 0,
        hash_keys: [0u64; 781],
    };

    let mut sections = f.split(' ');

    let b = sections.next().ok_or_else(short_err)?;
    for (y, row) in b.split('/').enumerate() {
        let mut offset: usize = 0;
        for (x, symbol) in row.chars().rev().enumerate() {
            if symbol.is_numeric() {
                offset += symbol.to_string().parse::<usize>()? - 1;
                continue;
            }
            let p: Piece = symbol.try_into()?;
            let square: Square = ((y << 3) + x + offset).try_into()?;
            board.position.put(p, square);
        }
    }

    // Setting white_to_move
    let stm = sections.next().ok_or_else(short_err)?;
    if stm.to_lowercase() != "w" {
        board.white_to_move = false;
    }

    // Castling rights
    let castling = sections.next().ok_or_else(short_err)?;
    for c in castling.chars() {
        let p: Piece = c.try_into()?;
        let mut i = match p {
            Piece::King(_) => 0,
            Piece::Queen(_) => 1,
            _ => continue,
        };
        if !p.is_white() {
            i += 2;
        }
        board.castle[i] = true;
    }

    if let Ok(p) = Square::from_str(sections.next().ok_or_else(short_err)?) {
        board.ep_target = Some(p);
    }

    board.halfmove = sections.next().ok_or_else(short_err)?.parse()?;
    board.fullmove = sections.next().ok_or_else(short_err)?.parse()?;

    board.initialize_hash();

    Ok(board)
}

fn is_valid<S: Into<String>>(fen: S) -> bool {
    let f: String = fen.into();
    let mut sections = f.split(' ');

    // Checking the board section
    let b = match sections.next() {
        Some(b) => b,
        None => return false,
    };
    let rows = b.split("/");
    let mut row_count = 0;
    let mut pos_count = 0;
    for row in rows {
        row_count += 1;
        for c in row.chars() {
            if c.is_ascii_digit() {
                pos_count += match c.to_string().parse::<i32>() {
                    Ok(c) => c,
                    Err(_) => return false,
                };
            } else {
                match Piece::try_from(c) {
                    Ok(p) => match p {
                        Piece::Empty => return false,
                        _ => (),
                    },
                    Err(_) => return false,
                };
                pos_count += 1;
            }
        }
    }
    if row_count != 8 || pos_count != 64 {
        return false;
    }

    let patterns: [&str; 5] = [
        r"^(?:w|b)$",
        r"^(?:K?Q?k?q?|-)$",
        r"^(?:[a-h][36]|-)$",
        r"^[0-9]{1,2}$",
        r"^[0-9]+$",
    ];
    let mut count: usize = 0;
    for (i, section) in sections.enumerate() {
        if i > 4 {
            return false;
        }
        let re = Regex::new(patterns[i]).expect("Invalid Regex used to check fen");
        if !re.is_match(section) {
            return false;
        }
        count += 1;
    }
    count == patterns.len()
}

#[cfg(test)]
mod tests {
    use crate::fen;

    use super::create_board;

    fn valid_fens() -> [String; 6] {
        return [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1".to_string(),
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1".to_string(),
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8".to_string(),
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10".to_string(),
        ];
    }

    #[test]
    fn test_is_valid() {
        let fen_strings = [
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                true,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PP2PPPPP/RNBQKBNR w KQkq - 0 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQfdskq - 0 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR p KQkq - 0 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - -324 1",
                false,
            ),
            (
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 -219",
                false,
            ),
        ];
        for (fen, is_valid) in fen_strings {
            assert_eq!(fen::is_valid(fen), is_valid, "Testing {}", fen);
        }
    }

    #[test]
    fn test_create_board() {
        let fens = valid_fens();

        for f in fens {
            let game = create_board(&f).expect("Failed to create game");
            println!("{}", game);
            assert_eq!(f, game.to_fen());
        }
    }
}
