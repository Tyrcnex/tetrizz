use rand::seq::SliceRandom;

use rand::Rng;
use rand::prelude::IteratorRandom;
use rayon::prelude::*;

use std::sync::atomic::{AtomicU32, Ordering};
use std::io::Write;

use tetrizz::{
    utils::{
        data::{Piece, PieceLocation},
        game::{Game, PlacementInfo},
        queue::extend_queue
    },
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

pub fn eval_fitness(agent: &Agent, opponent: &Agent) -> f64 {
    const GAMES_PLAYED: usize = 2;
    const MAX_MOVES: usize = 1000;

    let mut fitness = 0.0;
    let mut total_max_b2b = 0.0;

    for _ in 0..GAMES_PLAYED {
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
        }
        fitness += if game_ended { 0.0 } else if battle.who == 0 { 1.0 } else { -1.0 };
        total_max_b2b += (max_b2b as f64).ln_1p();
    }
    fitness * total_max_b2b / GAMES_PLAYED as f64
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
        self.eval.values[idx] += rng.random_range(-20.0..20.0);
        self.eval.normalize();
    }
}

fn main() {
    rayon::ThreadPoolBuilder::new().num_threads(8).build_global().unwrap();

    const NUM_AGENTS: usize = 30;
    const GENETIC_ITERATIONS: usize = 100;
    const MUTATE: usize = 6;
    const NEW_AGENTS: usize = 17;
    const OPPONENTS: usize = 10;
    let mut agents: Vec<Agent> = (0..NUM_AGENTS).map(|_| Agent::new_random()).collect();

    let mut rng = rand::rng();
    let mut best_agent: Agent = Agent::new_random();
    best_agent.fitness = 0.0;

    for n in 0..GENETIC_ITERATIONS {
        println!("\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n🤩🤩🤩🤩🤩🤩🤩🤩\n\x1b[1mITERATION {}/{GENETIC_ITERATIONS}\x1b[0m", n + 1);
        let started = AtomicU32::new(0);
        let completed = AtomicU32::new(0);
        let agents0 = agents.clone();
        agents.par_iter_mut()
            .for_each(|agent| {
                agent.fitness = 0.0;
                let mut agents0 = agents0.clone();
                let mut rng = rand::rng();
                let start_prev = started.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1)).unwrap();
                unsafe { 
                    let scol = if start_prev != NUM_AGENTS as u32 { "\x1b[1;33m" } else { "\x1b[1;32m" };
                    print!("   --- {scol}Started: {start_prev}/{NUM_AGENTS}\x1b[0m\t\t\x1b[1;33mCompleted: {}/{NUM_AGENTS}\x1b[0m\r", *completed.as_ptr());
                    let _ = std::io::stdout().flush();
                }
                agents0.shuffle(&mut rng);
                for a in &agents0[..OPPONENTS] {
                    agent.fitness += eval_fitness(&agent, &a);
                }
                let completed_prev = completed.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(x + 1)).unwrap();
                unsafe {
                    let scol = if *started.as_ptr() != NUM_AGENTS as u32 { "\x1b[1;33m" } else { "\x1b[1;32m" };
                    print!("   --- {scol}Started: {}/{NUM_AGENTS}\x1b[0m\t\t\x1b[1;33mCompleted: {completed_prev}/{NUM_AGENTS}\x1b[0m\r", *started.as_ptr());
                    let _ = std::io::stdout().flush();
                }
            });

        best_agent = agents.iter().fold(best_agent, |a, b| if a.fitness > b.fitness { a } else { b.clone() });

        for _ in 0..MUTATE {
            let idx = rng.random_range(0..agents.len());
            let mut new_agent = agents[idx].clone();
            new_agent.mutate();
            new_agent.fitness = f64::MAX;
            agents.push(new_agent);
        }

        for _ in 0..NEW_AGENTS {
            let mut new_agent = Agent::new_random();
            new_agent.fitness = f64::MAX;
            agents.push(new_agent);
        }

        agents.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        agents.truncate(NUM_AGENTS);

        println!("\x1b[1mAll current agents: \x1b[0m{:?}\n", agents);
        println!("\x1b[1mBest agent: \x1b[0m{:?}", best_agent);
    }
}