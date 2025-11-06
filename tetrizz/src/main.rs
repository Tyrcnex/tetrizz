use utils::{
    data::{Piece, PieceLocation},
    game::Game,
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

pub fn advance_player<T: Eval, U: Eval>(player: &mut Player<T>, opponent: &mut Player<U>) -> Option<PieceLocation> {
    let player_move = search(&player.game, &player.queue, &player.eval, 6, 10000);
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

fn main() {
    let mut queue0: Vec<Piece> = vec![];
    let mut queue1: Vec<Piece> = vec![];
    extend_queue(&mut queue0, 5);
    extend_queue(&mut queue1, 5);
    let mut battle = Battle {
        player0: Player {
            game: Game::new_empty(),
            queue: vec![],
            // eval: MinimalEval { values: [181.895492184286, -399.93570994040635, 381.4761690885799, -419.33154332619665, -83.48157030036961, -161.95787724878292, -116.73024914147564, -138.5506890753284, -388.288092562069, -240.1590094390449, -2.9401061494848495, -409.0500339178186, -207.62545452919613, 26.415515422135027] }
            eval: MinimalEval { values: [-73.93872853648777, 81.82811212276783, 60.57697336179847, 305.4549549464958, -133.27209591591566, -363.6837574131512, -95.27980969957166, 16.260463532806746, -318.0712253536878, 365.35144406460597, 272.04734928060304, -274.5635011733458, -384.6529856373499, 446.64435029914733] }
        },
        player1: Player {
            game: Game::new_empty(),
            queue: vec![],
            eval: MinimalEval { values: [-333.5903388433292, -257.50971325030974, -51.15233297009883, -306.7533752401169, -185.4127319664896, -107.0342999530273, -369.0213829655287, 96.63015564572557, -476.6250155906852, -58.72828521759004, 1068.12123537880416, -1016.9857268318688, -457.5993254187147, 2280.85733274743393] }
        },
        who: 0
    };
    loop {
        if queue0.len() <= 14 { extend_queue(&mut queue0, 1); }
        if queue1.len() <= 14 { extend_queue(&mut queue1, 1); }
        let oldgame0 = battle.player0.game.clone();
        let oldgame1 = battle.player1.game.clone();
        battle.player0.queue = queue0[..7].iter().copied().collect();
        battle.player1.queue = queue1[..7].iter().copied().collect();
        let who = battle.who;
        let result = battle.advance();
        if result.is_none() { break; }

        if who == 0 { queue0.remove(0); } else { queue1.remove(0); }
        
        let str0 = oldgame0.into_string(if who == 0 { result.as_ref() } else { None });
        let str1 = oldgame1.into_string(if who == 1 { result.as_ref() } else { None });
        let lines0: Vec<&str> = str0.lines().collect();
        let lines1: Vec<&str> = str1.lines().collect();
        println!("\n\n\n");
        for (line0, line1) in lines0.into_iter().zip(lines1.into_iter()) {
            println!("{}    {}", line0, line1);
        }
    }
}