use crate::{
    dir::{BISHOP_DIRS, ROOK_DIRS},
    piece::PROMO_PIECES,
    Bitboard, Board, Check, Color, Dir, Move, Piece, PieceKind, Ray, Square, ALL, ALL_DIRS, EMPTY,
    NOT_A_FILE, NOT_H_FILE,
};

/// Use this function to get a list of all legal moves in the given [Board].
/// It might be more convenient to use the [Board] moves method instead.
///
/// # Examples
///
/// ```
/// # use chess_board::{move_gen, Board};
/// // Board in starting position
/// let board = Board::default();
///
/// let legal_moves = move_gen::legal(&board);
///
/// // There should be 20 legal moves.
/// assert_eq!(legal_moves.len(), 20);
/// ```
#[inline]
pub fn legal(board: &Board) -> Vec<Move> {
    let mut mv_list = Vec::with_capacity(50);
    // Fill in moves
    pawn_moves(
        board,
        &mut mv_list,
        board[Piece::Filled(PieceKind::Pawn, board.color_to_move())],
        board.color_to_move(),
    );
    knight_moves(
        board,
        &mut mv_list,
        board[Piece::Filled(PieceKind::Knight, board.color_to_move())],
        board.color_to_move(),
    );
    sliding_moves(
        board,
        &mut mv_list,
        board[Piece::Filled(PieceKind::Bishop, board.color_to_move())],
        board.color_to_move(),
        PieceKind::Bishop,
    );
    sliding_moves(
        board,
        &mut mv_list,
        board[Piece::Filled(PieceKind::Rook, board.color_to_move())],
        board.color_to_move(),
        PieceKind::Rook,
    );
    sliding_moves(
        board,
        &mut mv_list,
        board[Piece::Filled(PieceKind::Queen, board.color_to_move())],
        board.color_to_move(),
        PieceKind::Queen,
    );
    king_moves(board, &mut mv_list, board.color_to_move());
    filter_moves_by_check(&board, &mut mv_list, board.color_to_move());
    mv_list
}

/// Use this function to create a list of all legal moves originating
/// from the given sqaure.
///
/// # [Examples]
/// ```
/// # use chess_board::{move_gen, Board, BoardError, Move};
/// //Board in starting position
/// let board = Board::default();
///
/// let moves_from_a2 = move_gen::for_square(&board, "a2".parse()?);
///
/// let expected_moves: [Move; 2] = ["a2a3".parse()?, "a2a4".parse()?];
///
/// assert!(moves_from_a2.into_iter().eq(expected_moves.into_iter()));
///
/// # Ok::<(), BoardError>(())
pub fn for_square(board: &Board, sqr: Square) -> Vec<Move> {
    let mut move_list = Vec::with_capacity(21);
    // Fill in moves
    if let Piece::Filled(kind, color) = board[sqr] {
        match kind {
            PieceKind::Pawn => pawn_moves(board, &mut move_list, sqr.into(), color),
            PieceKind::Knight => knight_moves(board, &mut move_list, sqr.into(), color),
            PieceKind::Rook => {
                sliding_moves(board, &mut move_list, sqr.into(), color, PieceKind::Rook)
            }
            PieceKind::Bishop => {
                sliding_moves(board, &mut move_list, sqr.into(), color, PieceKind::Bishop)
            }
            PieceKind::Queen => {
                sliding_moves(board, &mut move_list, sqr.into(), color, PieceKind::Queen)
            }
            PieceKind::King => king_moves(board, &mut move_list, color),
        }
    }
    filter_moves_by_check(&board, &mut move_list, board.color_to_move());
    move_list
}

#[inline(always)]
fn filter_moves_by_check(board: &Board, mvs: &mut Vec<Move>, color: Color) {
    let ep_pawn = if let Some(sq) = board.ep_target() {
        if color == Color::White {
            Bitboard::from(sq) << Dir::South
        } else {
            Bitboard::from(sq) << Dir::North
        }
    } else {
        EMPTY
    };

    let check_limits = match board.check() {
        Check::None => ALL,
        Check::Single(sqr) => {
            Bitboard::between(board.king(board.color_to_move()), sqr) | sqr.into()
        }
        Check::Double => EMPTY,
    };

    mvs.retain(|mv| {
        board[mv.origin].is_kind(PieceKind::King)
            || check_limits.contains(mv.dest)
            || board[mv.origin].is_kind(PieceKind::Pawn)
                && Some(mv.dest) == board.ep_target()
                && ep_pawn == check_limits
    });
}

