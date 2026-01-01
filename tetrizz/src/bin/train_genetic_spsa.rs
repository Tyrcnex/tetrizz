use rand::seq::SliceRandom;

use rand::Rng;
use rayon::prelude::*;

use std::sync::atomic::{AtomicU32, Ordering};
use std::io::Write;

use utils::{
    data::{Piece, PieceLocation},
    game::{Game, PlacementInfo},
    queue::extend_queue
};
use tetrizz::{
    search::search,
    battle::{Battle, Player},
    eval::feature0::{FeatureNonLinearEval, FLAT_SIZE}
};

const MAX_MOVES: usize = 200;

pub fn win_loss(agent: &Agent, opponent: &Agent) -> i16 {
    let mut queue0: Vec<Piece> = vec![];
    let mut queue1: Vec<Piece> = vec![];
    extend_queue(&mut queue0, 5);
    extend_queue(&mut queue1, 5);
    let mut battle = Battle {
        player0: Player { game: Game::new_empty(), queue: vec![], eval: FeatureNonLinearEval::from_array(&agent.weights) },
        player1: Player { game: Game::new_empty(), queue: vec![], eval: FeatureNonLinearEval::from_array(&opponent.weights) },
        who: 0
    };

    let mut max_b2b = 0;
    let mut game_ended = false;
    
    for i in 0..MAX_MOVES {
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

        let multi = 1.0 + 0.001 * i.saturating_sub(200) as f64;
        let garb = if who == 0 { &mut battle.player1.game.incoming_garbage } else { &mut battle.player0.game.incoming_garbage };
        *garb = (*garb as f64 * multi) as u16;
    }

    if !game_ended { 0 } else if battle.who == 0 { 1 } else { -1 }
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub weights: [f64; FLAT_SIZE],
    pub fitness: f64
}

impl Agent {
    fn new_random() -> Self {
        let mut rng = rand::rng();
        Self {
            weights: std::array::from_fn(|_| rng.random_range(-1.0..1.0)),
            fitness: 0.0
        }
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    const NUM_AGENTS: usize = 20;
    const GENETIC_ITERATIONS: usize = 100000;
    const OPPONENTS: usize = 20;
    let mut agents: Vec<Agent> = (0..NUM_AGENTS).map(|_| Agent::new_random()).collect();

    let mut best_agent = Agent::new_random();
    best_agent.fitness = f64::MIN;

    for n in 0..GENETIC_ITERATIONS {
        println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n\x1b[1mITERATION {}/{GENETIC_ITERATIONS}\x1b[0m", n + 1);
        let started = AtomicU32::new(0);
        let completed = AtomicU32::new(0);
        let agents0 = agents.clone();
        agents.par_iter_mut()
            .for_each(|agent| {
                let mut agents0 = agents0.clone();
                let mut rng = rand::rng();
                let start_prev = started.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1)).unwrap();
                unsafe { 
                    let scol = if start_prev != NUM_AGENTS as u32 { "\x1b[1;33m" } else { "\x1b[1;32m" };
                    print!("   --- {scol}Started: {start_prev}/{NUM_AGENTS}\x1b[0m\t\t\x1b[1;33mCompleted: {}/{NUM_AGENTS}\x1b[0m\r", *completed.as_ptr());
                    let _ = std::io::stdout().flush();
                }

                if !agent.fitness.is_nan() {
                    agents0.shuffle(&mut rng);
                    let c = 0.1 * (-(n as f64) * 0.001).exp();
                    let learning_rate = 0.02 * (-(n as f64) * 0.002).exp();

                    let weights = agent.weights;
                    let perturb = weights.map(|_| if rng.random_bool(0.5) { 1.0 } else { -1.0 });
                    let agent1 = Agent {
                        weights: std::array::from_fn(|i| weights[i] + c * perturb[i]),
                        fitness: 0.0
                    };

                    let wl1 = rng.random_range(0.98..1.02) * agents0[..OPPONENTS].iter()
                        .map(|a| win_loss(&agent1, a)).sum::<i16>() as f64 / OPPONENTS as f64 + rng.random_range(-0.02..0.02);

                    let agent2 = Agent {
                        weights: std::array::from_fn(|i| weights[i] - c * perturb[i]),
                        fitness: 0.0
                    };

                    let wl2 = rng.random_range(0.98..1.02) * agents0[..OPPONENTS].iter()
                        .map(|a| win_loss(&agent2, a)).sum::<i16>() as f64 / OPPONENTS as f64 + rng.random_range(-0.02..0.02);
                    
                    *agent = Agent { 
                        weights: std::array::from_fn(|i| weights[i] + learning_rate * perturb[i] * (wl1 - wl2) as f64 / (2.0 * c)),
                        fitness: wl1 + wl2
                    };
                }

                let completed_prev = completed.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1)).unwrap();
                unsafe {
                    let scol = if *started.as_ptr() != NUM_AGENTS as u32 { "\x1b[1;33m" } else { "\x1b[1;32m" };
                    print!("   --- {scol}Started: {}/{NUM_AGENTS}\x1b[0m\t\t\x1b[1;33mCompleted: {completed_prev}/{NUM_AGENTS}\x1b[0m\r", *started.as_ptr());
                    let _ = std::io::stdout().flush();
                }
            });

        best_agent = agents.iter().fold(best_agent, |a, b| if a.fitness > b.fitness { a } else { b.clone() });
        let best_agent_in_queue = agents.iter().fold({
            let mut a = Agent::new_random();
            a.fitness = f64::MIN;
            a
        }, |a, b| if a.fitness > b.fitness { a } else { b.clone() });

        println!("\x1b[1mAll current agents: \x1b[0m{:.5?}\n", agents);
        println!("\x1b[1mBest agent: \x1b[0m{:.9?}", best_agent);
        println!("\x1b[1mBest agent in queue: \x1b[0m{:.9?}", best_agent_in_queue);
    }
}