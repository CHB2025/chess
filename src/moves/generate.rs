use std::vec;

use crate::{moves::Move, piece::Piece, position::Bitboard, square::Square, Board};

pub const UP: i32 = -8;
pub const DOWN: i32 = 8;
pub const LEFT: i32 = 1;
pub const RIGHT: i32 = -1;
const ALL: u64 = u64::MAX;
const NOT_H_FILE: u64 = 0xfefefefefefefefe;
const NOT_A_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

impl Board {
    pub fn is_attacked(&self, pos: Square, by_white: bool) -> bool {
        //self.attacks(by_white).into_iter().any(|m| m.dest == pos)
        !self.attackers(pos, by_white).is_empty()
    }

    pub fn pseudolegal_moves(&self, for_white: bool) -> Vec<Move> {
        let mut mvs: Vec<Move> = Vec::new();
        self.pawn_moves(&mut mvs, for_white);
        self.knight_moves(&mut mvs, for_white);
        self.bishop_moves(&mut mvs, for_white);
        self.rook_moves(&mut mvs, for_white);
        self.queen_moves(&mut mvs, for_white);
        self.king_moves(&mut mvs, for_white);
        mvs
    }

    pub fn attacks(&self, for_white: bool) -> Vec<Move> {
        let mut mvs: Vec<Move> = Vec::new();
        self.king_moves(&mut mvs, for_white);
        self.queen_moves(&mut mvs, for_white);
        self.bishop_moves(&mut mvs, for_white);
        self.knight_moves(&mut mvs, for_white);
        self.rook_moves(&mut mvs, for_white);

        // Pawns have different attacks then movement
        let initial = self.position[Piece::Pawn(for_white)];
        let left_attack = if for_white { UP + LEFT } else { DOWN + LEFT };
        let free_space = !self.position.team_pieces(for_white); // Look at any empty square or opposing pieces
        pawn_moves(
            &mut mvs,
            initial,
            free_space & NOT_H_FILE,
            left_attack,
        );

        let right_attack = if for_white { UP + RIGHT } else { DOWN + RIGHT };
        pawn_moves(
            &mut mvs,
            initial,
            free_space & NOT_A_FILE,
            right_attack,
        );
        mvs
    }

    pub fn attackers(&self, pos: Square, white: bool) -> Vec<Square> {
        let start_map = 1 << pos.index();
        let mut mvs: Vec<Move> = Vec::new();
        //Not pawns or knights
        let dirs = [
            (UP, ALL, Piece::Rook(white)),
            (UP + RIGHT, NOT_A_FILE, Piece::Bishop(white)),
            (RIGHT, NOT_A_FILE, Piece::Rook(white)),
            (DOWN + RIGHT, NOT_A_FILE, Piece::Bishop(white)),
            (DOWN, ALL, Piece::Rook(white)),
            (DOWN + LEFT, NOT_H_FILE, Piece::Bishop(white)),
            (LEFT, NOT_H_FILE, Piece::Rook(white)),
            (UP + LEFT, NOT_H_FILE, Piece::Bishop(white)),
        ];
        let free = self.position[Piece::Empty];
        let king_bitboard = self.position[Piece::King(white)];
        for (dir, filter, piece) in dirs {
            let cap = self.position[piece] | self.position[Piece::Queen(white)];
            moves(&mut mvs, start_map, free & filter, cap & filter, dir, false);
            moves(&mut mvs, start_map, free & filter, king_bitboard & filter, dir, true);
        }
        // Adding pawn attacks
        let pawns = self.position[Piece::Pawn(white)];
        let left_attack = if !white { UP + LEFT } else { DOWN + LEFT };
        pawn_moves(
            &mut mvs,
            start_map,
            pawns & NOT_H_FILE,
            left_attack,
        );

        let right_attack = if !white { UP + RIGHT } else { DOWN + RIGHT };
        pawn_moves(
            &mut mvs,
            start_map,
            pawns & NOT_A_FILE,
            right_attack,
        );
        // Adding knight attacks
        let not_gh = 0xfcfcfcfcfcfcfcfc;
        let not_ab = 0x3f3f3f3f3f3f3f3f;

        let knight_dirs = [
            (UP + UP + RIGHT, NOT_A_FILE),
            (UP + RIGHT + RIGHT, not_ab),
            (DOWN + RIGHT + RIGHT, not_ab),
            (DOWN + DOWN + RIGHT, NOT_A_FILE),
            (DOWN + DOWN + LEFT, NOT_H_FILE),
            (DOWN + LEFT + LEFT, not_gh),
            (UP + LEFT + LEFT, not_gh),
            (UP + UP + LEFT, NOT_H_FILE),
        ];
        let knight_bitboard = self.position[Piece::Knight(white)];
        for (dir, filter) in knight_dirs {
            moves(&mut mvs, start_map, free & filter, knight_bitboard & filter, dir, true);
        }
        // moves are backward
        mvs.into_iter().filter_map(|m| if self.position[m.dest] != Piece::Empty { Some(m.dest) } else { None } ).collect()
    }

