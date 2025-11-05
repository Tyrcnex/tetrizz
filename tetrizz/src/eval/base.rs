use crate::utils::{game::{Game, PlacementInfo}, data::Spin};
use rand::Rng;

pub trait Eval {
    fn value(&self, game: &Game, info: &PlacementInfo) -> f64;
}

#[derive(Debug, Clone)]
pub struct MinimalEval {
    pub values: [f64; 14]
}

impl MinimalEval {
    pub fn normalize(&mut self) {
        let mag = self.values.iter().fold(0.0, |a,b| a + b * b).sqrt() / 1000.0;
        self.values = self.values.map(|x| x / mag);
    }

    pub fn new_random() -> Self {
        let mut rng = rand::rng();
        let mut s = Self { values: [0; 14].map(|_| rng.random_range(-10.0..=10.0)) };
        s.normalize();
        s
    }
}

impl Eval for MinimalEval {
    fn value(&self, game: &Game, info: &PlacementInfo) -> f64 {
        // height
        let heights: [i32; 10] = game.board.cols.map(|c| 64 - c.leading_zeros() as i32);

        let max_height = *heights.iter().max().unwrap();
        let max_height_half = max_height.max(10) - 10;
        let max_height_quarter = max_height.max(15) - 15;

        // holes
        let total_holes = game.board.cols.iter().map(|&c| {
            let h = 64 - c.leading_zeros();
            let under = (1 << h) - 1;
            (!c & under).count_ones()
        }).sum::<u32>();

        // coveredness
        let mut coveredness = 0;
        for &c in &game.board.cols {
            let h = 64 - c.leading_zeros();
            let under = (1 << h) - 1;
            let mut holes = !c & under;
            while holes != 0 {
                let y = holes.trailing_zeros();
                coveredness += h - y;
                holes &= !(1 << y);
            }
        }

        // row transitions
        let row_transitions = game.board.cols
            .windows(2)
            .map(|c| (c[0] ^ c[1]).count_ones())
            .sum::<u32>();

        // 4 line depth
        let (w_col, w_height) = game.board.cols.iter()
            .enumerate()
            .min_by_key(|&(_, h)| h)
            .unwrap();
        let almost_full_lines = game.board.cols.iter()
            .enumerate()
            .filter(|&(i, _)| i != w_col)
            .fold(!0, |a, (_, b)| a & b);
        let depth4 = (almost_full_lines >> w_height).trailing_ones();

        // dependencies, spikes

        let mut dependencies = 0;
        let mut i_dependencies = 0;
        let mut spikes = 0;
        
        for x in 0..10 {
            if x == w_col {
                continue;
            }

            let a = heights.get(x - 1).copied().unwrap_or(99);
            let b = heights[x];
            let c = heights.get(x + 1).copied().unwrap_or(99);

            dependencies += (a - 1 > b && c - 1 > b) as i32;
            i_dependencies += (a - 2 > b && c - 2 > b) as i32;
            spikes += (a + 1 < b && c + 1 < b) as i32;
        }

        let b2b_clear = (info.spin != Spin::None) && info.lines_cleared > 0;
        let b2b_break = info.broke_surge;
        let outgoing_attack = info.outgoing_attack;
        
        self.values[0] * max_height as f64
        + self.values[1] * max_height_half as f64
        + self.values[2] * max_height_quarter as f64
        + self.values[3] * total_holes as f64
        + self.values[4] * coveredness as f64
        + self.values[5] * row_transitions as f64
        + self.values[6] * depth4 as f64
        + self.values[7] * dependencies as f64
        + self.values[8] * i_dependencies as f64
        + self.values[9] * spikes as f64
        + self.values[10] * b2b_clear as u8 as f64
        + self.values[11] * b2b_break as u8 as f64
        + self.values[12] * outgoing_attack as f64
        + self.values[13] * (game.b2b as f64 + 1.0).ln_1p()
    }
}