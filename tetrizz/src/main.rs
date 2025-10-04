use tetrizz::utils::{
    game::Game,
    data::Board
};

fn main() {
    let b1 = Board { cols: [43,53,55,62,44,60,31,29,31,30] };
    let b2 = Board { cols: [59,61,63,126,252,188,159,255,60,56] };
    println!("{:?}", b1 & b2);
}