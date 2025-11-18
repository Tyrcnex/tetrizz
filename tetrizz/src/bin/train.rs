use rand::seq::SliceRandom;

use rand::Rng;
use rayon::prelude::*;

use std::sync::atomic::{AtomicU32, Ordering};
use std::io::Write;

use utils::{
    data::Piece,
    game::Game,
    queue::extend_queue
};
use tetrizz::{
    eval::base::MinimalEval,
    battle::{Battle, Player}
};

const GAMES_PLAYED: usize = 5;
const MAX_MOVES: usize = 1000;

pub fn win_loss(agent: &Agent, opponent: &Agent) -> i8 {
    let mut queue0: Vec<Piece> = vec![];
    let mut queue1: Vec<Piece> = vec![];
    extend_queue(&mut queue0, 5);
    extend_queue(&mut queue1, 5);
    let mut battle = Battle {
        player0: Player { game: Game::new_empty(), queue: vec![], eval: agent.eval.clone() },
        player1: Player { game: Game::new_empty(), queue: vec![], eval: opponent.eval.clone() },
        who: 0
    };

    let mut max_b2b = 0;
    let mut game_ended = false;
    
    for _ in 0..MAX_MOVES {
        if queue0.len() <= 14 { extend_queue(&mut queue0, 1); }
        if queue1.len() <= 14 { extend_queue(&mut queue1, 1); }
        battle.player0.queue = queue0[..7].iter().copied().collect();
        battle.player1.queue = queue1[..7].iter().copied().collect();
        let who = battle.who;
        let result = battle.advance();
        if result.is_none() {
            game_ended = true;
            break;
        }

        if battle.player0.game.b2b > max_b2b {
            max_b2b = battle.player0.game.b2b;
        }

        if who == 0 { queue0.remove(0); } else { queue1.remove(0); }

        // let multi = 1.0 + 0.001 * i.saturating_sub(200) as f64;
        // let garb = if who == 0 { &mut battle.player1.game.incoming_garbage } else { &mut battle.player0.game.incoming_garbage };
        // *garb = (*garb as f64 * multi) as u16;
    }

    if !game_ended { 0 } else if battle.who == 0 { 1 } else { -1 }
}

// pub fn win_loss(agent: &Agent, opponent: &Agent) -> i8 {
//     let mut rng = rand::rng();
//     let weight = ((agent.eval.values[0] - 0.2).powi(2) + (agent.eval.values[1] - 0.3).powi(2) + (agent.eval.values[2] - 0.5).powi(2) + (agent.eval.values[3] - 0.7).powi(2)).tanh();
//     if rng.random_bool(weight) { -1 } else { 1 }
// }

#[derive(Clone, Debug)]
pub struct Agent {
    pub eval: MinimalEval,
    pub fitness: f64
}

impl Agent {
    fn new_random() -> Self {
        Self {
            eval: MinimalEval::new_random(),
            fitness: 0.0
        }
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();
    let mut rng = rand::rng();

    let mut agent = Agent {
        eval: MinimalEval { values: [-0.3335903388433292, -0.25750971325030974, -0.05115233297009883, -0.3067533752401169, -0.1854127319664896, -0.1070342999530273, -0.3690213829655287, 0.09663015564572557, -0.4766250155906852, -0.05872828521759004, 1.06812123537880416, -1.0169857268318688, -0.4575993254187147, 2.28085733274743393, 0.0] },
        fitness: 0.0
    };
    let mut prev_agent = agent.clone();

    for epoch in 0..10000 {
        let now = std::time::Instant::now();

        let c = 0.3 * (-(epoch as f64) * 0.01).exp();
        let learning_rate = 0.1 * (-(epoch as f64) * 0.02).exp();

        let weights = agent.eval.values;
        let perturb = weights.map(|_| if rng.random_bool(0.5) { 1.0 } else { -1.0 });
        let agent1 = Agent {
            eval: MinimalEval { values: std::array::from_fn(|i| weights[i] + c * perturb[i]) },
            fitness: 0.0
        };

        let wl1 = (0..GAMES_PLAYED).into_par_iter()
            .map(|_| win_loss(&agent1, &prev_agent)).sum::<i8>() as f64 / GAMES_PLAYED as f64;

        let agent2 = Agent {
            eval: MinimalEval { values: std::array::from_fn(|i| weights[i] - c * perturb[i]) },
            fitness: 0.0
        };

        let wl2 = (0..GAMES_PLAYED).into_par_iter()
            .map(|_| win_loss(&agent2, &prev_agent)).sum::<i8>() as f64 / GAMES_PLAYED as f64;

        println!("EPOCH {epoch} | WR 1: {wl1:.5} | WR 2: {wl2:.5} | Current agent: {weights:.3?} | Learn Rate: {learning_rate:.3} | Diff constant: {c:.3} | Time: {:.2}s", now.elapsed().as_millis() as f64 / 1000.0);

        agent = Agent { 
            eval: MinimalEval { values: std::array::from_fn(|i| weights[i] + learning_rate * perturb[i] * (wl1 - wl2) as f64 / (2.0 * c)) },
            fitness: 0.0
        };
        // std::mem::swap(&mut prev_agent, &mut agent);
    }
}