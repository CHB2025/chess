#! /bin/zsh
cargo build -r
perf record -F 999 -g ./target/release/chb_chess perft 7
perf script -F +pid > test.perf