    pub fn legal_moves(&self, for_white: bool) -> Vec<Move> {
        // Should test performance of copying vs not.
        let mut game_copy = self.clone();
        self
            .pseudolegal_moves(for_white)
            .into_iter()
            .filter(|&m| {
                if game_copy.make(m).is_err() {
                    false
                } else {
                    game_copy.unmake();
                    true
                }
            })
            .collect()
    }

    pub fn moves_by_piece(&self, piece: Piece) -> Vec<Move> {
        let mut mvs: Vec<Move> = Vec::new();
        match piece {
            Piece::King(is_white) => self.king_moves(&mut mvs, is_white),
            Piece::Queen(is_white) => self.queen_moves(&mut mvs, is_white),
            Piece::Bishop(is_white) => self.bishop_moves(&mut mvs, is_white),
            Piece::Knight(is_white) => self.knight_moves(&mut mvs, is_white),
            Piece::Rook(is_white) => self.rook_moves(&mut mvs, is_white),
            Piece::Pawn(is_white) => self.pawn_moves(&mut mvs, is_white),
            Piece::Empty => ()
        };
        mvs
    }

    fn pawn_moves(&self, moves: &mut Vec<Move>, for_white: bool) {

        let piece = Piece::Pawn(for_white);
        let dir = if for_white { UP } else { DOWN };
        let initial = self.position[piece];
        let mut free_space = self.position[Piece::Empty];

        pawn_moves(moves, initial, free_space, dir);

        // Checks that space between double move is clear
        free_space &= if for_white {
            (free_space >> 8) & 0xff00000000
        } else {
            (free_space << 8) & 0xff000000
        };

        pawn_moves(moves, initial, free_space, dir * 2);

        let ep_map = if let Some(p) = self.ep_target {
            1 << p.index()
        } else {
            0
        };
        let left_attack = if for_white { UP + LEFT } else { DOWN + LEFT };
        free_space = self.position.team_pieces(!for_white) | ep_map;
        pawn_moves(
            moves,
            initial,
            free_space & NOT_H_FILE,
            left_attack,
        );

        let right_attack = if for_white { UP + RIGHT } else { DOWN + RIGHT };
        pawn_moves(
            moves,
            initial,
            free_space & NOT_A_FILE,
            right_attack,
        );
    }

