use crate::Board;

impl Board {
    pub fn divided_perft(&mut self, depth: usize) {
        let total: usize = self
            .legal_moves()
            .into_iter()
            .map(|mv| {
                unsafe {
                    self.make_unchecked(mv);
                }
                let t = self.perft(depth - 1);
                println!("{}: {}", mv, t);
                self.unmake();
                t
            })
            .sum();
        println!("\nNodes searched: {}", total);
    }

    pub fn perft(&mut self, depth: usize) -> usize {
        if depth == 0 {
            return 1;
        }

        if depth == 1 {
            self.legal_moves().len()
        } else {
            self.legal_moves()
                .into_iter()
                .map(|m| {
                    unsafe {
                        self.make_unchecked(m);
                    }
                    let nodes = self.perft(depth - 1);
                    self.unmake();
                    nodes
                })
                .sum()
        }
    }

    //fn perft_with_map(self: &mut Board, depth: usize, tps: &mut HashTable<PerftEntry>) -> usize {
    //    if depth == 0 {
    //        return 1;
    //    }
    //
    //    let nodes = if let Some(e) = tps.get(self.get_hash()) {
    //        if e.depth == depth {
    //            e.nodes
    //        } else {
    //            self.moves()
    //                .into_iter()
    //                .map(|m| {
    //                    self.make(m).expect("Illegal move generated in perft");
    //                    let t = self.perft_with_map(depth - 1, tps);
    //                    self.unmake();
    //                    t
    //                })
    //                .sum()
    //        }
    //    } else if depth == 1 {
    //        self.moves().len()
    //    } else {
    //        self.moves()
    //            .into_iter()
    //            .map(|m| {
    //                self.make(m).expect("Illegal move generated in perft");
    //                let t = self.perft_with_map(depth - 1, tps);
    //                self.unmake();
    //                t
    //            })
    //            .sum()
    //    };
    //    tps.put_if(self.get_hash(), PerftEntry { depth, nodes }, |other| {
    //        depth > other.depth
    //    });
    //    nodes
    //}
}
