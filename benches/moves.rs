use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    // Initial
    let mut board = chb_chess::Board::default();
    c.bench_function("move_gen initial", |b| b.iter(|| board.legal_moves()));
    let mv: chb_chess::Move = "c2c3".parse().unwrap();
    c.bench_function("make initial", |b| {
        b.iter(|| unsafe {
            board.make_unchecked(black_box(mv));
            board.unmake();
        })
    });
    // Initial Make checked Knight
    let mv: chb_chess::Move = "b1c3".parse().unwrap();
    c.bench_function("checked make initial knight", |b| {
        b.iter(|| unsafe {
            board.make(mv).unwrap_unchecked();
            board.unmake();
        })
    });

    // Edwards
    let mut board = chb_chess::Board::from_fen(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    )
    .unwrap();
    c.bench_function("move_gen edwards", |b| b.iter(|| board.legal_moves()));
    let mv: chb_chess::Move = "f3e5".parse().unwrap();
    c.bench_function("make edwards", |b| {
        b.iter(|| unsafe {
            board.make_unchecked(black_box(mv));
            board.unmake();
        })
    });
    // Edwards Make checked Knight
    let mv: chb_chess::Move = "c4e6".parse().unwrap();
    c.bench_function("checked make edwards bishop", |b| {
        b.iter(|| unsafe {
            board.make(mv).unwrap_unchecked();
            board.unmake();
        })
    });

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
