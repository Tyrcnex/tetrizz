use tetrizz::{
    utils::{
        data::{Board, Piece},
        game::Game
    },
    search::search,
    eval::base::MinimalEval
};

fn main() {
    let game = Game {
        board: Board { cols: [524287,262143,262143,131071,131071,131071,65534,21844,0,0] },
        hold: None,
        b2b: 10,
        combo: 2,
        incoming_garbage: 0
    };
    let queue = vec![Piece::I, Piece::S, Piece::J, Piece::Z, Piece::T, Piece::O, Piece::L, Piece::S, Piece::J, Piece::Z, Piece::T, Piece::L, Piece::O, Piece::I, Piece::S, Piece::T, Piece::Z];
    let eval = MinimalEval { values: [-333.5903388433292, -257.50971325030974, -51.15233297009883, -306.7533752401169, -185.4127319664896, -107.0342999530273, -369.0213829655287, 96.63015564572557, -476.6250155906852, -58.72828521759004, 1068.12123537880416, -1016.9857268318688, -457.5993254187147, 2280.85733274743393] };
    
    let mut res: Vec<f64> = vec![];
    for depth in 1..=15 {
        for width in (500..=20000).step_by(500) {
            let now = std::time::Instant::now();
            search(&game, &queue, &eval, depth, width);
            let elapsed = now.elapsed().as_micros() as f64 / 1000.0;
            println!("depth: {depth} | width: {width} | elapsed: {elapsed}ms");
            res.push(elapsed);
        }
    }
    println!("{res:?}");
}