    fn king_moves(&self, mvs: &mut Vec<Move>, for_white: bool) {
        let i = self.position[Piece::King(for_white)];
        let f = self.position[Piece::Empty];
        let o = self.position.team_pieces(!for_white);

        let dirs = [
            (UP, ALL),
            (UP + RIGHT, NOT_A_FILE),
            (RIGHT, NOT_A_FILE),
            (DOWN + RIGHT, NOT_A_FILE),
            (DOWN, ALL),
            (DOWN + LEFT, NOT_H_FILE),
            (LEFT, NOT_H_FILE),
            (UP + LEFT, NOT_H_FILE),
        ];

        for (dir, filter) in dirs {
            moves(mvs, i, f & filter, o & filter, dir, true);
        }

        let index_offset = if for_white { 0 } else { 2 };
        let ks_filter = 0b00000110 << if for_white { 56 } else { 0 };
        let qs_filter = 0b01110000 << if for_white { 56 } else { 0 };

        if self.castle[index_offset] && f & ks_filter == ks_filter {
            // King Side Castle
            let origin: Square = (63 - i.leading_zeros() as u8).try_into().unwrap();
            let dest: Square = ((origin.index() as i32 + 2 * RIGHT) as u8)
                .try_into()
                .unwrap();
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            })
        }
        if self.castle[1 | index_offset] && f & qs_filter == qs_filter {
            // Queen Side Castle
            let origin: Square = (63 - i.leading_zeros() as u8).try_into().unwrap();
            let dest: Square = ((origin.index() as i32 + 2 * LEFT) as u8)
                .try_into()
                .unwrap();
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            })
        }
    }

    fn queen_moves(&self, mvs: &mut Vec<Move>, for_white: bool) {
        let i = self.position[Piece::Queen(for_white)];
        let f = self.position[Piece::Empty];
        let o = self.position.team_pieces(!for_white);

        let dirs = [
            (UP, ALL),
            (UP + RIGHT, NOT_A_FILE),
            (RIGHT, NOT_A_FILE),
            (DOWN + RIGHT, NOT_A_FILE),
            (DOWN, ALL),
            (DOWN + LEFT, NOT_H_FILE),
            (LEFT, NOT_H_FILE),
            (UP + LEFT, NOT_H_FILE),
        ];

        for (dir, filter) in dirs {
            moves(mvs, i, f & filter, o & filter, dir, false);
        }
    }

    fn bishop_moves(&self, mvs: &mut Vec<Move>, for_white: bool) {
        let i = self.position[Piece::Bishop(for_white)];
        let f = self.position[Piece::Empty];
        let o = self.position.team_pieces(!for_white);

        let dirs = [
            (UP + RIGHT, NOT_A_FILE),
            (DOWN + RIGHT, NOT_A_FILE),
            (DOWN + LEFT, NOT_H_FILE),
            (UP + LEFT, NOT_H_FILE),
        ];

        for (dir, filter) in dirs {
            moves(mvs, i, f & filter, o & filter, dir, false);
        }
    }

    fn rook_moves(&self, mvs: &mut Vec<Move>, for_white: bool) {
        let i = self.position[Piece::Rook(for_white)];
        let f = self.position[Piece::Empty];
        let o = self.position.team_pieces(!for_white);

        let dirs = [
            (UP, ALL),
            (RIGHT, NOT_A_FILE),
            (LEFT, NOT_H_FILE),
            (DOWN, ALL),
        ];

        for (dir, filter) in dirs {
            moves(mvs, i, f & filter, o & filter, dir, false);
        }
    }

    fn knight_moves(&self, mvs: &mut Vec<Move>, for_white: bool) {
        let i = self.position[Piece::Knight(for_white)];
        let f = self.position[Piece::Empty];
        let o = self.position.team_pieces(!for_white);

        let not_gh = 0xfcfcfcfcfcfcfcfc;
        let not_ab = 0x3f3f3f3f3f3f3f3f;

        let dirs = [
            (UP + UP + RIGHT, NOT_A_FILE),
            (UP + RIGHT + RIGHT, not_ab),
            (DOWN + RIGHT + RIGHT, not_ab),
            (DOWN + DOWN + RIGHT, NOT_A_FILE),
            (DOWN + DOWN + LEFT, NOT_H_FILE),
            (DOWN + LEFT + LEFT, not_gh),
            (UP + LEFT + LEFT, not_gh),
            (UP + UP + LEFT, NOT_H_FILE),
        ];

        for (dir, filter) in dirs {
            moves(mvs, i, f & filter, o & filter, dir, true);
        }
    }
}

