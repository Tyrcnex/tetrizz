use tetrizz::{
    utils::{
        data::Piece,
        game::Game
    },
    eval::base::MinimalEval,
    search::search,
    keygen::{MovementAction, keygen}
};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
struct InObj {
    game: Game,
    queue: Vec<Piece>,
    beam_width: usize,
    beam_depth: usize,
    human: bool
}

fn main() {
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .ok()
            .expect("lol wtf is this");

        let parsed: InObj = serde_json::from_str(&input).unwrap();

        let eval = MinimalEval { values: [-333.5903388433292, -257.50971325030974, -51.15233297009883, -306.7533752401169, -185.4127319664896, -107.0342999530273, -369.0213829655287, 96.63015564572557, -476.6250155906852, -58.72828521759004, 1068.12123537880416, -1016.9857268318688, -457.5993254187147, 2280.85733274743393] };

        let found_move = search(
            &parsed.game,
            &parsed.queue.clone(),
            &eval,
            parsed.beam_depth,
            parsed.beam_width,
        );

        let found_move = if let Some(m) = found_move { m } else {
            println!("[\"Harddrop\", \"Harddrop\"]");
            continue;
        };

        let mut keys = keygen(&parsed.game.board, &found_move, parsed.human).expect(&format!("Could not find keypresses!\nInput: {input}\nMove: {found_move:?}"));

        if found_move.piece != parsed.queue[0] {
            keys.insert(0, MovementAction::Hold);
        }

        keys.push(MovementAction::Harddrop);

        println!("{}\n", serde_json::to_string(&keys).unwrap());
    }
}