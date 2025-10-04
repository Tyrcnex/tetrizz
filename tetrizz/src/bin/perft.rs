use tetrizz::utils::{data::{Piece, Board}, game::Game};
use tetrizz::movegen::{movegen_piece};

fn perft(game: &Game, queue: &[Piece; 7], idx: usize, depth: usize) -> usize {
    if depth == 1 {
        return movegen_piece(&game.board, queue[idx], true).len();
    }

    let mut nodes = 0;
    for mv in movegen_piece(&game.board, queue[idx], true) {
        let mut next_game = game.clone();
        next_game.advance(queue[idx], &mv);
        nodes += perft(&next_game, queue, idx + 1, depth - 1);
    }

    nodes
}

fn main() {
    let mut game = Game {
        board: Board { cols: [3, 15, 12, 0, 0, 0, 0, 0, 0, 1] },
        hold: None,
        b2b: 0,
        combo: 0,
        incoming_garbage: 0
    };
    let queue = [Piece::I, Piece::O, Piece::L, Piece::J, Piece::S, Piece::Z, Piece::T];

    for d in 3..=7 {
        let now = std::time::Instant::now();
        let nodes = perft(&game, &queue, 0, d);
        let elapsed = now.elapsed().as_micros() as usize;
        println!("Depth: {d}  |  Nodes: {nodes}  |  Time: {}ms  |  NPS: {}", elapsed as f32 / 1000.0, nodes as f32 / (elapsed as f32 / 1000000.0));
    }
}