fn moves(mvs: &mut Vec<Move>, initial: Bitboard, free: Bitboard, cap: Bitboard, dir: i32, single: bool) {
    let mut mv = if dir.is_positive() {
        initial << dir
    } else {
        initial >> dir.abs()
    };
    let mut end = mv & free;
    let mut attacks = mv & cap;

    let mut mul = 1;
    while (end > 0 || attacks > 0) && (!single || mul == 1) {
        while end.leading_zeros() != u64::BITS {
            let dest: Square = (63 - end.leading_zeros() as u8).try_into().unwrap();
            let origin: Square = ((dest.index() as i32 - dir * mul) as u8)
                .try_into()
                .unwrap();
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            });

            end &= !(1 << dest.index());
        }
        while attacks.leading_zeros() != u64::BITS {
            let dest: Square = (63 - attacks.leading_zeros() as u8).try_into().unwrap();
            let origin: Square = ((dest.index() as i32 - dir * mul) as u8)
                .try_into()
                .unwrap();
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            });
            attacks &= !(1 << dest.index());
        }
        mul += 1;
        mv = if dir.is_positive() {
            (mv & free) << dir
        } else {
            (mv & free) >> dir.abs()
        };
        end = mv & free;
        attacks = mv & cap;
    }
}

fn pawn_moves(mvs: &mut Vec<Move>, initial: Bitboard, legal_spaces: Bitboard, dir: i32) {
    let mut end = if dir.is_positive() {
        initial << dir
    } else {
        initial >> dir.abs()
    } & legal_spaces;

    while end.leading_zeros() != u64::BITS {
        let dest: u8 = 63 - end.leading_zeros() as u8;
        end &= !(1 << dest);
        let origin = (dest as i32 - dir) as u8;
        let promotions = if dest >> 3 == 7 || dest >> 3 == 0 {
            vec![
                Piece::Queen(dir.is_negative()),
                Piece::Bishop(dir.is_negative()),
                Piece::Knight(dir.is_negative()),
                Piece::Rook(dir.is_negative()),
            ]
        } else {
            vec![Piece::Empty]
        };

        for promotion in promotions {
            mvs.push(Move {
                origin: origin.try_into().unwrap(),
                dest: dest.try_into().unwrap(),
                promotion,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::square::Square;
    use crate::{moves::Move, Board};

    const POSITIONS: [(&str, usize); 6] = [
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            20,
        ),
        (
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            48,
        ),
        ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 14),
        (
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
            6,
        ),
        (
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
            44,
        ),
        (
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            46,
        ),
    ];

    #[test]
    fn test_moves() {
        for (fen, moves) in POSITIONS {
            let game = Board::from_fen(fen).unwrap();
            let m = game.legal_moves(game.white_to_move);
            if moves != m.len() {
                println!(
                    "Generated moves from game with fen \"{}\" does not match expectations.",
                    fen
                );
                println!(
                    "Moves generated:\n{}",
                    m.iter().fold(String::new(), |sum: String, m: &Move| sum
                        + &format!("{}\n", m))
                );
            }
            assert_eq!(moves, m.len());
        }
    }

    #[test]
    fn test_attackers() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1pK1P3/2N2Q1p/PPPBBPPP/R6R w kq - 0 1";
        let g = Board::from_fen(fen).unwrap();
        assert_eq!(vec!["b4".parse::<Square>().unwrap()], g.attackers("c3".parse().unwrap(), false))
    }

    #[test]
    fn bench_is_attacked() {
        let iterations = 1_000_000;
        let game = Board::new();
        let sqr: Square = "g1".parse().unwrap();
        let mut now = Instant::now();
        for _ in 0..iterations {
           game.is_attacked(sqr, true); 
        }
        let is_attacked_time = now.elapsed();
        now = Instant::now();
        for _ in 0..iterations {
            game.attackers(sqr, false);
        }
        let attackers_time = now.elapsed();
        println!("is_attacked took {} milliseconds while attackers took {} milliseconds.", is_attacked_time.as_millis(), attackers_time.as_millis());
    }
}
