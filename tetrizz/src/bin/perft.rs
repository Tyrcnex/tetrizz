use tetrizz::utils::{data::{Piece, PieceLocation, Board}, game::Game};
use tetrizz::movegen::{movegen_piece};

fn perft(arena: &mut Vec<PieceLocation>, game: &Game, queue: &[Piece; 7], idx: usize, depth: usize) -> usize {
    // if depth > 2 { arena.clear(); }
    let arena_idx = movegen_piece(arena, &game.board, queue[idx], true);
    if depth == 1 {
        return arena.len() - arena_idx;
    }

    let mut nodes = 0;
    let moves: Vec<PieceLocation> = arena[arena_idx..].iter().cloned().collect();
    for mv in moves {
        let mut next_game = game.clone();
        next_game.advance(queue[idx], &mv);
        nodes += perft(arena, &next_game, queue, idx + 1, depth - 1);
    }

    nodes
}

fn main() {
    let mut arena: Vec<PieceLocation> = vec![];
    let game = Game::new_empty();
    let queue = [Piece::I, Piece::O, Piece::L, Piece::J, Piece::S, Piece::Z, Piece::T];

    for d in 1..=7 {
        let now = std::time::Instant::now();
        let nodes = perft(&mut arena, &game, &queue, 0, d);
        let elapsed = now.elapsed().as_micros() as usize;
        println!("Depth: {d}  |  Nodes: {nodes}  |  Time: {}ms  |  NPS: {}", elapsed as f32 / 1000.0, nodes as f32 / (elapsed as f32 / 1000000.0));
    }
}