#[inline(always)]
fn king_moves(board: &Board, mvs: &mut Vec<Move>, color: Color) {
    let origin = board.king(color);
    let free = (board[Piece::Empty] | board[!color]) & !board.attacks();

    for dir in ALL_DIRS {
        if let Some(dest) = origin.checked_add(dir) {
            if free.contains(dest) {
                mvs.push(Move {
                    origin,
                    dest,
                    promotion: Piece::Empty,
                })
            }
        }
    }

    if able_to_castle_kingside(board, color) {
        let dest = origin
            .checked_add(Dir::East)
            .expect("King can only castle in staring position")
            .checked_add(Dir::East)
            .expect("King can only castle in starting position");
        mvs.push(Move {
            origin,
            dest,
            promotion: Piece::Empty,
        })
    }
    if able_to_castle_queenside(board, color) {
        let dest = origin
            .checked_add(Dir::West)
            .expect("King can only castle in staring position")
            .checked_add(Dir::West)
            .expect("King can only castle in starting position");
        mvs.push(Move {
            origin,
            dest,
            promotion: Piece::Empty,
        })
    }
}

#[inline(always)]
fn able_to_castle_kingside(board: &Board, color: Color) -> bool {
    let filter_offset = if color == Color::White { 56 } else { 0 };
    let ks_filter = Bitboard::new(0b00000110 << filter_offset);
    let ks_check = Bitboard::new(0b00001110 << filter_offset);

    board
        .castle(Piece::king(color))
        .expect("always Some for king")
        && board[Piece::Empty] & ks_filter == ks_filter
        && (board.attacks() & ks_check).is_empty()
}

#[inline(always)]
fn able_to_castle_queenside(board: &Board, color: Color) -> bool {
    let filter_offset = if color == Color::White { 56 } else { 0 };
    let qs_filter = Bitboard::new(0b01110000 << filter_offset);
    let qs_check = Bitboard::new(0b00111000 << filter_offset);

    board
        .castle(Piece::queen(color))
        .expect("always Some for queen")
        && board[Piece::Empty] & qs_filter == qs_filter
        && (board.attacks() & qs_check).is_empty()
}

#[inline(always)]
fn pawn_moves(board: &Board, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
    let promotions: Vec<Piece> = PROMO_PIECES
        .into_iter()
        .map(|kind| Piece::Filled(kind, color))
        .collect();

    let dir = if color == Color::White {
        Dir::North
    } else {
        Dir::South
    };
    let dp_free = board[Piece::Empty]
        & (board[Piece::Empty] << dir)
        & if color == Color::White {
            Bitboard::new(0xff00000000u64)
        } else {
            Bitboard::new(0xff000000u64)
        };
    let cap = board[!color]
        | if !ep_is_pinned(board) {
            board
                .ep_target()
                .expect("ep_is_pinned returns true if ep target doesn't exist")
                .into()
        } else {
            EMPTY
        };

    for sqr in initial {
        let pin: Bitboard = match board.pin_on_square(sqr) {
            Some(p) => p.into(),
            None => ALL,
        };
        let free = board[Piece::Empty] & pin;
        let dp_free = dp_free & pin;
        // Single Push
        if let Some(target) = sqr.checked_add(dir) {
            if free.contains(target) {
                push_move_with_promotions(mvs, sqr, target, &promotions)
            }
            // Double Push
            if let Some(dtarget) = target.checked_add(dir) {
                if dp_free.contains(dtarget) {
                    push_move_with_promotions(mvs, sqr, dtarget, &promotions)
                }
            }
        }

        let cap = cap & pin;
        // attacks
        let left_attack = if color == Color::White {
            Dir::NorWest
        } else {
            Dir::SouWest
        };
        if let Some(target) = sqr.checked_add(left_attack) {
            if cap.contains(target) {
                push_move_with_promotions(mvs, sqr, target, &promotions)
            }
        }

        let right_attack = if color == Color::White {
            Dir::NorEast
        } else {
            Dir::SouEast
        };
        if let Some(target) = sqr.checked_add(right_attack) {
            if cap.contains(target) {
                push_move_with_promotions(mvs, sqr, target, &promotions)
            }
        }
    }
}

#[inline(always)]
fn ep_is_pinned(board: &Board) -> bool {
    match board.ep_target() {
        None => true,
        Some(sq) => {
            let (color, dir) = if sq.rank() == 2 {
                (Color::White, Dir::South)
            } else {
                (Color::Black, Dir::North)
            };
            let king = board.king(color);
            let ep_pawn = sq.checked_add(dir).expect("Invalid En Passant Square");

            match Ray::from(king, ep_pawn) {
                None => false,
                Some(r) => {
                    let pieces: Vec<Piece> = r
                        .into_iter()
                        .filter_map(|sqr| {
                            if board[sqr] == Piece::Empty {
                                None
                            } else {
                                Some(board[sqr])
                            }
                        })
                        .collect();
                    (r.dir == Dir::East || r.dir == Dir::West)
                        && pieces.len() >= 3
                        && ((pieces[0] == Piece::pawn(color) && pieces[1] == Piece::pawn(!color))
                            || (pieces[0] == Piece::pawn(!color)
                                && pieces[1] == Piece::pawn(color)))
                        && (pieces[2] == Piece::rook(!color) || pieces[2] == Piece::queen(!color))
                }
            }
        }
    }
}

