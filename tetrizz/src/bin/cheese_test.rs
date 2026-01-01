use utils::{
    game::Game,
    queue::extend_queue
};
use rand::Rng;
use tetrizz::{
    search::search,
    eval::base::MinimalEval
};

fn main() {
    let mut rng = rand::rng();
    let mut game = Game::new_empty();
    let mut last_col = rng.random_range(0..10);
    for _ in 0..10 {
        game.board.add_garbage(last_col, 1);
        let c = rng.random_range(0..9);
        last_col = if c >= last_col { c + 1 } else { c };
    }
    println!("{}", game.into_string(None));

    let eval = MinimalEval { values: [-0.3335903388433292, -0.25750971325030974, -0.05115233297009883, -0.3067533752401169, -0.1854127319664896, -0.1070342999530273, -0.3690213829655287, 0.09663015564572557, -0.4766250155906852, -0.05872828521759004, 1.06812123537880416, -1.0169857268318688, -0.4575993254187147, 2.28085733274743393, 0.002] };
    let mut queue = vec![];
    extend_queue(&mut queue, 5);
    loop {
        if queue.len() < 8 { extend_queue(&mut queue, 2) }
        let found_loc = search(&game, &queue[..7].iter().copied().collect(), &eval, 6, 50000);
        let found_loc = if let Some(s) = found_loc { s } else { break; };
        println!("{}", game.into_string(Some(&found_loc)));
        game.advance(queue[0], &found_loc);
        queue.remove(0);
        
        if found_loc.blocks().iter().any(|(_, y)| *y < 10) {
            game.board.add_garbage(last_col, 1);
            let c = rng.random_range(0..9);
            last_col = if c >= last_col { c + 1 } else { c };
        }
    }
}