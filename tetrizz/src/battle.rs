use utils::{
    data::{Piece, PieceLocation},
    game::Game
};
use crate::{
    eval::base::Eval,
    search::search
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

pub fn advance_player<T: Eval, U: Eval>(player: &mut Player<T>, opponent: &mut Player<U>) -> Option<PieceLocation> {
    let player_move = search(&player.game, &player.queue, &player.eval, 6, 5000);
    if let Some(p) = player_move {
        let info = player.game.advance(player.queue[0], &p);
        opponent.game.incoming_garbage += info.outgoing_attack;
        return Some(p);
    }
    None
}

impl<T: Eval, U: Eval> Battle<T, U> {
    pub fn advance(&mut self) -> Option<PieceLocation> {
        if self.who == 0 {
            self.who = 1;
            return advance_player(&mut self.player0, &mut self.player1);
        } else {
            self.who = 0;
            return advance_player(&mut self.player1, &mut self.player0);
        }
    }
}