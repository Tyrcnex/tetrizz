use utils::{
    data::Piece,
    game::Game,
    queue::extend_queue
};
use tetrizz::{
    eval::base::MinimalEval,
    battle::{Battle, Player}
};

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
            eval: MinimalEval { values: [0.09474412438177726, -0.07647412133403209, 0.8234850718463612, 0.4287350902132853, -0.4158156337764227, -0.48157954936416414, -0.5968990859714556, -0.36850072029357717, -0.26110351981581015, -0.058585592392512036, -0.02909739653482427, -0.3301758750048648, 0.2745008852729226, 1.3639010033368975, -0.8133272965458327]  }
        },
        player1: Player {
            game: Game::new_empty(),
            queue: vec![],
            eval: MinimalEval { values: [-0.3335903388433292, -0.25750971325030974, -0.05115233297009883, -0.3067533752401169, -0.1854127319664896, -0.1070342999530273, -0.3690213829655287, 0.09663015564572557, -0.4766250155906852, -0.05872828521759004, 1.06812123537880416, -1.0169857268318688, -0.4575993254187147, 2.28085733274743393, 0.002] }
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
    println!("{}", battle.who);
}