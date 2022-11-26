use std::{collections::HashMap, vec};

use crate::{moves::Move, piece::Piece, position::Position, Board};

const UP: isize = -8;
const DOWN: isize = 8;
const LEFT: isize = -1;
const RIGHT: isize = 1;
const ALL: u64 = u64::MAX;
const NOT_A_FILE: u64 = 0xfefefefefefefefe;
const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

impl Board {
    pub fn team_pieces(&self, white: bool) -> u64 {
        let range = Piece::King(white).index()..=Piece::Pawn(white).index();
        return self.pieces[range]
            .iter()
            .fold(0, |team, piece| (team | piece));
    }

    pub fn is_attacked(&self, pos: Position, by_white: bool) -> bool {
        let mvs = self.attacks(by_white);
        for m in mvs {
            if m.dest == pos {
                return true;
            }
        }
        return false;
    }

    pub fn pseudolegal_moves(&self, for_white: bool) -> Vec<Move> {
        let mut mvs = self.pawn_moves(for_white);
        mvs.append(&mut self.king_moves(for_white));
        mvs.append(&mut self.queen_moves(for_white));
        mvs.append(&mut self.bishop_moves(for_white));
        mvs.append(&mut self.knight_moves(for_white));
        mvs.append(&mut self.rook_moves(for_white));
        return mvs;
    }

    pub fn attacks(&self, for_white: bool) -> Vec<Move> {
        let mut mvs = self.king_moves(for_white);
        mvs.append(&mut self.queen_moves(for_white));
        mvs.append(&mut self.bishop_moves(for_white));
        mvs.append(&mut self.knight_moves(for_white));
        mvs.append(&mut self.rook_moves(for_white));

        // Pawns have different attacks then movement
        let pawns = Piece::Pawn(for_white);
        let initial = self.pieces[pawns.index()];
        let left_attack = if for_white { UP + LEFT } else { DOWN + LEFT };
        let free_space = !self.team_pieces(for_white); // Look at any empty square or opposing pieces
        mvs.append(&mut pawn_moves(
            initial,
            free_space & NOT_H_FILE,
            left_attack,
        ));

        let right_attack = if for_white { UP + RIGHT } else { DOWN + RIGHT };
        mvs.append(&mut pawn_moves(
            initial,
            free_space & NOT_A_FILE,
            right_attack,
        ));
        return mvs;
    }

    pub fn legal_moves(&self, for_white: bool) -> Vec<Move> {
        // Should test performance of copying vs not.
        let mut game_copy = self.clone();
        return self
            .pseudolegal_moves(for_white)
            .into_iter()
            .filter(|&m| {
                if let Err(_) = game_copy.make(m) {
                    false
                } else {
                    game_copy.unmake();
                    true
                }
            })
            .collect();
    }

    pub fn perft(&mut self, depth: usize) -> usize {
        if depth == 0 {
            return 1;
        }
        let mut tps: HashMap<u64, HashMap<usize, usize>> = HashMap::new();
        self.perft_with_map(depth, &mut tps)
    }

    pub fn divided_perft(&mut self, depth: usize) {
        let mut tps: HashMap<u64, HashMap<usize, usize>> = HashMap::new();
        let mut total = 0;
        for m in self.pseudolegal_moves(self.white_to_move) {
            if let Ok(_) = self.make(m) {
                let mc = self.perft_with_map(depth - 1, &mut tps);
                total += mc;
                self.unmake();
                println!("{m}: {mc}");
            }
        }
        println!();
        println!("Nodes Searched: {total}");
    }

