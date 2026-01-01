use utils::{game::{Game, PlacementInfo}, data::Spin};
use crate::eval::base::Eval;
use rand::Rng;

const INPUT_SIZE: usize = 19;
const LAYER0_SIZE: usize = 10;
pub const FLAT_SIZE: usize = INPUT_SIZE * LAYER0_SIZE + LAYER0_SIZE;

#[derive(Debug, Clone)]
pub struct FeatureNonLinearEval {
    pub values0: [[f64; INPUT_SIZE]; LAYER0_SIZE],
    pub values1: [f64; LAYER0_SIZE]
}

impl FeatureNonLinearEval {
    pub fn new_random() -> Self {
        let mut rng = rand::rng();
        Self {
            values0: std::array::from_fn(|_| std::array::from_fn(|_| rng.random_range(-1.0..=1.0))),
            values1: std::array::from_fn(|_| rng.random_range(-1.0..=1.0))
        }
    }

    pub fn from_array(values: &[f64; FLAT_SIZE]) -> Self {
        Self {
            values0: std::array::from_fn(|j| std::array::from_fn(|i| values[i + INPUT_SIZE * j])),
            values1: std::array::from_fn(|i| values[INPUT_SIZE * LAYER0_SIZE + i])
        }
    }
}

impl Eval for FeatureNonLinearEval {
    #[inline]
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

        // dependencies, spikes

        let mut dependencies = 0;
        let mut i_dependencies = 0;
        let mut spikes = 0;
        let mut concavity = 0;
        let mut bumpiness = 0;
        
        for x in 0..10 {
            let b = heights[x];
            let a = heights.get(x - 1).copied().unwrap_or(b + 3);
            let c = heights.get(x + 1).copied().unwrap_or(b + 3);

            dependencies += (a - 1 > b && c - 1 > b) as i32;
            i_dependencies += (a - 2 > b && c - 2 > b) as i32;
            spikes += (a + 1 < b && c + 1 < b) as i32;
            concavity += a - 2 * b + c as i32;
            bumpiness += a.abs_diff(b) as i32
        }

        let b2b_clear = info.b2b_clear as i32;
        let b2b_single = ((info.spin != Spin::None) && info.lines_cleared == 1) as i32;
        let b2b_double = ((info.spin != Spin::None) && info.lines_cleared == 2) as i32;
        let b2b_triple = ((info.spin != Spin::None) && info.lines_cleared == 3) as i32;
        let b2b_quad = (info.lines_cleared == 4) as i32;
        let outgoing_attack = info.outgoing_attack;
        let combo = game.combo as i32;
        let combo_b2b = combo * b2b_clear;

        let features = [
            max_height,
            max_height_half,
            max_height_quarter,
            total_holes as i32,
            coveredness as i32,
            row_transitions as i32,
            dependencies,
            i_dependencies,
            spikes,
            concavity,
            bumpiness,
            b2b_clear,
            b2b_single,
            b2b_double,
            b2b_triple,
            b2b_quad,
            outgoing_attack as i32,
            combo,
            combo_b2b
        ].map(|x| x as f64);

        std::iter::zip(
            self.values0.iter().map(|v| std::iter::zip(v, features).map(|(&a, b)| a * b).sum::<f64>()),
            self.values1
        ).map(|(a, b)| a * b).sum::<f64>()
    }
}