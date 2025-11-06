use std::collections::BinaryHeap;

use utils::{
    game::Game,
    data::{Piece, PieceLocation},
    movegen::movegen
};
use crate::eval::base::Eval;


#[derive(Clone, Debug)]
pub struct Node {
    pub game: Game,
    pub id: usize,
    pub score: f64
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for Node {  }

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

pub fn search(root: &Game, queue: &Vec<Piece>, eval: &impl Eval, depth: usize, width: usize) -> Option<PieceLocation> {
    assert!(queue.len() >= depth);

    let mut heap: BinaryHeap<Node> = BinaryHeap::with_capacity(width + 1);
    let mut next: BinaryHeap<Node> = BinaryHeap::with_capacity(width + 1);
    let mut arena: Vec<PieceLocation> = vec![];
    let mut search_positions: Vec<PieceLocation> = vec![];

    movegen(&mut arena, &root.board, queue[0], Some(root.hold.unwrap_or_else(|| queue[1])), true);
    
    for (id, loc) in arena[..].iter().enumerate() {
        let mut game = root.clone();
        let placement_info = game.advance(queue[0], loc);
        // if !placement_info.b2b_clear && placement_info.lines_cleared > 0 {
        //     continue;
        // }
        search_positions.push(loc.clone());
        if !game.can_spawn_piece(queue[1]) {
            continue;
        }
        let score = eval.value(&game, &placement_info);
        insert_if_better(&mut heap, Node { game, id, score }, width);
    }
    
    for idx in 1..depth {
        for node in &heap {
            let current_piece = queue.get(idx).copied().or(node.game.hold);
            if current_piece.is_none() {
                break;
            }
            let current_piece = current_piece.unwrap();
            let next_piece = queue.get(idx + 1).copied();
            let start = movegen(&mut arena, &node.game.board, current_piece, node.game.hold.or(next_piece), true);
            for loc in &arena[start..] {
                let mut game = node.game.clone();
                let placement_info = game.advance(current_piece, &loc);
                // if !placement_info.b2b_clear && placement_info.lines_cleared > 0 {
                //     continue;
                // }
                if let Some(n) = next_piece && !game.can_spawn_piece(n) {
                    continue;
                }
                let score = eval.value(&game, &placement_info);
                insert_if_better(&mut next, Node { game, id: node.id, score }, width);
            }
        }
        if next.len() == 0 {
            break;
        }
        heap.clear();
        std::mem::swap(&mut heap, &mut next);
    }

    if let Some(m) = heap.into_iter().min() {
        return Some(search_positions[m.id].clone());
    }
    None
}

fn insert_if_better(heap: &mut BinaryHeap<Node>, node: Node, width: usize) {
    if heap.len() < width {
        heap.push(node);
    } else if let Some(worst) = heap.peek() {
        if node.score > worst.score {
            heap.pop();
            heap.push(node);
        }
    }
}