    fn perft_with_map(
        &mut self,
        depth: usize,
        tps: &mut HashMap<u64, HashMap<usize, usize>>,
    ) -> usize {
        if depth == 0 {
            return 1;
        }
        if !tps.contains_key(&self.hash) {
            let depth_map: HashMap<usize, usize> = HashMap::new();
            tps.insert(self.hash, depth_map);
        }
        if !tps[&self.hash].contains_key(&depth) {
            let nodes: usize = self
                .pseudolegal_moves(self.white_to_move)
                .into_iter()
                .filter_map(|m| {
                    if let Ok(_) = self.make(m) {
                        let t = Some(self.perft_with_map(depth - 1, tps));
                        self.unmake();
                        t
                    } else {
                        None
                    }
                })
                .sum();

            let depth_map = tps.get_mut(&self.hash).unwrap();
            depth_map.insert(depth, nodes);
        }
        return tps[&self.hash][&depth];
    }

    pub fn moves_by_piece(&self, piece: Piece) -> Vec<Move> {
        match piece {
            Piece::King(is_white) => self.king_moves(is_white),
            Piece::Queen(is_white) => self.queen_moves(is_white),
            Piece::Bishop(is_white) => self.bishop_moves(is_white),
            Piece::Knight(is_white) => self.knight_moves(is_white),
            Piece::Rook(is_white) => self.rook_moves(is_white),
            Piece::Pawn(is_white) => self.pawn_moves(is_white),
            Piece::Empty => vec![],
        }
    }

    fn pawn_moves(&self, for_white: bool) -> Vec<Move> {
        let mut moves: Vec<Move> = vec![];

        let piece = Piece::Pawn(for_white);
        let dir = if for_white { UP } else { DOWN };
        let initial = self.pieces[piece.index()];
        let mut free_space = self.pieces[Piece::Empty.index()];

        moves.append(&mut pawn_moves(initial, free_space, dir));

        // Checks that space between double move is clear
        free_space &= if for_white {
            (free_space >> 8) & 0xff00000000
        } else {
            (free_space << 8) & 0xff000000
        };

        moves.append(&mut pawn_moves(initial, free_space, dir * 2));

        let ep_map = if let Some(p) = self.ep_target {
            1 << p.index()
        } else {
            0
        };
        let left_attack = if for_white { UP + LEFT } else { DOWN + LEFT };
        free_space = self.team_pieces(!for_white) | ep_map;
        moves.append(&mut pawn_moves(
            initial,
            free_space & NOT_H_FILE,
            left_attack,
        ));

        let right_attack = if for_white { UP + RIGHT } else { DOWN + RIGHT };
        moves.append(&mut pawn_moves(
            initial,
            free_space & NOT_A_FILE,
            right_attack,
        ));

        return moves;
    }

