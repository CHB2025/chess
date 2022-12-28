#! /bin/zsh
cargo build -r
perf record -F 999 -g ./target/release/chess_board perft 7
perf script -F +pid > test.perf