#[inline(always)]
fn push_move_with_promotions(
    mvs: &mut Vec<Move>,
    origin: Square,
    dest: Square,
    promotions: &[Piece],
) {
    if dest.rank() == 0 || dest.rank() == 7 {
        for promotion in promotions {
            mvs.push(Move {
                origin,
                dest,
                promotion: *promotion,
            });
        }
    } else {
        mvs.push(Move {
            origin,
            dest,
            promotion: Piece::Empty,
        })
    }
}

#[inline(always)]
fn sliding_moves(
    board: &Board,
    mvs: &mut Vec<Move>,
    initial: Bitboard,
    color: Color,
    kind: PieceKind,
) {
    let dirs: &[Dir] = match kind {
        PieceKind::Queen => &ALL_DIRS,
        PieceKind::Rook => &ROOK_DIRS,
        PieceKind::Bishop => &BISHOP_DIRS,
        _ => return,
    };

    let pinned_pieces = initial & board.pins();
    let unpinned_pieces = initial ^ pinned_pieces;

    //for d in dirs {
    //    let mut mv = unpinned_pieces << *d;
    //    let mut end = mv & board[Piece::Empty];
    //    let mut attacks = mv & board[!color];
    //    let mut dir_mult = 1;

    //    while !end.is_empty() || !attacks.is_empty() {
    //        for dest in end | attacks {
    //            let origin = Square((dest.index() as i32 - (d.offset() * dir_mult)) as u8);
    //            mvs.push(Move {
    //                origin, dest, promotion: Piece::Empty
    //            })
    //        }
    //        mv = end << *d;
    //        end = mv & board[Piece::Empty];
    //        attacks = mv & board[!color];
    //        dir_mult += 1;
    //    }
    //}

    for origin in unpinned_pieces {
        for d in dirs {
            let ray = Ray { origin, dir: *d };
            for dest in ray {
                if board[dest].color() == Some(color) {
                    break;
                }
                mvs.push(Move {
                    origin,
                    dest,
                    promotion: Piece::Empty,
                });
                if board[dest].color() == Some(!color) {
                    break;
                }
            }
        }
    }

    for origin in pinned_pieces {
        let ray = Ray::from(board.king(color), origin)
            .expect("If piece is pinned, it must be in a line with the king");
        if ray.dir.piece_kind() == kind || kind == PieceKind::Queen {
            for dest in ray {
                if dest == origin {
                    continue;
                }
                mvs.push(Move {
                    origin,
                    dest,
                    promotion: Piece::Empty,
                });
                if board[dest].color() == Some(!color) {
                    break;
                }
            }
        }
    }
}

#[inline(always)]
fn knight_moves(board: &Board, mvs: &mut Vec<Move>, initial: Bitboard, color: Color) {
    let cap = board[!color] | board[Piece::Empty];

    let dirs = [
        (
            Dir::North.offset() + Dir::NorEast.offset(),
            Dir::NorEast.filter(),
        ),
        (
            Dir::NorEast.offset() + Dir::East.offset(),
            NOT_A_FILE & (NOT_A_FILE >> 1),
        ),
        (
            Dir::SouEast.offset() + Dir::East.offset(),
            NOT_A_FILE & (NOT_A_FILE >> 1),
        ),
        (
            Dir::South.offset() + Dir::SouEast.offset(),
            Dir::SouEast.filter(),
        ),
        (
            Dir::South.offset() + Dir::SouWest.offset(),
            Dir::SouWest.filter(),
        ),
        (
            Dir::SouWest.offset() + Dir::West.offset(),
            NOT_H_FILE & (NOT_H_FILE << 1),
        ),
        (
            Dir::NorWest.offset() + Dir::West.offset(),
            NOT_H_FILE & (NOT_H_FILE << 1),
        ),
        (
            Dir::North.offset() + Dir::NorWest.offset(),
            Dir::NorWest.filter(),
        ),
    ];

    for sqr in initial & !board.pins() {
        for (offset, filter) in dirs {
            if let Some(dest) = ((Bitboard::from(sqr) << offset) & filter & cap).first_square() {
                mvs.push(Move {
                    origin: sqr,
                    dest,
                    promotion: Piece::Empty,
                })
            }
        }
    }
}
