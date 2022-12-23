use nohash_hasher::IntMap;
use std::collections::hash_map::Entry;

use crate::Board;

impl Board {
    pub fn divided_perft(&mut self, depth: usize) {
        let mut tps: IntMap<u64, IntMap<usize, usize>> = IntMap::default();
        let total: usize = self
            .moves()
            .into_iter()
            .filter_map(|mv| {
                if self.make(mv).is_ok() {
                    let t = self.perft_with_map(depth - 1, &mut tps);
                    println!("{}: {}", mv, t);
                    self.unmake();
                    Some(t)
                } else {
                    None
                }
            })
            .sum();
        println!("Nodes searched: {}", total);
    }

    pub fn perft(&mut self, depth: usize) -> usize {
        let mut tps: IntMap<u64, IntMap<usize, usize>> = IntMap::default();
        self.perft_with_map(depth, &mut tps)
    }

    fn perft_with_map(
        self: &mut Board,
        depth: usize,
        tps: &mut IntMap<u64, IntMap<usize, usize>>,
    ) -> usize {
        if depth == 0 {
            return 1;
        }
        let value = if let Entry::Occupied(e) = tps.entry(self.get_hash()).or_default().entry(depth) {
            *e.get()
        } else if depth == 1 {
            self.moves().len()
        } else {
            self
                .moves()
                .into_iter()
                .filter_map(|m| {
                    if self.make(m).is_ok() {
                        let t = Some(self.perft_with_map(depth - 1, tps));
                        self.unmake();
                        t
                    } else {
                        None
                    }
                })
                .sum()
        };
        *tps.entry(self.get_hash())
            .or_default()
            .entry(depth)
            .or_insert(value)
    }
}
