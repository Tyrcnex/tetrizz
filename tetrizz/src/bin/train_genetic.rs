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
    eval::base::{Eval, MinimalEval}
};

pub struct Player<T: Eval> {
    pub game: Game,
    pub queue: Vec<Piece>,
    pub eval: T
}

pub struct Battle<T: Eval, U: Eval> {
    pub player0: Player<T>,
    pub player1: Player<U>,
    pub who: u8
}

pub fn advance_player<T: Eval, U: Eval>(player: &mut Player<T>, opponent: &mut Player<U>) -> Option<(PieceLocation, PlacementInfo)> {
    let player_move = search(&player.game, &player.queue, &player.eval, 6, 3000);
    if let Some(p) = player_move {
        let info = player.game.advance(player.queue[0], &p);
        opponent.game.incoming_garbage += info.outgoing_attack;
        return Some((p, info));
    }
    None
}

impl<T: Eval, U: Eval> Battle<T, U> {
    pub fn advance(&mut self) -> Option<(PieceLocation, PlacementInfo)> {
        let res = if self.who == 0 {
            advance_player(&mut self.player0, &mut self.player1)
        } else {
            advance_player(&mut self.player1, &mut self.player0)
        };
        if res.is_none() { return None; }
        self.who = 1 - self.who;
        res
    }
}

const MAX_MOVES: usize = 200;

pub fn win_loss(agent: &Agent, opponent: &Agent) -> i16 {
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

    if !game_ended { -1 } else if battle.who == 0 { -1 } else { max_b2b }
}

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

    fn mutate(&mut self) {
        let mut rng = rand::rng();
        let idx = rng.random_range(0..self.eval.values.len());
        self.eval.values[idx] += rng.random_range(-0.1..0.1);
        self.eval.normalize();
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    const NUM_AGENTS: usize = 8;
    const GENETIC_ITERATIONS: usize = 500;
    const OPPONENTS: usize = 8;
    let base_agent = Agent { eval: MinimalEval { values: [-0.3335903388433292, -0.25750971325030974, -0.05115233297009883, -0.3067533752401169, -0.1854127319664896, -0.1070342999530273, -0.3690213829655287, 0.09663015564572557, -0.4766250155906852, -0.05872828521759004, 1.06812123537880416, -1.0169857268318688, -0.4575993254187147, 2.28085733274743393, 0.002] }, fitness: 0.0 };
    // let mut agents: Vec<Agent> = (0..NUM_AGENTS).map(|i| {
    //     let mut agent = base_agent.clone();
    //     if i > 0 { agent.mutate(); }
    //     else { agent.fitness = f64::NAN; }
    //     agent
    // }).collect();
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
                    let c = 0.3 * (-(n as f64) * 0.01).exp();
                    let learning_rate = 0.1 * (-(n as f64) * 0.02).exp();

                    let weights = agent.eval.values;
                    let perturb = weights.map(|_| if rng.random_bool(0.5) { 1.0 } else { -1.0 });
                    let agent1 = Agent {
                        eval: MinimalEval { values: std::array::from_fn(|i| weights[i] + c * perturb[i]) },
                        fitness: 0.0
                    };

                    let wl1 = rng.random_range(0.98..1.02) * agents0[..OPPONENTS].iter()
                        .map(|a| win_loss(&agent1, a)).sum::<i16>() as f64 / OPPONENTS as f64 + rng.random_range(-0.02..0.02);

                    let agent2 = Agent {
                        eval: MinimalEval { values: std::array::from_fn(|i| weights[i] - c * perturb[i]) },
                        fitness: 0.0
                    };

                    let wl2 = rng.random_range(0.98..1.02) * agents0[..OPPONENTS].iter()
                        .map(|a| win_loss(&agent2, a)).sum::<i16>() as f64 / OPPONENTS as f64 + rng.random_range(-0.02..0.02);
                    
                    *agent = Agent { 
                        eval: MinimalEval { values: std::array::from_fn(|i| weights[i] + learning_rate * perturb[i] * (wl1 - wl2) as f64 / (2.0 * c)) },
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

        println!("\x1b[1mAll current agents: \x1b[0m{:?}\n", agents);
        println!("\x1b[1mBest agent: \x1b[0m{:?}", best_agent);
        println!("\x1b[1mBest agent in queue: \x1b[0m{:?}", best_agent_in_queue);
    }
}