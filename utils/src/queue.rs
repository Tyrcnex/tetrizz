use super::data::Piece;
use rand::prelude::SliceRandom;

pub fn extend_queue(queue: &mut Vec<Piece>, bags: usize) {
    let mut rng = rand::rng();
    let bag = [Piece::I, Piece::J, Piece::L, Piece::O, Piece::S, Piece::T, Piece::Z];
    for _ in 0..bags {
        let mut new_bag = bag.to_vec();
        new_bag.shuffle(&mut rng);
        queue.extend(new_bag);
    }
}