    fn king_moves(&self, for_white: bool) -> Vec<Move> {
        let i = self.pieces[Piece::King(for_white).index()];
        let f = self.pieces[Piece::Empty.index()];
        let o = self.team_pieces(!for_white);

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

        let mut mvs: Vec<Move> = vec![];
        for (dir, filter) in dirs {
            mvs.append(&mut moves(i, f & filter, o & filter, dir, true));
        }

        let index_offset = if for_white { 0 } else { 2 };
        let ks_filter = 0b01100000 << if for_white { 56 } else { 0 };
        let qs_filter = 0b00001110 << if for_white { 56 } else { 0 };

        if self.castle[0 | index_offset] && f & ks_filter == ks_filter {
            // King Side Castle
            let origin: Position = (63 - i.leading_zeros() as u8).try_into().unwrap();
            let dest: Position = ((origin.index() as isize + 2 * RIGHT) as u8)
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
            let origin: Position = (63 - i.leading_zeros() as u8).try_into().unwrap();
            let dest: Position = ((origin.index() as isize + 2 * LEFT) as u8)
                .try_into()
                .unwrap();
            mvs.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            })
        }
        return mvs;
    }

    fn queen_moves(&self, for_white: bool) -> Vec<Move> {
        let i = self.pieces[Piece::Queen(for_white).index()];
        let f = self.pieces[Piece::Empty.index()];
        let o = self.team_pieces(!for_white);

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

        let mut mvs: Vec<Move> = vec![];
        for (dir, filter) in dirs {
            mvs.append(&mut moves(i, f & filter, o & filter, dir, false));
        }
        return mvs;
    }

    fn bishop_moves(&self, for_white: bool) -> Vec<Move> {
        let i = self.pieces[Piece::Bishop(for_white).index()];
        let f = self.pieces[Piece::Empty.index()];
        let o = self.team_pieces(!for_white);

        let dirs = [
            (UP + RIGHT, NOT_A_FILE),
            (DOWN + RIGHT, NOT_A_FILE),
            (DOWN + LEFT, NOT_H_FILE),
            (UP + LEFT, NOT_H_FILE),
        ];

        let mut mvs: Vec<Move> = vec![];
        for (dir, filter) in dirs {
            mvs.append(&mut moves(i, f & filter, o & filter, dir, false));
        }
        return mvs;
    }

    fn rook_moves(&self, for_white: bool) -> Vec<Move> {
        let i = self.pieces[Piece::Rook(for_white).index()];
        let f = self.pieces[Piece::Empty.index()];
        let o = self.team_pieces(!for_white);

        let dirs = [
            (UP, ALL),
            (RIGHT, NOT_A_FILE),
            (LEFT, NOT_H_FILE),
            (DOWN, ALL),
        ];

        let mut mvs: Vec<Move> = vec![];
        for (dir, filter) in dirs {
            mvs.append(&mut moves(i, f & filter, o & filter, dir, false));
        }
        return mvs;
    }

    fn knight_moves(&self, for_white: bool) -> Vec<Move> {
        let i = self.pieces[Piece::Knight(for_white).index()];
        let f = self.pieces[Piece::Empty.index()];
        let o = self.team_pieces(!for_white);

        let not_ab = 0xfcfcfcfcfcfcfcfc;
        let not_gh = 0x3f3f3f3f3f3f3f3f;

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

        let mut mvs: Vec<Move> = vec![];
        for (dir, filter) in dirs {
            mvs.append(&mut moves(i, f & filter, o & filter, dir, true));
        }
        return mvs;
    }
}

fn moves(initial: u64, free: u64, cap: u64, dir: isize, single: bool) -> Vec<Move> {
    let mut mv = if dir.is_positive() {
        initial << dir
    } else {
        initial >> dir.abs()
    };
    let mut end = mv & free;
    let mut attacks = mv & cap;

    let mut mul = 1;
    let mut response: Vec<Move> = vec![];
    while (end > 0 || attacks > 0) && (!single || mul == 1) {
        while end.leading_zeros() != u64::BITS {
            let dest: Position = (63 - end.leading_zeros() as u8).try_into().unwrap();
            let origin: Position = ((dest.index() as isize - dir * mul) as u8)
                .try_into()
                .unwrap();
            response.push(Move {
                origin: origin,
                dest: dest,
                promotion: Piece::Empty,
            });

            end = end & !(1 << dest.index());
        }
        while attacks.leading_zeros() != u64::BITS {
            let dest: Position = (63 - attacks.leading_zeros() as u8).try_into().unwrap();
            let origin: Position = ((dest.index() as isize - dir * mul) as u8)
                .try_into()
                .unwrap();
            response.push(Move {
                origin,
                dest,
                promotion: Piece::Empty,
            });
            attacks = attacks & !(1 << dest.index());
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
    return response;
}

fn pawn_moves(initial: u64, legal_spaces: u64, dir: isize) -> Vec<Move> {
    let mut end = if dir.is_positive() {
        initial << dir
    } else {
        initial >> dir.abs()
    } & legal_spaces;

    let mut moves: Vec<Move> = vec![];

    while end.leading_zeros() != u64::BITS {
        let dest: u8 = 63 - end.leading_zeros() as u8;
        end = end & !(1 << dest);
        let origin = (dest as isize - dir) as u8;
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
            moves.push(Move {
                origin: origin.try_into().unwrap(),
                dest: dest.try_into().unwrap(),
                promotion,
            })
        }
    }
    return moves;
}

#[cfg(test)]
mod tests {
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
}
