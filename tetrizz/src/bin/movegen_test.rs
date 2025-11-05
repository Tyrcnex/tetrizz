use tetrizz::{
    utils::data::{Board, Piece, PieceLocation},
    movegen::movegen_piece
};

fn main() {
    // let board = Board { cols: [6665,64,119,3608,0,18,703,9,4224,16583] };
    // let board = Board { cols: [1,0,77,112,19,16,113,192,0,12] };
    let board = Board { cols: [0,0,0,0,63,36,36,0,0,0] };
    let mut arena: Vec<PieceLocation> = vec![];
    movegen_piece(&mut arena, &board, Piece::O, true);
    println!("{arena:?}");
}